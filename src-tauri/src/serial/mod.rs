use std::{sync::Arc, time::Duration};

use anyhow::{Result, anyhow, bail};
use chrono::{SecondsFormat, Utc};
use serde::Serialize;
use tauri::{AppHandle, Emitter};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, WriteHalf},
    select,
    sync::{Mutex, watch},
};
use tokio_serial::{
    DataBits, FlowControl, Parity, SerialPortBuilderExt, SerialPortType, SerialStream, StopBits,
};

pub mod hotplug;
pub mod types;

use types::{
    FlowControl as ConfigFlowControl, LineEnding, Parity as ConfigParity, PortInfo, SendFormat,
    SerialConfig, SerialStatus,
};

const READ_BUFFER_SIZE: usize = 4096;

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "lowercase")]
enum Direction {
    Receive,
    Send,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SerialFrameEvent {
    direction: Direction,
    bytes: Vec<u8>,
    hex: String,
    text: String,
    timestamp: String,
}

impl SerialFrameEvent {
    fn new(direction: Direction, bytes: Vec<u8>) -> Self {
        Self {
            direction,
            hex: bytes_to_hex(&bytes),
            text: String::from_utf8_lossy(&bytes).into_owned(),
            bytes,
            timestamp: Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true),
        }
    }
}

struct SerialConnection {
    generation: u64,
    writer: Arc<Mutex<WriteHalf<SerialStream>>>,
    cancel: watch::Sender<bool>,
}

#[derive(Default)]
struct SerialState {
    connection: Option<SerialConnection>,
    generation: u64,
    path: Option<String>,
    config: Option<SerialConfig>,
    rx_bytes: u64,
    tx_bytes: u64,
    opened_at: Option<String>,
    last_error: Option<String>,
}

impl SerialState {
    fn status(&self) -> SerialStatus {
        SerialStatus {
            connected: self.connection.is_some(),
            path: self.path.clone(),
            config: self.config.clone(),
            rx_bytes: self.rx_bytes,
            tx_bytes: self.tx_bytes,
            opened_at: self.opened_at.clone(),
            last_error: self.last_error.clone(),
        }
    }
}

pub struct SerialManager {
    state: Arc<Mutex<SerialState>>,
    app_handle: AppHandle,
}

impl SerialManager {
    pub fn new(app_handle: AppHandle) -> Self {
        Self {
            state: Arc::new(Mutex::new(SerialState::default())),
            app_handle,
        }
    }

    pub fn list_ports() -> Result<Vec<PortInfo>> {
        let mut ports = tokio_serial::available_ports()
            .map_err(|error| anyhow!("无法读取串口列表：{error}"))?
            .into_iter()
            .map(|port| {
                let (manufacturer, serial_number, vid, pid, product) = match port.port_type {
                    SerialPortType::UsbPort(usb) => (
                        usb.manufacturer,
                        usb.serial_number,
                        Some(format!("{:04X}", usb.vid)),
                        Some(format!("{:04X}", usb.pid)),
                        usb.product,
                    ),
                    _ => (None, None, None, None, None),
                };

                PortInfo {
                    display_name: product.unwrap_or_else(|| short_port_name(&port.port_name)),
                    path: port.port_name,
                    manufacturer,
                    serial_number,
                    vid,
                    pid,
                }
            })
            .collect::<Vec<_>>();
        ports.sort_by(|a, b| a.path.cmp(&b.path));
        Ok(ports)
    }

    pub async fn connect(&self, path: String, config: SerialConfig) -> Result<SerialStatus> {
        config.validate()?;
        if path.trim().is_empty() {
            bail!("请选择串口");
        }

        let stream = serial_builder(&path, &config)
            .open_native_async()
            .map_err(|error| anyhow!("无法打开 {path}：{error}"))?;
        let (reader, writer) = tokio::io::split(stream);
        let (cancel, mut cancelled) = watch::channel(false);

        let (generation, old_connection, status) = {
            let mut state = self.state.lock().await;
            state.generation = state.generation.wrapping_add(1);
            let generation = state.generation;
            let old_connection = state.connection.take();
            state.connection = Some(SerialConnection {
                generation,
                writer: Arc::new(Mutex::new(writer)),
                cancel,
            });
            state.path = Some(path);
            state.config = Some(config);
            state.rx_bytes = 0;
            state.tx_bytes = 0;
            state.opened_at = Some(Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true));
            state.last_error = None;
            (generation, old_connection, state.status())
        };

        if let Some(old) = old_connection {
            let _ = old.cancel.send(true);
        }

        let state = Arc::clone(&self.state);
        let app_handle = self.app_handle.clone();
        tauri::async_runtime::spawn(async move {
            let mut reader = reader;
            let mut buffer = [0_u8; READ_BUFFER_SIZE];

            loop {
                select! {
                    changed = cancelled.changed() => {
                        if changed.is_err() || *cancelled.borrow() {
                            break;
                        }
                    }
                    result = reader.read(&mut buffer) => {
                        match result {
                            Ok(0) => {
                                finish_connection(&state, &app_handle, generation, Some("串口已断开".into())).await;
                                break;
                            }
                            Ok(count) => {
                                let bytes = buffer[..count].to_vec();
                                let is_current = {
                                    let mut state = state.lock().await;
                                    let is_current = state.connection.as_ref().is_some_and(|connection| connection.generation == generation);
                                    if is_current {
                                        state.rx_bytes = state.rx_bytes.saturating_add(count as u64);
                                    }
                                    is_current
                                };
                                if is_current {
                                    let _ = app_handle.emit("serial:frame", SerialFrameEvent::new(Direction::Receive, bytes));
                                }
                            }
                            Err(error) => {
                                finish_connection(&state, &app_handle, generation, Some(format!("串口读取失败：{error}"))).await;
                                break;
                            }
                        }
                    }
                }
            }
        });

