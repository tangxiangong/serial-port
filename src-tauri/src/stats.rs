use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use serde::Serialize;
use tauri::Emitter;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StatsSnapshot {
    pub port_id: String,
    pub rx_bytes_per_sec: u64,
    pub tx_bytes_per_sec: u64,
    pub rx_frames_per_sec: u64,
    pub tx_frames_per_sec: u64,
    pub error_count: u64,
}

pub struct StatsCollector {
    pub port_id: String,
    rx_bytes: Arc<AtomicU64>,
    tx_bytes: Arc<AtomicU64>,
    rx_frames: Arc<AtomicU64>,
    tx_frames: Arc<AtomicU64>,
    error_count: Arc<AtomicU64>,
    cancel_token: tokio::sync::watch::Sender<bool>,
}

impl StatsCollector {
    pub fn new(port_id: String, app_handle: tauri::AppHandle) -> Self {
        let rx_bytes = Arc::new(AtomicU64::new(0));
        let tx_bytes = Arc::new(AtomicU64::new(0));
        let rx_frames = Arc::new(AtomicU64::new(0));
        let tx_frames = Arc::new(AtomicU64::new(0));
        let error_count = Arc::new(AtomicU64::new(0));

        let (cancel_tx, mut cancel_rx) = tokio::sync::watch::channel(false);

        // Spawn sampling task — every 1 second, snapshot and reset counters, emit event
        let pid = port_id.clone();
        let rx_b = Arc::clone(&rx_bytes);
        let tx_b = Arc::clone(&tx_bytes);
        let rx_f = Arc::clone(&rx_frames);
        let tx_f = Arc::clone(&tx_frames);
        let err = Arc::clone(&error_count);

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = cancel_rx.changed() => break,
                    _ = tokio::time::sleep(std::time::Duration::from_secs(1)) => {
                        let snapshot = StatsSnapshot {
                            port_id: pid.clone(),
                            rx_bytes_per_sec: rx_b.swap(0, Ordering::Relaxed),
                            tx_bytes_per_sec: tx_b.swap(0, Ordering::Relaxed),
                            rx_frames_per_sec: rx_f.swap(0, Ordering::Relaxed),
                            tx_frames_per_sec: tx_f.swap(0, Ordering::Relaxed),
                            error_count: err.load(Ordering::Relaxed),
                        };
                        let _ = app_handle.emit("serial:stats", &snapshot);
                    }
                }
            }
        });

        Self {
            port_id,
            rx_bytes,
            tx_bytes,
            rx_frames,
            tx_frames,
            error_count,
            cancel_token: cancel_tx,
        }
    }

    pub fn record_rx(&self, bytes: u64) {
        self.rx_bytes.fetch_add(bytes, Ordering::Relaxed);
        self.rx_frames.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_tx(&self, bytes: u64) {
        self.tx_bytes.fetch_add(bytes, Ordering::Relaxed);
        self.tx_frames.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_error(&self) {
        self.error_count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn snapshot(&self) -> StatsSnapshot {
        StatsSnapshot {
            port_id: self.port_id.clone(),
            rx_bytes_per_sec: self.rx_bytes.load(Ordering::Relaxed),
            tx_bytes_per_sec: self.tx_bytes.load(Ordering::Relaxed),
            rx_frames_per_sec: self.rx_frames.load(Ordering::Relaxed),
            tx_frames_per_sec: self.tx_frames.load(Ordering::Relaxed),
            error_count: self.error_count.load(Ordering::Relaxed),
        }
    }

    pub fn stop(self) {
        let _ = self.cancel_token.send(true);
    }
}
