//! Platform-independent serial operations used by both generated native bridges.
mod serial;
pub use serial::encoding::DecodedText;
pub use serial::types::*;
pub use serial::{Direction, PollBatch, SerialFrameEvent, SerialManager};
