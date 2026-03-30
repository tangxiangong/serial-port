use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeviceKind {
    Serial,
    Bluetooth,
}

impl DeviceKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Serial => "串口",
            Self::Bluetooth => "蓝牙",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecordKind {
    Read,
    Write,
    System,
}

impl RecordKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Read => "接收",
            Self::Write => "发送",
            Self::System => "系统",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DisplayMode {
    Hex,
    Ascii,
    Dec,
}

impl DisplayMode {
    pub fn label(self) -> &'static str {
        match self {
            Self::Hex => "HEX",
            Self::Ascii => "ASCII",
            Self::Dec => "DEC",
        }
    }

    pub fn next(self, has_dec_types: bool) -> Self {
        match (self, has_dec_types) {
            (Self::Hex, _) => Self::Ascii,
            (Self::Ascii, true) => Self::Dec,
            (Self::Ascii, false) => Self::Hex,
            (Self::Dec, _) => Self::Hex,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeviceInfo {
    pub kind: DeviceKind,
    pub id: String,
    pub name: String,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RecordEntry {
    pub kind: RecordKind,
    pub data: Vec<u8>,
    pub time: DateTime<Utc>,
    pub display: DisplayMode,
}

impl RecordEntry {
    pub fn new(kind: RecordKind, data: Vec<u8>, display: DisplayMode) -> Self {
        Self {
            kind,
            data,
            time: Utc::now(),
            display,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionMeta {
    pub id: String,
    pub title: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub record_count: usize,
    pub device: DeviceInfo,
}

impl SessionMeta {
    pub fn new(device: DeviceInfo) -> Self {
        let now = Utc::now();
        Self {
            id: format!("session-{}", now.timestamp_millis()),
            title: now.format("%Y-%m-%d %H:%M:%S").to_string(),
            created_at: now,
            updated_at: now,
            record_count: 0,
            device,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct QuickInputItem {
    pub id: String,
    pub enabled: bool,
    pub content: String,
    pub remark: String,
    pub delay_ms: u64,
    pub is_hex: bool,
}

impl QuickInputItem {
    pub fn default_items() -> Vec<Self> {
        vec![
            Self {
                id: "quick-at-rst".into(),
                enabled: true,
                content: "AT+RST".into(),
                remark: "重启模块".into(),
                delay_ms: 1_000,
                is_hex: false,
            },
            Self {
                id: "quick-at-gmr".into(),
                enabled: true,
                content: "AT+GMR".into(),
                remark: "查询版本信息".into(),
                delay_ms: 1_000,
                is_hex: false,
            },
            Self {
                id: "quick-hex".into(),
                enabled: false,
                content: "01 02 03 04".into(),
                remark: "HEX测试数据".into(),
                delay_ms: 1_000,
                is_hex: true,
            },
        ]
    }
}
