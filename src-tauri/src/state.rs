use crate::{
    ble::BleManager, bridge::Bridge, file_logger::FileLogger, record::RecordManager,
    serial::SerialManager, stats::StatsCollector,
};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;

/// Global application state managed by Tauri.
pub struct AppState {
    pub serial_manager: Arc<Mutex<SerialManager>>,
    pub ble_manager: Arc<Mutex<BleManager>>,
    /// Per-port record managers. Key is port_id for serial, "ble" for BLE.
    pub record_managers: Arc<Mutex<HashMap<String, RecordManager>>>,
    /// Parsed usb-device.json for port enumeration.
    pub usb_json: serde_json::Value,
    /// Per-port file loggers.
    pub file_loggers: Arc<Mutex<HashMap<String, FileLogger>>>,
    /// Active bridge (only one at a time).
    pub active_bridge: Arc<Mutex<Option<Bridge>>>,
    /// Per-port stats collectors.
    pub stats_collectors: Arc<Mutex<HashMap<String, Arc<StatsCollector>>>>,
}
