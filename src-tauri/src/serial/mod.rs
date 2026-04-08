use anyhow::{Result, anyhow};
use chrono::Utc;
use ringbuf::traits::{Consumer, Producer, Split};
use serde::Serialize;
use std::{collections::HashMap, sync::Arc, time::Duration};
use tauri::{AppHandle, Emitter};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    select,
    sync::{
        Mutex,
        watch::{Sender, channel},
    },
    time::sleep,
};
use tokio_serial::{
    DataBits, FlowControl, Parity, SerialPortBuilderExt, SerialPortType, SerialStream, StopBits,
};

pub mod hotplug;
pub mod types;

use types::{FlowControl as CfgFlowControl, Parity as CfgParity, PortInfo, SerialConfig};

use crate::{
    codec::{bytes_to_ascii_string, bytes_to_dec_string, bytes_to_hex_display},
    file_logger::FileLogger,
    stats::StatsCollector,
};

/// Payload emitted for each received serial frame.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SerialFrameEvent {
    port_id: String,
    raw: Vec<u8>,
    hex: String,
    ascii: String,
    dec: String,
    timestamp: String,
}

/// Internal state for a single open serial connection.
struct SerialConnection {
    stream: Arc<Mutex<SerialStream>>,
    cancel_token: Sender<bool>,
}

/// Manages multiple concurrent serial port connections.
pub struct SerialManager {
    connections: Arc<Mutex<HashMap<String, SerialConnection>>>,
    app_handle: AppHandle,
}

impl SerialManager {
    pub fn new(app_handle: AppHandle) -> Self {
        Self {
            connections: Arc::new(Mutex::new(HashMap::new())),
            app_handle,
        }
    }

    /// List available serial ports, enriching USB ports with device names from
    /// the provided `usb_json` lookup table.
    ///
    /// The JSON structure is:
    /// ```json
    /// { "1a86": { "name": "QinHeng Electronics", "devices": { "7523": { "name": "CH340 serial converter", "devname": "CH340" } } } }
    /// ```
    pub fn list_ports(usb_json: &serde_json::Value) -> Vec<PortInfo> {
        let ports = match tokio_serial::available_ports() {
            Ok(p) => p,
            Err(_) => return Vec::new(),
        };

        ports
            .into_iter()
            .map(|info| {
                let (vid, pid, device_name) = match &info.port_type {
                    SerialPortType::UsbPort(usb) => {
                        let vid_hex = format!("{:04x}", usb.vid);
                        let pid_hex = format!("{:04x}", usb.pid);

                        let name = usb_json
                            .get(&vid_hex)
                            .and_then(|vendor| vendor.get("devices"))
                            .and_then(|devices| devices.get(&pid_hex))
                            .and_then(|device| {
                                device
                                    .get("devname")
                                    .or_else(|| device.get("name"))
                                    .and_then(|v| v.as_str())
                                    .map(String::from)
                            });

                        (Some(vid_hex), Some(pid_hex), name)
                    }
                    _ => (None, None, None),
                };

                PortInfo {
                    path: info.port_name,
                    vid,
                    pid,
                    device_name,
                }
            })
            .collect()
    }

