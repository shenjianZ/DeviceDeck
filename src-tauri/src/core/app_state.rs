use std::sync::Arc;

use crate::core::log_bus::LogBus;
use crate::repositories::database::Database;

pub struct AppState {
    pub db: Arc<Database>,
    pub log_bus: Arc<LogBus>,
}

impl AppState {
    pub fn new(db: Arc<Database>, log_bus: Arc<LogBus>) -> Self {
        Self { db, log_bus }
    }
}
