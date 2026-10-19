//! Interoptopus owns the ABI, wire encoding and managed memory wrappers.
//! Prefix exports so they cannot collide with imported Windows/Winsock symbols.
use interoptopus::{
    builtins_wire, ffi, function,
    inventory::RustInventory,
    lang::types::{TypeInfo, WireIO},
    pattern::result::panic_to_result,
    wire::Wire,
};
use serial_studio_service::{
    LineEnding, PollBatch, PortInfo, SendFormat, SerialConfig, SerialStatus,
};

#[ffi]
pub enum BridgeError {
    Serialization,
}
#[ffi]
pub struct ConnectRequest {
    pub path: String,
    pub config: SerialConfig,
}
#[ffi]
pub struct SendRequest {
    pub text: String,
    pub format: SendFormat,
    pub line_ending: LineEnding,
}
#[ffi]
pub struct StatusResponse {
    pub error: Option<String>,
}
#[ffi]
pub struct PortsResponse {
    pub payload: Vec<PortInfo>,
    pub error: Option<String>,
}
#[ffi]
pub struct PollResponse {
    pub payload: Option<PollBatch>,
    pub error: Option<String>,
}
#[ffi]
pub struct ConnectionResponse {
    pub payload: Option<SerialStatus>,
    pub error: Option<String>,
}

fn invoke<T: TypeInfo + WireIO>(work: impl FnOnce() -> T) -> ffi::Result<Wire<T>, BridgeError> {
    panic_to_result(|| {
        Wire::try_from(work())
            .map_err(|_| BridgeError::Serialization)
            .into()
    })
}
fn request<T: TypeInfo + WireIO>(mut wire: Wire<T>) -> Result<T, BridgeError> {
    wire.try_unwire().map_err(|_| BridgeError::Serialization)
}

#[ffi]
pub fn serial_studio_ports() -> ffi::Result<Wire<PortsResponse>, BridgeError> {
    invoke(|| match serial_studio_service::ports() {
        Ok(payload) => PortsResponse {
            payload,
            error: None,
        },
        Err(error) => PortsResponse {
            payload: Vec::new(),
            error: Some(error.to_string()),
        },
    })
}
#[ffi]
pub fn serial_studio_poll() -> ffi::Result<Wire<PollResponse>, BridgeError> {
    invoke(|| match serial_studio_service::poll() {
        Ok(payload) => PollResponse {
            payload: Some(payload),
            error: None,
        },
        Err(error) => PollResponse {
            payload: None,
            error: Some(error.to_string()),
        },
    })
}
#[ffi]
pub fn serial_studio_connect(
    input: Wire<ConnectRequest>,
) -> ffi::Result<Wire<ConnectionResponse>, BridgeError> {
    panic_to_result(|| {
        let input = match request(input) {
            Ok(value) => value,
            Err(error) => return ffi::Result::Err(error),
        };
        invoke(|| connection(serial_studio_service::connect(input.path, input.config)))
    })
}
fn connection(
    result: Result<SerialStatus, serial_studio_service::CoreError>,
) -> ConnectionResponse {
    match result {
        Ok(payload) => ConnectionResponse {
            payload: Some(payload),
            error: None,
        },
        Err(error) => ConnectionResponse {
            payload: None,
            error: Some(error.to_string()),
        },
    }
}
#[ffi]
pub fn serial_studio_disconnect() -> ffi::Result<Wire<ConnectionResponse>, BridgeError> {
    invoke(|| connection(serial_studio_service::disconnect()))
}
#[ffi]
pub fn serial_studio_send(
    input: Wire<SendRequest>,
) -> ffi::Result<Wire<StatusResponse>, BridgeError> {
    panic_to_result(|| {
        let input = match request(input) {
            Ok(value) => value,
            Err(error) => return ffi::Result::Err(error),
        };
        invoke(|| StatusResponse {
            error: serial_studio_service::send(input.text, input.format, input.line_ending)
                .err()
                .map(|error| error.to_string()),
        })
    })
}

pub fn inventory() -> RustInventory {
    RustInventory::new()
        .register(builtins_wire!())
        .register(function!(serial_studio_ports))
        .register(function!(serial_studio_poll))
        .register(function!(serial_studio_connect))
        .register(function!(serial_studio_disconnect))
        .register(function!(serial_studio_send))
        .validate()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn wire_preserves_serial_config_and_transport_errors() {
        let config = SerialConfig::default();
        let request = Wire::from(ConnectRequest {
            path: String::new(),
            config,
        });
        let response = serial_studio_connect(request).unwrap().unwire();
        assert!(response.error.unwrap().contains("请选择串口"));
        let request = Wire::from(SendRequest {
            text: "中文".into(),
            format: SendFormat::Ascii,
            line_ending: LineEnding::Crlf,
        });
        let response = serial_studio_send(request).unwrap().unwire();
        assert!(response.error.unwrap().contains("ASCII"));
    }
}
