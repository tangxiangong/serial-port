use std::path::PathBuf;
use std::pin::Pin;
use std::time::Duration;

use anyhow::{Context, Result, anyhow};
use async_trait::async_trait;
use futures::Stream;
use serde::Deserialize;
use tokio::io::{AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf};
use tokio::sync::{Mutex, mpsc};
use tokio::task::JoinHandle;
use tokio::time::{Instant, sleep_until};
use tokio_serial::{
    DataBits, FlowControl, Parity, SerialPortBuilderExt, SerialPortType, SerialStream, StopBits,
    UsbPortInfo, available_ports,
};
use tokio_stream::wrappers::UnboundedReceiverStream;
use tokio_util::sync::CancellationToken;

use crate::core::records::{DeviceInfo, DeviceKind};
use crate::infra::transport::{AsyncTransport, TransportEvent, event_channel};
use crate::state::{SerialConfig, SerialPortDescriptor};

#[derive(Debug, Deserialize)]
struct UsbDeviceEntry {
    #[serde(default)]
    devname: Option<String>,
}

#[derive(Debug, Deserialize)]
struct UsbVendorEntry {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    devices: std::collections::HashMap<String, UsbDeviceEntry>,
}

#[derive(Debug, Default, Deserialize)]
struct UsbDeviceRegistry(std::collections::HashMap<String, UsbVendorEntry>);

fn usb_registry_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("usb-device.json")
}

fn load_usb_registry() -> UsbDeviceRegistry {
    std::fs::read(usb_registry_path())
        .ok()
        .and_then(|bytes| {
            serde_json::from_slice::<std::collections::HashMap<String, UsbVendorEntry>>(&bytes).ok()
        })
        .map(UsbDeviceRegistry)
        .unwrap_or_default()
}

fn usb_lookup_name(registry: &UsbDeviceRegistry, usb: &UsbPortInfo) -> Option<String> {
    let vendor_key = format!("{:04x}", usb.vid);
    let product_key = format!("{:04x}", usb.pid);
    let vendor = registry.0.get(&vendor_key)?;
    if let Some(device_name) = vendor
        .devices
        .get(&product_key)
        .and_then(|device| device.devname.clone())
    {
        return Some(device_name);
    }
    if let Some(product) = usb.product.clone() {
        return Some(product);
    }
    vendor.name.clone()
}

#[derive(Debug)]
pub struct SerialConnection {
    device: DeviceInfo,
    writer: std::sync::Arc<Mutex<WriteHalf<SerialStream>>>,
    events: Option<mpsc::UnboundedReceiver<TransportEvent>>,
    read_task: JoinHandle<()>,
    cancel: CancellationToken,
}

pub async fn list_ports() -> Result<Vec<SerialPortDescriptor>> {
    let registry = load_usb_registry();
    let ports = tokio::task::spawn_blocking(available_ports)
        .await
        .context("列举串口任务失败")??;
    let ports = ports
        .into_iter()
        .map(|port| {
            let display_name = match &port.port_type {
                SerialPortType::UsbPort(usb) => usb_lookup_name(&registry, usb)
                    .or_else(|| usb.manufacturer.clone())
                    .unwrap_or_else(|| port.port_name.clone()),
                _ => port.port_name.clone(),
            };
            let kind = match &port.port_type {
                SerialPortType::UsbPort(usb) => {
                    format!("USB {:04x}:{:04x}", usb.vid, usb.pid)
                }
                SerialPortType::BluetoothPort => "Bluetooth".to_string(),
                SerialPortType::PciPort => "PCI".to_string(),
                SerialPortType::Unknown => "Unknown".to_string(),
            };
            SerialPortDescriptor {
                display_name,
                kind,
                port_name: port.port_name,
            }
        })
        .collect();
    Ok(ports)
}

