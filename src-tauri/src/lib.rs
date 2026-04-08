mod ble;
mod bridge;
mod checksum;
mod codec;
mod file_logger;
mod modbus;
mod record;
mod send;
mod serial;
mod state;
mod stats;
mod tray;

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use chrono::Utc;
use tauri::Manager;

use serde::{Deserialize, Serialize};

use ble::types::{BleDeviceInfo, BleServiceConfig};
use checksum::CheckAlgorithm;
use file_logger::LogFormat;
use record::{FormattedRecord, Record, RecordType};
use send::{LineEnding, SendFormat};
use serial::types::{PortInfo, SerialConfig};
use state::AppState;

/// Which physical transport to send data through.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DeviceType {
    Serial,
    Ble,
}

/// Format for the `validate_input` command.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ValidateFormat {
    Hex,
    Ascii,
    Dec,
}

// ── Serial Commands ──────────────────────────────────────────────

#[tauri::command]
async fn list_serial_ports(state: tauri::State<'_, AppState>) -> Result<Vec<PortInfo>, String> {
    Ok(serial::SerialManager::list_ports(&state.usb_json))
}

#[tauri::command]
async fn open_serial_port(
    port_id: String,
    path: String,
    config: SerialConfig,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let mgr = state.serial_manager.lock().await;
    mgr.open_port(port_id.clone(), path, config)
        .await
        .map_err(|e| e.to_string())?;
    // Create a RecordManager for this port
    let mut records = state.record_managers.lock().await;
    records
        .entry(port_id)
        .or_insert_with(record::RecordManager::new);
    Ok(())
}

