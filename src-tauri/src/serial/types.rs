use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Parity {
    #[default]
    None,
    Odd,
    Even,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FlowControl {
    #[default]
    None,
    Software,
    Hardware,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SerialConfig {
    pub baud_rate: u32,
    pub data_bits: u8,
    pub stop_bits: u8,
    pub parity: Parity,
    pub flow_control: FlowControl,
}

impl Default for SerialConfig {
    fn default() -> Self {
        SerialConfig {
            baud_rate: 9600,
            data_bits: 8,
            stop_bits: 1,
            parity: Parity::default(),
            flow_control: FlowControl::default(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PortInfo {
    pub path: String,
    pub vid: Option<String>,
    pub pid: Option<String>,
    pub device_name: Option<String>,
}
