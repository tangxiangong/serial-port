use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::core::checksum::ChecksumKind;
use crate::core::records::{
    DeviceInfo, DisplayMode, QuickInputItem, RecordEntry, RecordKind, SessionMeta,
};
use crate::core::search::SearchResult;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeviceType {
    Serial,
    Ble,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerialConfig {
    pub baud_rate: u32,
    pub baud_rate_list: Vec<u32>,
    pub data_bits: u8,
    pub stop_bits: f32,
    pub parity: String,
    pub flow_control: String,
}

impl Default for SerialConfig {
    fn default() -> Self {
        Self {
            baud_rate: 9_600,
            baud_rate_list: vec![9_600, 19_200, 38_400, 57_600, 115_200],
            data_bits: 8,
            stop_bits: 1.0,
            parity: "none".into(),
            flow_control: "none".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BleProfile {
    pub name: String,
    pub description: String,
    pub service: u32,
    pub write_characteristic: u32,
    pub notify_characteristic: u32,
}

impl BleProfile {
    pub fn builtin() -> Vec<Self> {
        vec![
            Self {
                name: "通用Ⅰ型".into(),
                description: "适用于DX-BT24等常见蓝牙串口透传模块".into(),
                service: 0xFFE0,
                write_characteristic: 0xFFE1,
                notify_characteristic: 0xFFE1,
            },
            Self {
                name: "通用Ⅱ型".into(),
                description: "适用于DX-BT16等蓝牙串口透传模块".into(),
                service: 0xFFE0,
                write_characteristic: 0xFFE2,
                notify_characteristic: 0xFFE1,
            },
        ]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutPrefs {
    pub show_full_screen: bool,
    pub show_setting_panel: bool,
    pub show_send_panel: bool,
    pub show_quick_input_panel: bool,
}

impl Default for LayoutPrefs {
    fn default() -> Self {
        Self {
            show_full_screen: true,
            show_setting_panel: true,
            show_send_panel: true,
            show_quick_input_panel: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub device_type: DeviceType,
    pub line_ending_mode: String,
    pub custom_line_ending: String,
    pub has_dec_types: bool,
    pub record_cache_enabled: bool,
    pub send_display: DisplayMode,
    pub read_display: DisplayMode,
    pub serial: SerialConfig,
    pub ble_profiles: Vec<BleProfile>,
    pub selected_ble_profile: String,
    pub layout: LayoutPrefs,
    pub auto_send_interval_ms: u64,
    pub quick_loop_interval_ms: u64,
    pub selected_checksum: Option<ChecksumKind>,
}

impl Default for AppSettings {
    fn default() -> Self {
        let ble_profiles = BleProfile::builtin();
        let selected_ble_profile = ble_profiles
            .first()
            .map(|item| item.name.clone())
            .unwrap_or_default();
        Self {
            device_type: DeviceType::Serial,
            line_ending_mode: "无".into(),
            custom_line_ending: String::new(),
            has_dec_types: false,
            record_cache_enabled: true,
            send_display: DisplayMode::Hex,
            read_display: DisplayMode::Hex,
            serial: SerialConfig::default(),
            ble_profiles,
            selected_ble_profile,
            layout: LayoutPrefs::default(),
            auto_send_interval_ms: 1_000,
            quick_loop_interval_ms: 5_000,
            selected_checksum: None,
        }
    }
}

impl AppSettings {
    pub fn line_ending(&self) -> String {
        match self.line_ending_mode.as_str() {
            "LF" => "\n".into(),
            "CR" => "\r".into(),
            "CRLF" => "\r\n".into(),
            "自定义" => self.custom_line_ending.clone(),
            _ => String::new(),
        }
    }

    pub fn active_ble_profile(&self) -> BleProfile {
        self.ble_profiles
            .iter()
            .find(|profile| profile.name == self.selected_ble_profile)
            .cloned()
            .unwrap_or_else(|| {
                self.ble_profiles.first().cloned().unwrap_or(BleProfile {
                    name: "默认".into(),
                    description: String::new(),
                    service: 0,
                    write_characteristic: 0,
                    notify_characteristic: 0,
                })
            })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SerialPortDescriptor {
    pub port_name: String,
    pub display_name: String,
    pub kind: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BleDeviceDescriptor {
    pub id: String,
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionStatus {
    Idle,
    Connecting,
    Connected,
    Disconnecting,
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub bootstrapped: bool,
    pub settings: AppSettings,
    pub quick_inputs: Vec<QuickInputItem>,
    pub records: Vec<RecordEntry>,
    pub live_reading: Option<RecordEntry>,
    pub sessions: Vec<SessionMeta>,
    pub selected_session_id: Option<String>,
    pub selected_session_records: Vec<RecordEntry>,
    pub search_results: Vec<SearchResult>,
    pub serial_ports: Vec<SerialPortDescriptor>,
    pub selected_serial_port: Option<String>,
    pub ble_devices: Vec<BleDeviceDescriptor>,
    pub selected_ble_device: Option<String>,
    pub connection_status: ConnectionStatus,
    pub active_device: Option<DeviceInfo>,
    pub latest_notice: String,
    pub error_notice: Option<String>,
    pub is_auto_sending: bool,
    pub is_quick_loop_running: bool,
    pub history_open: bool,
    pub settings_open: bool,
    pub search_open: bool,
    pub rx_count: usize,
    pub tx_count: usize,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            bootstrapped: false,
            settings: AppSettings::default(),
            quick_inputs: QuickInputItem::default_items(),
            records: Vec::new(),
            live_reading: None,
            sessions: Vec::new(),
            selected_session_id: None,
            selected_session_records: Vec::new(),
            search_results: Vec::new(),
            serial_ports: Vec::new(),
            selected_serial_port: None,
            ble_devices: Vec::new(),
            selected_ble_device: None,
            connection_status: ConnectionStatus::Idle,
            active_device: None,
            latest_notice: "等待连接".into(),
            error_notice: None,
            is_auto_sending: false,
            is_quick_loop_running: false,
            history_open: false,
            settings_open: false,
            search_open: false,
            rx_count: 0,
            tx_count: 0,
        }
    }
}

impl AppState {
    pub fn apply_record(&mut self, record: RecordEntry) {
        match record.kind {
            RecordKind::Read => {
                self.rx_count += record.data.len();
            }
            RecordKind::Write => {
                self.tx_count += record.data.len();
            }
            RecordKind::System => {}
        }
        self.records.push(record);
    }

    pub fn begin_live_read(&mut self, display: DisplayMode, chunk: Vec<u8>) {
        if let Some(live) = &mut self.live_reading {
            live.data.extend_from_slice(&chunk);
        } else {
            self.live_reading = Some(RecordEntry {
                kind: RecordKind::Read,
                data: chunk,
                time: Utc::now(),
                display,
            });
        }
    }

    pub fn finalize_live_read(&mut self) -> Option<RecordEntry> {
        self.live_reading.take()
    }

    pub fn current_session_label(&self) -> String {
        self.active_device
            .as_ref()
            .map(|device| format!("{} · {}", device.kind.label(), device.name))
            .unwrap_or_else(|| "未连接设备".into())
    }
}

#[derive(Debug, Clone)]
pub struct BootstrapData {
    pub settings: AppSettings,
    pub quick_inputs: Vec<QuickInputItem>,
    pub sessions: Vec<SessionMeta>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExportRecord {
    #[serde(rename = "type")]
    pub kind: String,
    pub data: String,
    pub timestamp: i64,
    pub time: String,
    pub display: String,
}

impl ExportRecord {
    pub fn from_record(record: &RecordEntry, rendered: String) -> Self {
        Self {
            kind: record.kind.label().to_lowercase(),
            data: rendered,
            timestamp: record.time.timestamp_millis(),
            time: record.time.format("%Y-%m-%d %H:%M:%S%.3f").to_string(),
            display: record.display.label().to_lowercase(),
        }
    }
}
