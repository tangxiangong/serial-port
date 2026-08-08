use std::sync::Arc;

use crate::serial::SerialManager;

pub struct AppState {
    pub serial: Arc<SerialManager>,
}
