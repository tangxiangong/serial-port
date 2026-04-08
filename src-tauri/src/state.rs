use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::ble::BleManager;
use crate::bridge::Bridge;
use crate::file_logger::FileLogger;
use crate::record::RecordManager;
use crate::serial::SerialManager;

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
}
