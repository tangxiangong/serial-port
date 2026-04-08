pub mod types;

use crate::codec;
use anyhow::{Result, anyhow};
use btleplug::api::{
    Central, CharPropFlags, Characteristic, Manager as _, Peripheral as _, ScanFilter, WriteType,
};
use btleplug::platform::{Adapter, Manager, Peripheral};
use futures::StreamExt;
use serde::Serialize;
use std::{sync::Arc, time::Duration};
use tauri::{AppHandle, Emitter};
use tokio::sync::{Mutex, watch::Sender};
use types::{BleDeviceInfo, BleServiceConfig};
use uuid::Uuid;

pub struct BleManager {
    app_handle: AppHandle,
    adapter: Option<Adapter>,
    connected_peripheral: Arc<Mutex<Option<Peripheral>>>,
    write_characteristic: Arc<Mutex<Option<Characteristic>>>,
    cancel_token: Arc<Mutex<Option<Sender<bool>>>>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct BleFrameEvent {
    raw: Vec<u8>,
    hex: String,
    ascii: String,
    dec: String,
    timestamp: String,
}

impl BleManager {
    pub async fn new(app_handle: AppHandle) -> Self {
        let adapter = async {
            let manager = Manager::new().await.ok()?;
            let adapters = manager.adapters().await.ok()?;
            adapters.into_iter().next()
        }
        .await;

        BleManager {
            app_handle,
            adapter,
            connected_peripheral: Arc::new(Mutex::new(None)),
            write_characteristic: Arc::new(Mutex::new(None)),
            cancel_token: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn scan(&self, service_uuid: &str, duration_secs: u64) -> Result<Vec<BleDeviceInfo>> {
        let adapter = self
            .adapter
            .as_ref()
            .ok_or_else(|| anyhow!("No Bluetooth adapter available"))?;

        let uuid = Uuid::parse_str(service_uuid)
            .map_err(|e| anyhow!("Invalid service UUID '{}': {}", service_uuid, e))?;

        let filter = ScanFilter {
            services: vec![uuid],
        };

        adapter.start_scan(filter).await?;
        tokio::time::sleep(Duration::from_secs(duration_secs)).await;
        adapter.stop_scan().await?;

        let peripherals = adapter.peripherals().await?;
        let mut devices = Vec::new();

        for peripheral in peripherals {
            let id = peripheral.id().to_string();
            let props = peripheral.properties().await.ok().flatten();
            let name = props.as_ref().and_then(|p| p.local_name.clone());
            let rssi = props.as_ref().and_then(|p| p.rssi);
            devices.push(BleDeviceInfo { id, name, rssi });
        }

        Ok(devices)
    }

    pub async fn connect(&self, device_id: &str, config: BleServiceConfig) -> Result<()> {
        let adapter = self
            .adapter
            .as_ref()
            .ok_or_else(|| anyhow!("No Bluetooth adapter available"))?;

        // Find peripheral by matching its id string representation
        let peripherals = adapter.peripherals().await?;
        let peripheral = peripherals
            .into_iter()
            .find(|p| p.id().to_string() == device_id)
            .ok_or_else(|| anyhow!("Peripheral '{}' not found", device_id))?;

        peripheral.connect().await?;
        peripheral.discover_services().await?;

        let service_uuid = Uuid::parse_str(&config.service_uuid)
            .map_err(|e| anyhow!("Invalid service UUID: {}", e))?;
        let notify_uuid = Uuid::parse_str(&config.notify_characteristic_uuid)
            .map_err(|e| anyhow!("Invalid notify characteristic UUID: {}", e))?;
        let write_uuid = Uuid::parse_str(&config.write_characteristic_uuid)
            .map_err(|e| anyhow!("Invalid write characteristic UUID: {}", e))?;

        let chars = peripheral.characteristics();

        let notify_char = chars
            .iter()
            .find(|c| {
                c.uuid == notify_uuid
                    && c.service_uuid == service_uuid
                    && c.properties.contains(CharPropFlags::NOTIFY)
            })
            .cloned()
            .ok_or_else(|| anyhow!("Notify characteristic {} not found", notify_uuid))?;

        let write_char = chars
            .iter()
            .find(|c| {
                c.uuid == write_uuid
                    && c.service_uuid == service_uuid
                    && c.properties.contains(CharPropFlags::WRITE_WITHOUT_RESPONSE)
            })
            .cloned()
            .ok_or_else(|| anyhow!("Write characteristic {} not found", write_uuid))?;

        peripheral.subscribe(&notify_char).await?;

        // Store write characteristic and peripheral
        *self.write_characteristic.lock().await = Some(write_char);
        *self.connected_peripheral.lock().await = Some(peripheral.clone());

        // Set up cancel channel
        let (cancel_tx, mut cancel_rx) = tokio::sync::watch::channel(false);
        *self.cancel_token.lock().await = Some(cancel_tx);

        // Spawn notification listener task
        let app_handle = self.app_handle.clone();
        tokio::spawn(async move {
            let stream = match peripheral.notifications().await {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("BLE: failed to get notifications stream: {e}");
                    return;
                }
            };
            tokio::pin!(stream);

            loop {
                tokio::select! {
                    Some(notification) = stream.next() => {
                        let data = &notification.value;
                        let now = chrono::Local::now()
                            .format("%Y-%m-%d %H:%M:%S%.3f")
                            .to_string();

                        let event = BleFrameEvent {
                            raw: data.clone(),
                            hex: codec::bytes_to_hex_spaced(data),
                            ascii: codec::bytes_to_ascii_string(data),
                            dec: codec::bytes_to_dec_string(data),
                            timestamp: now,
                        };

                        if let Err(e) = app_handle.emit("ble:frame", &event) {
                            eprintln!("BLE: failed to emit ble:frame event: {e}");
                        }
                    }
                    Ok(_) = cancel_rx.changed() => {
                        if *cancel_rx.borrow() {
                            break;
                        }
                    }
                    else => break,
                }
            }
        });

        Ok(())
    }

    pub async fn write(&self, data: &[u8]) -> Result<()> {
        let peripheral_guard = self.connected_peripheral.lock().await;
        let peripheral = peripheral_guard
            .as_ref()
            .ok_or_else(|| anyhow!("Not connected to any BLE peripheral"))?;

        let char_guard = self.write_characteristic.lock().await;
        let write_char = char_guard
            .as_ref()
            .ok_or_else(|| anyhow!("Write characteristic not available"))?;

        peripheral
            .write(write_char, data, WriteType::WithoutResponse)
            .await?;

        Ok(())
    }

    pub async fn disconnect(&self) -> Result<()> {
        // Signal the notification task to stop
        if let Some(tx) = self.cancel_token.lock().await.take() {
            let _ = tx.send(true);
        }

        // Disconnect the peripheral
        if let Some(peripheral) = self.connected_peripheral.lock().await.take() {
            peripheral.disconnect().await?;
        }

        // Clear write characteristic
        *self.write_characteristic.lock().await = None;

        Ok(())
    }
}
