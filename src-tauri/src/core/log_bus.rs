use std::sync::Arc;

use tauri::{AppHandle, Emitter};
use uuid::Uuid;

use super::types::{AppLog, LogLevel, LogSource};
use crate::repositories::database::Database;
use crate::repositories::log::LogRepository;

pub struct LogBus {
    app_handle: AppHandle,
    db: Arc<Database>,
}

impl LogBus {
    pub fn new(app_handle: AppHandle, db: Arc<Database>) -> Self {
        Self { app_handle, db }
    }

    pub fn emit_log(
        &self,
        source: LogSource,
        level: LogLevel,
        device_serial: &str,
        message: &str,
    ) {
        let log = AppLog {
            id: Uuid::new_v4().to_string(),
            time: now_millis(),
            source,
            level,
            device_serial: device_serial.into(),
            message: message.into(),
        };

        let repo = LogRepository::new(&self.db);
        if let Err(e) = repo.insert_log(&log) {
            eprintln!("Failed to persist log: {e}");
        }

        if let Err(e) = self.app_handle.emit("log://new", &log) {
            eprintln!("Failed to emit log event: {e}");
        }
    }

    pub fn system_info(&self, message: &str) {
        self.emit_log(LogSource::System, LogLevel::Info, "", message);
    }

    pub fn system_error(&self, message: &str) {
        self.emit_log(LogSource::System, LogLevel::Error, "", message);
    }

    pub fn adb_info(&self, device_serial: &str, message: &str) {
        self.emit_log(LogSource::Adb, LogLevel::Info, device_serial, message);
    }

    pub fn adb_error(&self, device_serial: &str, message: &str) {
        self.emit_log(LogSource::Adb, LogLevel::Error, device_serial, message);
    }

    pub fn scrcpy_info(&self, device_serial: &str, message: &str) {
        self.emit_log(LogSource::Scrcpy, LogLevel::Info, device_serial, message);
    }

    pub fn scrcpy_error(&self, device_serial: &str, message: &str) {
        self.emit_log(LogSource::Scrcpy, LogLevel::Error, device_serial, message);
    }
}

fn now_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
