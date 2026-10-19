//! Official UniFFI generation, matching BibCiTeX's remote-record bridge.
mod records;
pub use records::*;
uniffi::setup_scaffolding!();

#[uniffi::export]
pub fn version() -> String {
    serial_studio_service::version()
}
#[uniffi::export]
pub fn ports() -> Result<Vec<PortInfo>, CoreError> {
    serial_studio_service::ports()
}
#[uniffi::export]
pub fn poll() -> Result<PollBatch, CoreError> {
    serial_studio_service::poll()
}
#[uniffi::export]
pub fn connect(path: String, config: SerialConfig) -> Result<SerialStatus, CoreError> {
    serial_studio_service::connect(path, config)
}
#[uniffi::export]
pub fn disconnect() -> Result<SerialStatus, CoreError> {
    serial_studio_service::disconnect()
}
#[uniffi::export]
pub fn send(input: String, format: SendFormat, line_ending: LineEnding) -> Result<(), CoreError> {
    serial_studio_service::send(input, format, line_ending)
}
