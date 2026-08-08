mod serial;
mod state;

use std::sync::Arc;

use serial::types::{LineEnding, PortInfo, SendFormat, SerialConfig, SerialStatus};
use state::AppState;
use tauri::Manager;

#[tauri::command]
fn list_serial_ports() -> Result<Vec<PortInfo>, String> {
    serial::SerialManager::list_ports().map_err(|error| error.to_string())
}

#[tauri::command]
async fn connect_serial(
    path: String,
    config: SerialConfig,
    state: tauri::State<'_, AppState>,
) -> Result<SerialStatus, String> {
    state
        .serial
        .connect(path, config)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn disconnect_serial(state: tauri::State<'_, AppState>) -> Result<SerialStatus, String> {
    Ok(state.serial.disconnect().await)
}

#[tauri::command]
async fn send_serial(
    input: String,
    format: SendFormat,
    line_ending: LineEnding,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    state
        .serial
        .send(input, format, line_ending)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn get_serial_status(state: tauri::State<'_, AppState>) -> Result<SerialStatus, String> {
    Ok(state.serial.status().await)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let app_handle = app.handle().clone();
            serial::hotplug::start_hotplug_watcher(app_handle.clone());
            app.manage(AppState {
                serial: Arc::new(serial::SerialManager::new(app_handle)),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_serial_ports,
            connect_serial,
            disconnect_serial,
            send_serial,
            get_serial_status,
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Serial Studio");
}
