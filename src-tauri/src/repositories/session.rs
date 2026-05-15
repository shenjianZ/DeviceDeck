use crate::core::error::AppError;
use crate::core::types::{MirrorSession, SessionStatus};
use crate::repositories::database::Database;

pub struct SessionRepository<'a> {
    db: &'a Database,
}

impl<'a> SessionRepository<'a> {
    pub fn new(db: &'a Database) -> Self {
        Self { db }
    }

    pub fn save_session(&self, session: &MirrorSession) -> Result<(), AppError> {
        let conn = self.db.conn()?;
        conn.execute(
            "INSERT OR REPLACE INTO mirror_sessions (id, device_serial, platform, process_id, status, started_at, stopped_at, config)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![
                session.id,
                session.device_serial,
                session.platform,
                session.process_id,
                serde_json::to_string(&session.status)?,
                session.started_at as i64,
                session.stopped_at.map(|t| t as i64),
                serde_json::to_string(&session.config)?,
            ],
        )?;
        Ok(())
    }

    pub fn update_session_status(
        &self,
        id: &str,
        status: SessionStatus,
        stopped_at: Option<u64>,
    ) -> Result<(), AppError> {
        let conn = self.db.conn()?;
        conn.execute(
            "UPDATE mirror_sessions SET status = ?1, stopped_at = ?2 WHERE id = ?3",
            rusqlite::params![
                serde_json::to_string(&status)?,
                stopped_at.map(|t| t as i64),
                id,
            ],
        )?;
        Ok(())
    }

    pub fn get_all_sessions(&self) -> Result<Vec<MirrorSession>, AppError> {
        let conn = self.db.conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, device_serial, platform, process_id, status, started_at, stopped_at, config
             FROM mirror_sessions ORDER BY started_at DESC",
        )?;

        let sessions = stmt
            .query_map([], |row| {
                let status_str: String = row.get(4)?;
                let config_str: String = row.get(7)?;
                Ok(MirrorSession {
                    id: row.get(0)?,
                    device_serial: row.get(1)?,
                    platform: row.get(2)?,
                    process_id: row.get(3)?,
                    status: serde_json::from_str(&status_str).unwrap_or(SessionStatus::Failed),
                    started_at: row.get(5)?,
                    stopped_at: row.get(6)?,
                    config: serde_json::from_str(&config_str).unwrap_or_default(),
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(sessions)
    }
}
