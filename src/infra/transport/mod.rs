pub mod ble;
pub mod serial;

use std::pin::Pin;

use anyhow::Result;
use async_trait::async_trait;
use futures::Stream;
use tokio::sync::mpsc;

use crate::core::records::DeviceInfo;
use crate::state::{BleDeviceDescriptor, BleProfile, SerialConfig, SerialPortDescriptor};

#[derive(Debug, Clone)]
pub enum TransportEvent {
    Connected(DeviceInfo),
    Disconnected,
    BytesReceived(Vec<u8>),
    FrameReceived(Vec<u8>),
    Error(String),
}

#[async_trait]
pub trait AsyncTransport {
    async fn disconnect(&mut self) -> Result<()>;
    async fn send(&self, bytes: &[u8]) -> Result<()>;
    fn subscribe(&mut self) -> Pin<Box<dyn Stream<Item = TransportEvent> + Send>>;
    fn current_device_info(&self) -> Option<DeviceInfo>;
}

pub enum ActiveTransport {
    Serial(serial::SerialConnection),
    Ble(ble::BleConnection),
}

impl ActiveTransport {
    pub async fn disconnect(&mut self) -> Result<()> {
        match self {
            Self::Serial(serial) => serial.disconnect().await,
            Self::Ble(ble) => ble.disconnect().await,
        }
    }

    pub async fn send(&self, bytes: &[u8]) -> Result<()> {
        match self {
            Self::Serial(serial) => serial.send(bytes).await,
            Self::Ble(ble) => ble.send(bytes).await,
        }
    }

    pub fn take_events(&mut self) -> Pin<Box<dyn Stream<Item = TransportEvent> + Send>> {
        match self {
            Self::Serial(serial) => serial.subscribe(),
            Self::Ble(ble) => ble.subscribe(),
        }
    }

    pub fn device(&self) -> Option<DeviceInfo> {
        match self {
            Self::Serial(serial) => serial.current_device_info(),
            Self::Ble(ble) => ble.current_device_info(),
        }
    }
}

pub fn event_channel() -> (
    mpsc::UnboundedSender<TransportEvent>,
    mpsc::UnboundedReceiver<TransportEvent>,
) {
    mpsc::unbounded_channel()
}

#[allow(dead_code)]
pub enum TransportDiscovery {
    SerialPorts(Vec<SerialPortDescriptor>),
    BleDevices(Vec<BleDeviceDescriptor>),
    BleProfile(BleProfile),
    SerialConfig(SerialConfig),
}
