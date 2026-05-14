use crate::core::error::AppError;
use crate::core::types::AppLog;
use crate::repositories::database::Database;

pub struct LogRepository<'a> {
    db: &'a Database,
}

impl<'a> LogRepository<'a> {
    pub fn new(db: &'a Database) -> Self {
        Self { db }
    }

    pub fn insert_log(&self, log: &AppLog) -> Result<(), AppError> {
        let conn = self.db.conn()?;
        conn.execute(
            "INSERT INTO logs (id, time, source, level, device_serial, message) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![
                log.id,
                log.time as i64,
                serde_json::to_string(&log.source)?,
                serde_json::to_string(&log.level)?,
                log.device_serial,
                log.message,
            ],
        )?;
        Ok(())
    }

    pub fn get_recent_logs(&self, limit: u32) -> Result<Vec<AppLog>, AppError> {
        let conn = self.db.conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, time, source, level, device_serial, message FROM logs ORDER BY time DESC LIMIT ?1",
        )?;

        let logs = stmt
            .query_map([limit], |row| {
                let source_str: String = row.get(2)?;
                let level_str: String = row.get(3)?;
                Ok(AppLog {
                    id: row.get(0)?,
                    time: row.get(1)?,
                    source: serde_json::from_str(&source_str).unwrap_or(crate::core::types::LogSource::System),
                    level: serde_json::from_str(&level_str).unwrap_or(crate::core::types::LogLevel::Info),
                    device_serial: row.get(4)?,
                    message: row.get(5)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(logs)
    }

    pub fn clear_logs(&self) -> Result<(), AppError> {
        let conn = self.db.conn()?;
        conn.execute("DELETE FROM logs", [])?;
        Ok(())
    }

    pub fn cleanup_old_logs(&self, retention_days: u32) -> Result<u64, AppError> {
        let cutoff = now_millis() - (retention_days as u64 * 24 * 60 * 60 * 1000);
        let conn = self.db.conn()?;
        let affected = conn.execute("DELETE FROM logs WHERE time < ?1", [cutoff as i64])?;
        Ok(affected as u64)
    }
}

fn now_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