    /// Open a serial port and start a background read task that aggregates
    /// incoming bytes into frames using a 5 ms silence timeout.
    ///
    /// If a `StatsCollector` is provided, the read loop will call `record_rx`
    /// for each emitted frame.
    pub async fn open_port(
        &self,
        port_id: String,
        path: String,
        config: SerialConfig,
        stats: Option<Arc<StatsCollector>>,
        file_loggers: Arc<Mutex<HashMap<String, FileLogger>>>,
    ) -> Result<()> {
        let mut conns = self.connections.lock().await;
        if conns.contains_key(&port_id) {
            return Err(anyhow!("port '{}' is already open", port_id));
        }

        // Build and open the serial stream
        let builder = tokio_serial::new(&path, config.baud_rate)
            .data_bits(match config.data_bits {
                5 => DataBits::Five,
                6 => DataBits::Six,
                7 => DataBits::Seven,
                _ => DataBits::Eight,
            })
            .stop_bits(match config.stop_bits {
                2 => StopBits::Two,
                _ => StopBits::One,
            })
            .parity(match config.parity {
                CfgParity::Odd => Parity::Odd,
                CfgParity::Even => Parity::Even,
                CfgParity::None => Parity::None,
            })
            .flow_control(match config.flow_control {
                CfgFlowControl::Software => FlowControl::Software,
                CfgFlowControl::Hardware => FlowControl::Hardware,
                CfgFlowControl::None => FlowControl::None,
            });

        let stream = builder
            .open_native_async()
            .map_err(|e| anyhow!("failed to open serial port '{}': {}", path, e))?;

        let stream = Arc::new(Mutex::new(stream));
        let (cancel_tx, mut cancel_rx) = channel(false);

        let connection = SerialConnection {
            stream: Arc::clone(&stream),
            cancel_token: cancel_tx,
        };
        conns.insert(port_id.clone(), connection);

        // Spawn the background read task
        let app_handle = self.app_handle.clone();
        let task_port_id = port_id.clone();

        tauri::async_runtime::spawn(async move {
            let (mut prod, mut cons) = ringbuf::HeapRb::<u8>::new(64 * 1024).split();
            let mut tmp = [0u8; 1024];

            loop {
                select! {
                    _ = cancel_rx.changed() => break,
                    result = async {
                        let mut guard = stream.lock().await;
                        guard.read(&mut tmp).await
                    } => {
                        match result {
                            Ok(0) => break, // EOF
                            Ok(n) => {
                                prod.push_slice(&tmp[..n]);
                                // Wait for 5 ms of silence to aggregate a frame
                                loop {
                                    select! {
                                        _ = sleep(Duration::from_millis(5)) => {
                                            // Silence timeout — emit the frame
                                            let data: Vec<u8> = cons.pop_iter().collect();
                                            if !data.is_empty() {
                                                if let Some(ref s) = stats {
                                                    s.record_rx(data.len() as u64);
                                                }
                                                // Log to file if active
                                                {
                                                    let mut loggers = file_loggers.lock().await;
                                                    if let Some(logger) = loggers.get_mut(&task_port_id) {
                                                        let _ = logger.log_data(&crate::file_logger::DataDirection::Rx, &data).await;
                                                    }
                                                }
                                                let event = SerialFrameEvent {
                                                    port_id: task_port_id.clone(),
                                                    hex: bytes_to_hex_display(&data),
                                                    ascii: bytes_to_ascii_string(&data),
                                                    dec: bytes_to_dec_string(&data),
                                                    raw: data,
                                                    timestamp: Utc::now().to_rfc3339(),
                                                };
                                                let _ = app_handle.emit("serial:frame", &event);
                                            }
                                            break;
                                        }
                                        more = async {
                                            let mut guard = stream.lock().await;
                                            guard.read(&mut tmp).await
                                        } => {
                                            match more {
                                                Ok(0) => break,
                                                Ok(n) => { prod.push_slice(&tmp[..n]); }
                                                Err(_) => break,
                                            }
                                        }
                                    }
                                }
                            }
                            Err(_) => break,
                        }
                    }
                }
            }
        });

        Ok(())
    }

    /// Write data to an open serial port.
    pub async fn write_port(&self, port_id: &str, data: &[u8]) -> Result<()> {
        let conns = self.connections.lock().await;
        let conn = conns
            .get(port_id)
            .ok_or_else(|| anyhow!("port '{}' is not open", port_id))?;

        let mut stream = conn.stream.lock().await;
        stream
            .write_all(data)
            .await
            .map_err(|e| anyhow!("write to port '{}' failed: {}", port_id, e))?;
        stream
            .flush()
            .await
            .map_err(|e| anyhow!("flush port '{}' failed: {}", port_id, e))?;

        Ok(())
    }

    /// Close a single serial port by its ID.
    pub async fn close_port(&self, port_id: &str) -> Result<()> {
        let mut conns = self.connections.lock().await;
        let conn = conns
            .remove(port_id)
            .ok_or_else(|| anyhow!("port '{}' is not open", port_id))?;

        // Signal the read task to stop
        let _ = conn.cancel_token.send(true);
        Ok(())
    }

    /// Close all open serial ports.
    pub async fn close_all(&self) -> Result<()> {
        let mut conns = self.connections.lock().await;
        for (_, conn) in conns.drain() {
            let _ = conn.cancel_token.send(true);
        }
        Ok(())
    }

    /// Return a list of currently open port IDs.
    pub async fn get_open_ports(&self) -> Vec<String> {
        let conns = self.connections.lock().await;
        conns.keys().cloned().collect()
    }

    /// Get a clone of the stream Arc for a given port. Used by the bridge module.
    pub async fn get_stream(&self, port_id: &str) -> Result<Arc<Mutex<SerialStream>>> {
        let conns = self.connections.lock().await;
        let conn = conns
            .get(port_id)
            .ok_or_else(|| anyhow!("port '{}' is not open", port_id))?;
        Ok(Arc::clone(&conn.stream))
    }
}
