use rusqlite::Connection;
use std::path::Path;
use std::sync::Mutex;

use crate::core::error::AppError;

pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    pub fn open(data_dir: &Path) -> Result<Self, AppError> {
        std::fs::create_dir_all(data_dir)
            .map_err(|e| AppError::internal_error(&format!("无法创建数据目录: {e}")))?;

        let db_path = data_dir.join("devicedeck.db");
        let conn = Connection::open(&db_path)
            .map_err(|e| AppError::internal_error(&format!("无法打开数据库: {e}")))?;

        let db = Self {
            conn: Mutex::new(conn),
        };
        db.run_migrations()?;
        Ok(db)
    }

    fn run_migrations(&self) -> Result<(), AppError> {
        let conn = self.conn.lock().map_err(|e| AppError::internal_error(&e.to_string()))?;

        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS mirror_sessions (
                id TEXT PRIMARY KEY,
                device_serial TEXT NOT NULL,
                platform TEXT NOT NULL,
                process_id INTEGER,
                status TEXT NOT NULL DEFAULT 'running',
                started_at INTEGER NOT NULL,
                stopped_at INTEGER,
                config TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS logs (
                id TEXT PRIMARY KEY,
                time INTEGER NOT NULL,
                source TEXT NOT NULL,
                level TEXT NOT NULL,
                device_serial TEXT NOT NULL DEFAULT '',
                message TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_logs_time ON logs(time);
            ",
        ).map_err(|e| AppError::internal_error(&format!("数据库迁移失败: {e}")))?;

        Ok(())
    }

    pub fn conn(&self) -> Result<std::sync::MutexGuard<'_, Connection>, AppError> {
        self.conn.lock().map_err(|e| AppError::internal_error(&e.to_string()))
    }
}
