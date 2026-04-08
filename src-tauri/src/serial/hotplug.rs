use std::collections::HashSet;
use tauri::{AppHandle, Emitter};

/// Start a background task that polls `available_ports()` every 500 ms
/// and emits events when ports are added or removed.
///
/// Returns a `watch::Sender<bool>` — send `true` to stop the watcher.
pub fn start_hotplug_watcher(app_handle: AppHandle) -> tokio::sync::watch::Sender<bool> {
    let (cancel_tx, mut cancel_rx) = tokio::sync::watch::channel(false);

    tauri::async_runtime::spawn(async move {
        let mut known_ports: HashSet<String> = HashSet::new();

        // Initialize with current ports
        if let Ok(ports) = tokio_serial::available_ports() {
            for port in &ports {
                known_ports.insert(port.port_name.clone());
            }
        }

        loop {
            tokio::select! {
                _ = cancel_rx.changed() => break,
                _ = tokio::time::sleep(std::time::Duration::from_millis(500)) => {
                    if let Ok(ports) = tokio_serial::available_ports() {
                        let current: HashSet<String> =
                            ports.iter().map(|p| p.port_name.clone()).collect();

                        // Newly appeared ports
                        for path in current.difference(&known_ports) {
                            let _ = app_handle.emit(
                                "serial:port_added",
                                serde_json::json!({ "path": path }),
                            );
                        }

                        // Disappeared ports
                        for path in known_ports.difference(&current) {
                            let _ = app_handle.emit(
                                "serial:port_removed",
                                serde_json::json!({ "path": path }),
                            );
                        }

                        known_ports = current;
                    }
                }
            }
        }
    });

    cancel_tx
}
