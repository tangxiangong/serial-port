pub use serial_studio_service::{
    CoreError, DecodedText, Direction, FlowControl, LineEnding, Parity, PollBatch, PortInfo,
    SendFormat, SerialConfig, SerialFrameEvent, SerialStatus,
};

#[uniffi::remote(Error)]
pub enum CoreError {
    Operation { message: String },
}
#[uniffi::remote(Enum)]
pub enum Direction {
    Receive,
    Send,
}
#[uniffi::remote(Enum)]
pub enum Parity {
    None,
    Odd,
    Even,
}
#[uniffi::remote(Enum)]
pub enum FlowControl {
    None,
    Software,
    Hardware,
}
#[uniffi::remote(Enum)]
pub enum SendFormat {
    Hex,
    Ascii,
    Gbk,
    Utf8,
}
#[uniffi::remote(Enum)]
pub enum LineEnding {
    None,
    Lf,
    Cr,
    Crlf,
}
#[uniffi::remote(Record)]
pub struct SerialConfig {
    pub baud_rate: u32,
    pub data_bits: u8,
    pub stop_bits: u8,
    pub parity: Parity,
    pub flow_control: FlowControl,
}
#[uniffi::remote(Record)]
pub struct PortInfo {
    pub path: String,
    pub display_name: String,
    pub manufacturer: Option<String>,
    pub serial_number: Option<String>,
    pub vid: Option<String>,
    pub pid: Option<String>,
}
#[uniffi::remote(Record)]
pub struct SerialStatus {
    pub connected: bool,
    pub path: Option<String>,
    pub config: Option<SerialConfig>,
    pub rx_bytes: u64,
    pub tx_bytes: u64,
    pub opened_at: Option<String>,
    pub last_error: Option<String>,
}
#[uniffi::remote(Record)]
pub struct DecodedText {
    pub ascii: String,
    pub gbk: String,
    pub utf8: String,
}
#[uniffi::remote(Record)]
pub struct SerialFrameEvent {
    pub direction: Direction,
    pub bytes: Vec<u8>,
    pub hex: String,
    pub text: DecodedText,
    pub timestamp: String,
}
#[uniffi::remote(Record)]
pub struct PollBatch {
    pub status: SerialStatus,
    pub frames: Vec<SerialFrameEvent>,
    pub dropped_frames: u64,
}
