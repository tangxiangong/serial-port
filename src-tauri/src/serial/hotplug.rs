use std::{collections::HashSet, time::Duration};
use tauri::{AppHandle, Emitter, async_runtime::spawn};
use tokio::{
    select,
    sync::watch::{Sender, channel},
    time::sleep,
};
use tokio_serial::available_ports;

/// Start a background task that polls `available_ports()` every 500 ms
/// and emits events when ports are added or removed.
///
/// Returns a `watch::Sender<bool>` — send `true` to stop the watcher.
pub fn start_hotplug_watcher(app_handle: AppHandle) -> Sender<bool> {
    let (cancel_tx, mut cancel_rx) = channel(false);

    spawn(async move {
        let mut known_ports: HashSet<String> = HashSet::new();

        // Initialize with current ports
        if let Ok(ports) = available_ports() {
            for port in &ports {
                known_ports.insert(port.port_name.clone());
            }
        }

        loop {
            select! {
                _ = cancel_rx.changed() => break,
                _ = sleep(Duration::from_millis(500)) => {
                    if let Ok(ports) = available_ports() {
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
