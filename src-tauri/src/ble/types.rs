use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BleDeviceInfo {
    pub id: String,
    pub name: Option<String>,
    pub rssi: Option<i16>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BleServiceConfig {
    pub service_uuid: String,
    pub notify_characteristic_uuid: String,
    pub write_characteristic_uuid: String,
}