        self.emit_status(&status);
        Ok(status)
    }

    pub async fn disconnect(&self) -> SerialStatus {
        let (connection, status) = {
            let mut state = self.state.lock().await;
            let connection = state.connection.take();
            state.last_error = None;
            (connection, state.status())
        };
        if let Some(connection) = connection {
            let _ = connection.cancel.send(true);
        }
        self.emit_status(&status);
        status
    }

    pub async fn send(
        &self,
        input: String,
        format: SendFormat,
        line_ending: LineEnding,
    ) -> Result<()> {
        let mut bytes = encode_input(&input, format)?;
        if matches!(format, SendFormat::Text) {
            bytes.extend_from_slice(line_ending.bytes());
        }
        if bytes.is_empty() {
            bail!("发送内容不能为空");
        }

        let (generation, writer) = {
            let state = self.state.lock().await;
            let connection = state
                .connection
                .as_ref()
                .ok_or_else(|| anyhow!("串口未连接"))?;
            (connection.generation, Arc::clone(&connection.writer))
        };

        let write_result = async {
            let mut writer = writer.lock().await;
            writer.write_all(&bytes).await?;
            writer.flush().await
        }
        .await;

        if let Err(error) = write_result {
            let reason = format!("串口发送失败：{error}");
            finish_connection(
                &self.state,
                &self.app_handle,
                generation,
                Some(reason.clone()),
            )
            .await;
            bail!(reason);
        }

        {
            let mut state = self.state.lock().await;
            if state
                .connection
                .as_ref()
                .is_some_and(|connection| connection.generation == generation)
            {
                state.tx_bytes = state.tx_bytes.saturating_add(bytes.len() as u64);
            }
        }

        let event = SerialFrameEvent::new(Direction::Send, bytes);
        let _ = self.app_handle.emit("serial:frame", &event);
        Ok(())
    }

    pub async fn status(&self) -> SerialStatus {
        self.state.lock().await.status()
    }

    fn emit_status(&self, status: &SerialStatus) {
        let _ = self.app_handle.emit("serial:status", status);
    }
}

fn serial_builder(path: &str, config: &SerialConfig) -> tokio_serial::SerialPortBuilder {
    tokio_serial::new(path, config.baud_rate)
        .data_bits(match config.data_bits {
            5 => DataBits::Five,
            6 => DataBits::Six,
            7 => DataBits::Seven,
            _ => DataBits::Eight,
        })
        .stop_bits(if config.stop_bits == 2 {
            StopBits::Two
        } else {
            StopBits::One
        })
        .parity(match config.parity {
            ConfigParity::None => Parity::None,
            ConfigParity::Odd => Parity::Odd,
            ConfigParity::Even => Parity::Even,
        })
        .flow_control(match config.flow_control {
            ConfigFlowControl::None => FlowControl::None,
            ConfigFlowControl::Software => FlowControl::Software,
            ConfigFlowControl::Hardware => FlowControl::Hardware,
        })
        .timeout(Duration::from_millis(100))
}

async fn finish_connection(
    state: &Arc<Mutex<SerialState>>,
    app_handle: &AppHandle,
    generation: u64,
    error: Option<String>,
) {
    let status = {
        let mut state = state.lock().await;
        if !state
            .connection
            .as_ref()
            .is_some_and(|connection| connection.generation == generation)
        {
            return;
        }
        state.connection = None;
        state.last_error = error;
        state.status()
    };
    let _ = app_handle.emit("serial:status", status);
}

fn short_port_name(path: &str) -> String {
    path.rsplit('/').next().unwrap_or(path).to_string()
}

fn bytes_to_hex(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|byte| format!("{byte:02X}"))
        .collect::<Vec<_>>()
        .join(" ")
}

fn encode_input(input: &str, format: SendFormat) -> Result<Vec<u8>> {
    match format {
        SendFormat::Text => Ok(input.as_bytes().to_vec()),
        SendFormat::Hex => parse_hex(input),
    }
}

fn parse_hex(input: &str) -> Result<Vec<u8>> {
    let compact = input
        .replace("0x", "")
        .replace("0X", "")
        .chars()
        .filter(|character| !character.is_whitespace() && *character != ',' && *character != '-')
        .collect::<String>();
    if compact.is_empty() {
        return Ok(Vec::new());
    }
    if compact.len() % 2 != 0 {
        bail!("十六进制内容必须由完整字节组成");
    }
    (0..compact.len())
        .step_by(2)
        .map(|index| {
            u8::from_str_radix(&compact[index..index + 2], 16)
                .map_err(|_| anyhow!("包含无效的十六进制字符"))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_common_hex_input_styles() {
        assert_eq!(parse_hex("AA 01 ff").unwrap(), vec![0xAA, 0x01, 0xFF]);
        assert_eq!(parse_hex("0xAA,0x01").unwrap(), vec![0xAA, 0x01]);
        assert_eq!(parse_hex("AA-01").unwrap(), vec![0xAA, 0x01]);
    }

    #[test]
    fn rejects_incomplete_hex_bytes() {
        assert!(parse_hex("A 01").is_err());
    }

    #[test]
    fn line_endings_are_only_bytes() {
        assert_eq!(LineEnding::Crlf.bytes(), b"\r\n");
    }
}
