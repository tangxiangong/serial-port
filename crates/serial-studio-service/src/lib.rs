pub use serial_studio_core::*;
use std::sync::{Mutex, OnceLock};
use tokio::runtime::{Builder, Runtime};

#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    #[error("{message}")]
    Operation { message: String },
}

impl From<anyhow::Error> for CoreError {
    fn from(error: anyhow::Error) -> Self {
        Self::Operation {
            message: error.to_string(),
        }
    }
}

struct Service {
    runtime: Runtime,
    serial: SerialManager,
    calls: Mutex<()>,
}
static SERVICE: OnceLock<Result<Service, String>> = OnceLock::new();

fn service() -> Result<&'static Service, CoreError> {
    SERVICE
        .get_or_init(|| {
            Builder::new_multi_thread()
                .worker_threads(2)
                .enable_all()
                .build()
                .map(|runtime| Service {
                    runtime,
                    serial: SerialManager::new(),
                    calls: Mutex::new(()),
                })
                .map_err(|error| error.to_string())
        })
        .as_ref()
        .map_err(|message| CoreError::Operation {
            message: message.clone(),
        })
}

fn perform<T>(operation: impl FnOnce(&Service) -> anyhow::Result<T>) -> Result<T, CoreError> {
    let service = service()?;
    let _guard = service.calls.lock().map_err(|_| CoreError::Operation {
        message: "核心状态异常，请重新启动应用".into(),
    })?;
    operation(service).map_err(Into::into)
}

pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_owned()
}
pub fn ports() -> Result<Vec<PortInfo>, CoreError> {
    SerialManager::list_ports().map_err(Into::into)
}
pub fn poll() -> Result<PollBatch, CoreError> {
    perform(|s| Ok(s.runtime.block_on(s.serial.poll())))
}
pub fn connect(path: String, config: SerialConfig) -> Result<SerialStatus, CoreError> {
    perform(|s| s.runtime.block_on(s.serial.connect(path, config)))
}
pub fn disconnect() -> Result<SerialStatus, CoreError> {
    perform(|s| Ok(s.runtime.block_on(s.serial.disconnect())))
}
pub fn send(input: String, format: SendFormat, line_ending: LineEnding) -> Result<(), CoreError> {
    perform(|s| {
        s.runtime
            .block_on(s.serial.send(input, format, line_ending))
    })
}
