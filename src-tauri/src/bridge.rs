use anyhow::Result;
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    sync::{
        Mutex,
        watch::{Receiver, Sender, channel},
    },
};
use tokio_serial::SerialStream;

/// State for an active bridge between two serial ports.
pub struct Bridge {
    cancel_token: Sender<bool>,
}

impl Bridge {
    /// Start bridging: read from port_a's stream, write to port_b, and vice versa.
    /// Also emit events for monitoring.
    pub async fn start(
        port_a_stream: Arc<Mutex<SerialStream>>,
        port_b_stream: Arc<Mutex<SerialStream>>,
        app_handle: AppHandle,
    ) -> Result<Self> {
        let (cancel_tx, cancel_rx) = channel(false);

        // Spawn A -> B forwarding task
        let cancel_rx_a = cancel_rx.clone();
        let stream_a = Arc::clone(&port_a_stream);
        let stream_b_write = Arc::clone(&port_b_stream);
        let handle_a = app_handle.clone();

        tokio::spawn(async move {
            Self::forward_loop(stream_a, stream_b_write, "a_to_b", handle_a, cancel_rx_a).await;
        });

        // Spawn B -> A forwarding task
        let cancel_rx_b = cancel_rx;
        let stream_b = port_b_stream;
        let stream_a_write = port_a_stream;
        let handle_b = app_handle;

        tokio::spawn(async move {
            Self::forward_loop(stream_b, stream_a_write, "b_to_a", handle_b, cancel_rx_b).await;
        });

        Ok(Self {
            cancel_token: cancel_tx,
        })
    }

    async fn forward_loop(
        read_stream: Arc<Mutex<SerialStream>>,
        write_stream: Arc<Mutex<SerialStream>>,
        direction: &str,
        app_handle: AppHandle,
        mut cancel_rx: Receiver<bool>,
    ) {
        let mut buf = [0u8; 1024];
        loop {
            tokio::select! {
                _ = cancel_rx.changed() => break,
                result = async { read_stream.lock().await.read(&mut buf).await } => {
                    match result {
                        Ok(0) | Err(_) => break,
                        Ok(n) => {
                            let data = &buf[..n];
                            // Forward to other port
                            let _ = write_stream.lock().await.write_all(data).await;
                            // Emit for monitoring
                            let _ = app_handle.emit("bridge:data", serde_json::json!({
                                "direction": direction,
                                "data": crate::codec::bytes_to_hex_spaced(data),
                                "timestamp": chrono::Utc::now().to_rfc3339(),
                            }));
                        }
                    }
                }
            }
        }
    }

    pub fn stop(self) {
        let _ = self.cancel_token.send(true);
    }
}