pub async fn connect(port_name: &str, config: &SerialConfig) -> Result<SerialConnection> {
    let mut builder = tokio_serial::new(port_name, config.baud_rate);
    builder = builder
        .data_bits(match config.data_bits {
            5 => DataBits::Five,
            6 => DataBits::Six,
            7 => DataBits::Seven,
            _ => DataBits::Eight,
        })
        .parity(match config.parity.as_str() {
            "odd" => Parity::Odd,
            "even" => Parity::Even,
            _ => Parity::None,
        })
        .flow_control(match config.flow_control.as_str() {
            "hardware" => FlowControl::Hardware,
            "software" => FlowControl::Software,
            _ => FlowControl::None,
        })
        .stop_bits(match config.stop_bits as i32 {
            2 => StopBits::Two,
            _ => StopBits::One,
        });

    let stream = builder
        .open_native_async()
        .with_context(|| format!("打开串口 {port_name} 失败"))?;
    let (reader, writer) = tokio::io::split(stream);
    let writer = std::sync::Arc::new(Mutex::new(writer));
    let (event_tx, event_rx) = event_channel();
    let cancel = CancellationToken::new();
    let read_task = spawn_reader(reader, event_tx.clone(), cancel.clone());
    let device = DeviceInfo {
        kind: DeviceKind::Serial,
        id: port_name.to_string(),
        name: port_name.to_string(),
        detail: format!("{} baud", config.baud_rate),
    };
    let _ = event_tx.send(TransportEvent::Connected(device.clone()));
    Ok(SerialConnection {
        device,
        writer,
        events: Some(event_rx),
        read_task,
        cancel,
    })
}

fn spawn_reader(
    mut reader: ReadHalf<SerialStream>,
    event_tx: mpsc::UnboundedSender<TransportEvent>,
    cancel: CancellationToken,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        let mut read_buf = [0u8; 4096];
        let mut frame_buf = Vec::new();
        let mut deadline = None::<Instant>;

        loop {
            tokio::select! {
                _ = cancel.cancelled() => {
                    let _ = event_tx.send(TransportEvent::Disconnected);
                    break;
                }
                _ = async {
                    if let Some(deadline) = deadline {
                        sleep_until(deadline).await;
                    } else {
                        futures::future::pending::<()>().await;
                    }
                }, if deadline.is_some() => {
                    if !frame_buf.is_empty() {
                        let frame = std::mem::take(&mut frame_buf);
                        let _ = event_tx.send(TransportEvent::FrameReceived(frame));
                    }
                    deadline = None;
                }
                read = reader.read(&mut read_buf) => {
                    match read {
                        Ok(0) => {
                            let _ = event_tx.send(TransportEvent::Disconnected);
                            break;
                        }
                        Ok(size) => {
                            let chunk = read_buf[..size].to_vec();
                            frame_buf.extend_from_slice(&chunk);
                            deadline = Some(Instant::now() + Duration::from_millis(5));
                            let _ = event_tx.send(TransportEvent::BytesReceived(chunk));
                        }
                        Err(error) => {
                            let _ = event_tx.send(TransportEvent::Error(error.to_string()));
                            break;
                        }
                    }
                }
            }
        }
    })
}

#[async_trait]
impl AsyncTransport for SerialConnection {
    async fn disconnect(&mut self) -> Result<()> {
        self.cancel.cancel();
        self.read_task.abort();
        Ok(())
    }

    async fn send(&self, bytes: &[u8]) -> Result<()> {
        let mut writer = self.writer.lock().await;
        writer.write_all(bytes).await.context("串口发送失败")
    }

    fn subscribe(&mut self) -> Pin<Box<dyn Stream<Item = TransportEvent> + Send>> {
        let rx = self
            .events
            .take()
            .ok_or_else(|| anyhow!("transport stream already taken"))
            .unwrap_or_else(|_| {
                let (_, rx) = event_channel();
                rx
            });
        Box::pin(UnboundedReceiverStream::new(rx))
    }

    fn current_device_info(&self) -> Option<DeviceInfo> {
        Some(self.device.clone())
    }
}
