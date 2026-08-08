use std::{collections::BTreeSet, time::Duration};

use tauri::{AppHandle, Emitter};
use tokio::{select, sync::watch, time::sleep};

pub fn start_hotplug_watcher(app_handle: AppHandle) -> watch::Sender<bool> {
    let (cancel, mut cancelled) = watch::channel(false);

    tauri::async_runtime::spawn(async move {
        let mut known = current_paths();
        loop {
            select! {
                changed = cancelled.changed() => {
                    if changed.is_err() || *cancelled.borrow() {
                        break;
                    }
                }
                _ = sleep(Duration::from_millis(750)) => {
                    let current = current_paths();
                    if current != known {
                        known = current;
                        let _ = app_handle.emit("serial:ports-changed", ());
                    }
                }
            }
        }
    });

    cancel
}

fn current_paths() -> BTreeSet<String> {
    tokio_serial::available_ports()
        .unwrap_or_default()
        .into_iter()
        .map(|port| port.port_name)
        .collect()
}