#[tauri::command]
async fn close_serial_port(
    port_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let mgr = state.serial_manager.lock().await;
    mgr.close_port(&port_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn close_all_serial_ports(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let mgr = state.serial_manager.lock().await;
    mgr.close_all().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_open_ports(state: tauri::State<'_, AppState>) -> Result<Vec<String>, String> {
    let mgr = state.serial_manager.lock().await;
    Ok(mgr.get_open_ports().await)
}

// ── BLE Commands ─────────────────────────────────────────────────

#[tauri::command]
async fn ble_scan(
    service_uuid: String,
    duration: u64,
    state: tauri::State<'_, AppState>,
) -> Result<Vec<BleDeviceInfo>, String> {
    let mgr = state.ble_manager.lock().await;
    mgr.scan(&service_uuid, duration)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn ble_connect(
    device_id: String,
    service_config: BleServiceConfig,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let mgr = state.ble_manager.lock().await;
    mgr.connect(&device_id, service_config)
        .await
        .map_err(|e| e.to_string())?;
    // Create a RecordManager for BLE
    let mut records = state.record_managers.lock().await;
    records
        .entry("ble".to_string())
        .or_insert_with(record::RecordManager::new);
    Ok(())
}

#[tauri::command]
async fn ble_disconnect(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let mgr = state.ble_manager.lock().await;
    mgr.disconnect().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn ble_write(data: Vec<u8>, state: tauri::State<'_, AppState>) -> Result<(), String> {
    let mgr = state.ble_manager.lock().await;
    mgr.write(&data).await.map_err(|e| e.to_string())
}

// ── Send Commands ────────────────────────────────────────────────

#[tauri::command]
async fn send_data(
    port_id: String,
    input: String,
    format: SendFormat,
    line_ending: LineEnding,
    check_algorithm: CheckAlgorithm,
    device_type: DeviceType,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let buffer = send::build_send_buffer(&input, &format, &line_ending, &check_algorithm)
        .map_err(|e| e.to_string())?;

    let record_key = match device_type {
        DeviceType::Ble => {
            let mgr = state.ble_manager.lock().await;
            mgr.write(&buffer).await.map_err(|e| e.to_string())?;
            "ble".to_string()
        }
        DeviceType::Serial => {
            let mgr = state.serial_manager.lock().await;
            mgr.write_port(&port_id, &buffer)
                .await
                .map_err(|e| e.to_string())?;
            port_id
        }
    };

    let mut records = state.record_managers.lock().await;
    let mgr = records
        .entry(record_key)
        .or_insert_with(record::RecordManager::new);
    mgr.add_record(Record {
        record_type: RecordType::Write,
        data: buffer,
        timestamp: Utc::now(),
    });

    Ok(())
}

#[tauri::command]
fn format_hex_input_cmd(input: String) -> String {
    codec::format_hex_input(&input)
}

#[tauri::command]
fn validate_input(input: String, format: ValidateFormat) -> bool {
    match format {
        ValidateFormat::Hex => codec::validate_hex_input(&input),
        ValidateFormat::Ascii | ValidateFormat::Dec => true,
    }
}

// ── Record Commands ──────────────────────────────────────────────

#[tauri::command]
async fn get_records(
    port_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<Vec<FormattedRecord>, String> {
    let records = state.record_managers.lock().await;
    match records.get(&port_id) {
        Some(mgr) => Ok(mgr.get_records()),
        None => Ok(vec![]),
    }
}

#[tauri::command]
async fn clear_records(
    port_id: String,
    record_type: Option<RecordType>,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let mut records = state.record_managers.lock().await;
    if let Some(mgr) = records.get_mut(&port_id) {
        match record_type {
            Some(rt) => mgr.clear_records_by_type(&rt),
            None => mgr.clear_records(),
        }
    }
    Ok(())
}

#[tauri::command]
async fn export_records(
    port_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    let records = state.record_managers.lock().await;
    match records.get(&port_id) {
        Some(mgr) => Ok(mgr.export_records()),
        None => Ok("[]".to_string()),
    }
}

#[tauri::command]
async fn get_record_counts(
    port_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let records = state.record_managers.lock().await;
    match records.get(&port_id) {
        Some(mgr) => Ok(serde_json::json!({ "rx": mgr.rx_count(), "tx": mgr.tx_count() })),
        None => Ok(serde_json::json!({ "rx": 0, "tx": 0 })),
    }
}

// ── Modbus Commands ─────────────────────────────────────────────

#[tauri::command]
fn parse_modbus_rtu(data: Vec<u8>) -> Result<modbus::ModbusFrame, String> {
    modbus::parse_rtu_frame(&data).map_err(|e| e.to_string())
}

#[tauri::command]
fn parse_modbus_ascii(data: Vec<u8>) -> Result<modbus::ModbusFrame, String> {
    modbus::parse_ascii_frame(&data).map_err(|e| e.to_string())
}

// ── Stats Command ───────────────────────────────────────────────

#[tauri::command]
async fn get_current_stats(
    port_id: String,
    _state: tauri::State<'_, AppState>,
) -> Result<stats::StatsSnapshot, String> {
    Ok(stats::StatsSnapshot {
        port_id,
        rx_bytes_per_sec: 0,
        tx_bytes_per_sec: 0,
        rx_frames_per_sec: 0,
        tx_frames_per_sec: 0,
        error_count: 0,
    })
}

// ── File Logger Commands ────────────────────────────────────────

#[tauri::command]
async fn start_file_logging(
    port_id: String,
    path: String,
    format: LogFormat,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let logger = file_logger::FileLogger::new(std::path::PathBuf::from(path), format)
        .await
        .map_err(|e| e.to_string())?;
    let mut loggers = state.file_loggers.lock().await;
    loggers.insert(port_id, logger);
    Ok(())
}

#[tauri::command]
async fn stop_file_logging(
    port_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let mut loggers = state.file_loggers.lock().await;
    if let Some(logger) = loggers.remove(&port_id) {
        logger.close().await.map_err(|e| e.to_string())?;
    }
    Ok(())
}

// ── Bridge Commands ─────────────────────────────────────────────

#[tauri::command]
async fn start_bridge(
    port_a_id: String,
    port_b_id: String,
    state: tauri::State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let mgr = state.serial_manager.lock().await;
    let stream_a = mgr
        .get_stream(&port_a_id)
        .await
        .map_err(|e| e.to_string())?;
    let stream_b = mgr
        .get_stream(&port_b_id)
        .await
        .map_err(|e| e.to_string())?;
    drop(mgr);

    let bridge = bridge::Bridge::start(stream_a, stream_b, app)
        .await
        .map_err(|e| e.to_string())?;

    let mut active = state.active_bridge.lock().await;
    if let Some(old) = active.take() {
        old.stop();
    }
    *active = Some(bridge);
    Ok(())
}

#[tauri::command]
async fn stop_bridge(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let mut active = state.active_bridge.lock().await;
    if let Some(bridge) = active.take() {
        bridge.stop();
    }
    Ok(())
}

// ── App Setup ────────────────────────────────────────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .plugin(tauri_plugin_log::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // Load usb-device.json from bundled resources
            let resource_path = app
                .path()
                .resolve(
                    "resources/usb-device.json",
                    tauri::path::BaseDirectory::Resource,
                )
                .expect("failed to resolve usb-device.json");
            let usb_json_str =
                std::fs::read_to_string(&resource_path).unwrap_or_else(|_| "{}".to_string());
            let usb_json: serde_json::Value =
                serde_json::from_str(&usb_json_str).unwrap_or(serde_json::json!({}));

            let app_handle = app.handle().clone();

            // Start hotplug watcher
            serial::hotplug::start_hotplug_watcher(app_handle.clone());

            // Create managers - BleManager::new() is async, use block_on
            let ble_manager = tauri::async_runtime::block_on(async {
                ble::BleManager::new(app_handle.clone()).await
            });

            let state = AppState {
                serial_manager: Arc::new(Mutex::new(serial::SerialManager::new(app_handle))),
                ble_manager: Arc::new(Mutex::new(ble_manager)),
                record_managers: Arc::new(Mutex::new(HashMap::new())),
                usb_json,
                file_loggers: Arc::new(Mutex::new(HashMap::new())),
                active_bridge: Arc::new(Mutex::new(None)),
            };

            app.manage(state);

            // Set up system tray
            let tray_handle = app.handle().clone();
            tray::setup_tray(&tray_handle).expect("failed to setup system tray");

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_serial_ports,
            open_serial_port,
            close_serial_port,
            close_all_serial_ports,
            get_open_ports,
            ble_scan,
            ble_connect,
            ble_disconnect,
            ble_write,
            send_data,
            format_hex_input_cmd,
            validate_input,
            get_records,
            clear_records,
            export_records,
            get_record_counts,
            parse_modbus_rtu,
            parse_modbus_ascii,
            get_current_stats,
            start_file_logging,
            stop_file_logging,
            start_bridge,
            stop_bridge,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
