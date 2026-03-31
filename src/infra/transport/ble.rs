use crate::{
    core::records::{DeviceInfo, DeviceKind},
    infra::transport::{AsyncTransport, TransportEvent, event_channel},
    state::{BleDeviceDescriptor, BleProfile},
};
use anyhow::{Context, Result, anyhow};
use async_trait::async_trait;
use btleplug::{
    api::{
        Central, CharPropFlags, Characteristic, Manager as _, Peripheral as _, ScanFilter,
        WriteType, bleuuid::uuid_from_u16,
    },
    platform::{Adapter, Manager, Peripheral},
};
use futures::{Stream, StreamExt};
use std::{pin::Pin, time::Duration};
use tokio::{sync::mpsc, task::JoinHandle, time::sleep};
use tokio_stream::wrappers::UnboundedReceiverStream;
use tokio_util::sync::CancellationToken;

#[derive(Debug)]
pub struct BleConnection {
    device: DeviceInfo,
    peripheral: Peripheral,
    write_characteristic: Characteristic,
    events: Option<mpsc::UnboundedReceiver<TransportEvent>>,
    notification_task: JoinHandle<()>,
    cancel: CancellationToken,
}

pub async fn list_devices(profile: &BleProfile) -> Result<Vec<BleDeviceDescriptor>> {
    let adapter = primary_adapter().await?;
    adapter.start_scan(ScanFilter::default()).await?;
    sleep(Duration::from_secs(2)).await;
    let target_service = uuid_from_u16(profile.service as u16);
    let mut devices = Vec::new();
    for peripheral in adapter.peripherals().await? {
        let properties = peripheral.properties().await?;
        let Some(properties) = properties else {
            continue;
        };
        let services_match = properties.services.contains(&target_service);
        if services_match || properties.local_name.is_some() {
            let id = format!("{}", peripheral.id());
            let name = properties
                .local_name
                .unwrap_or_else(|| "Bluetooth Device".into());
            let description = if services_match {
                format!("匹配服务 {}", target_service)
            } else {
                "附近蓝牙设备".into()
            };
            devices.push(BleDeviceDescriptor {
                id,
                name,
                description,
            });
        }
    }
    Ok(devices)
}

pub async fn connect(device_id: &str, profile: &BleProfile) -> Result<BleConnection> {
    let adapter = primary_adapter().await?;
    adapter.start_scan(ScanFilter::default()).await?;
    sleep(Duration::from_secs(2)).await;
    let peripheral = adapter
        .peripherals()
        .await?
        .into_iter()
        .find(|peripheral| format!("{}", peripheral.id()) == device_id)
        .ok_or_else(|| anyhow!("未找到蓝牙设备"))?;

    peripheral.connect().await?;
    peripheral.discover_services().await?;

    let write_uuid = uuid_from_u16(profile.write_characteristic as u16);
    let notify_uuid = uuid_from_u16(profile.notify_characteristic as u16);
    let characteristics = peripheral.characteristics();
    let write_characteristic = characteristics
        .iter()
        .find(|characteristic| {
            characteristic.uuid == write_uuid
                && (characteristic.properties.contains(CharPropFlags::WRITE)
                    || characteristic
                        .properties
                        .contains(CharPropFlags::WRITE_WITHOUT_RESPONSE))
        })
        .cloned()
        .ok_or_else(|| anyhow!("未找到蓝牙写特征"))?;
    let notify_characteristic = characteristics
        .iter()
        .find(|characteristic| characteristic.uuid == notify_uuid)
        .cloned()
        .ok_or_else(|| anyhow!("未找到蓝牙通知特征"))?;

    peripheral.subscribe(&notify_characteristic).await?;

    let properties = peripheral.properties().await?;
    let name = properties
        .as_ref()
        .and_then(|item| item.local_name.clone())
        .unwrap_or_else(|| "Bluetooth Device".into());
    let device = DeviceInfo {
        kind: DeviceKind::Bluetooth,
        id: device_id.to_string(),
        name,
        detail: profile.name.clone(),
    };
    let (event_tx, event_rx) = event_channel();
    let cancel = CancellationToken::new();
    let notification_task = spawn_notifications(
        peripheral.clone(),
        event_tx.clone(),
        cancel.clone(),
        device.clone(),
    );
    let _ = event_tx.send(TransportEvent::Connected(device.clone()));

    Ok(BleConnection {
        device,
        peripheral,
        write_characteristic,
        events: Some(event_rx),
        notification_task,
        cancel,
    })
}

async fn primary_adapter() -> Result<Adapter> {
    let manager = Manager::new().await?;
    manager
        .adapters()
        .await?
        .into_iter()
        .next()
        .ok_or_else(|| anyhow!("未找到蓝牙适配器"))
}

fn spawn_notifications(
    peripheral: Peripheral,
    event_tx: mpsc::UnboundedSender<TransportEvent>,
    cancel: CancellationToken,
    _device: DeviceInfo,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        let mut notifications = match peripheral.notifications().await {
            Ok(stream) => stream,
            Err(error) => {
                let _ = event_tx.send(TransportEvent::Error(error.to_string()));
                return;
            }
        };
        loop {
            tokio::select! {
                _ = cancel.cancelled() => {
                    let _ = event_tx.send(TransportEvent::Disconnected);
                    break;
                }
                message = notifications.next() => {
                    match message {
                        Some(value) => {
                            let chunk = value.value;
                            let _ = event_tx.send(TransportEvent::BytesReceived(chunk.clone()));
                            let _ = event_tx.send(TransportEvent::FrameReceived(chunk));
                        }
                        None => {
                            let _ = event_tx.send(TransportEvent::Disconnected);
                            break;
                        }
                    }
                }
            }
        }
    })
}

#[async_trait]
impl AsyncTransport for BleConnection {
    async fn disconnect(&mut self) -> Result<()> {
        self.cancel.cancel();
        self.notification_task.abort();
        if self.peripheral.is_connected().await.unwrap_or(false) {
            self.peripheral.disconnect().await?;
        }
        Ok(())
    }

    async fn send(&self, bytes: &[u8]) -> Result<()> {
        self.peripheral
            .write(
                &self.write_characteristic,
                bytes,
                WriteType::WithoutResponse,
            )
            .await
            .context("蓝牙发送失败")
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
