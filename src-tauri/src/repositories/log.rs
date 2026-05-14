use crate::core::error::AppError;
use crate::core::types::AppLog;
use crate::repositories::database::Database;

pub struct LogRepository<'a> {
    db: &'a Database,
}

/// 分页查询结果
#[derive(Debug, Clone, serde::Serialize)]
pub struct PaginatedLogs {
    pub logs: Vec<AppLog>,
    pub total: u32,
    pub page: u32,
    pub page_size: u32,
    pub total_pages: u32,
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

    /// 分页查询日志
    pub fn get_logs_paginated(
        &self,
        page: u32,
        page_size: u32,
        source_filter: Option<&str>,
        level_filter: Option<&str>,
    ) -> Result<PaginatedLogs, AppError> {
        let conn = self.db.conn()?;
        let page = page.max(1);
        let page_size = page_size.clamp(1, 100);
        let offset = (page - 1) * page_size;

        // 构建 WHERE 子句
        let mut conditions = Vec::new();
        let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

        if let Some(source) = source_filter {
            if source != "all" {
                conditions.push("source = ?");
                params.push(Box::new(format!("\"{}\"", source)));
            }
        }

        if let Some(level) = level_filter {
            if level != "all" {
                conditions.push("level = ?");
                params.push(Box::new(format!("\"{}\"", level)));
            }
        }

        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", conditions.join(" AND "))
        };

        // 查询总数
        let count_sql = format!("SELECT COUNT(*) FROM logs {}", where_clause);
        let total: u32 = conn.query_row(
            &count_sql,
            rusqlite::params_from_iter(params.iter().map(|p| p.as_ref())),
            |row| row.get(0),
        )?;

        // 查询分页数据
        let query_sql = format!(
            "SELECT id, time, source, level, device_serial, message FROM logs {} ORDER BY time DESC LIMIT ? OFFSET ?",
            where_clause
        );

        let mut stmt = conn.prepare(&query_sql)?;

        let mut all_params: Vec<Box<dyn rusqlite::types::ToSql>> = params;
        all_params.push(Box::new(page_size));
        all_params.push(Box::new(offset));

        let logs = stmt
            .query_map(
                rusqlite::params_from_iter(all_params.iter().map(|p| p.as_ref())),
                |row| {
                    let source_str: String = row.get(2)?;
                    let level_str: String = row.get(3)?;
                    Ok(AppLog {
                        id: row.get(0)?,
                        time: row.get(1)?,
                        source: serde_json::from_str(&source_str)
                            .unwrap_or(crate::core::types::LogSource::System),
                        level: serde_json::from_str(&level_str)
                            .unwrap_or(crate::core::types::LogLevel::Info),
                        device_serial: row.get(4)?,
                        message: row.get(5)?,
                    })
                },
            )?
            .collect::<Result<Vec<_>, _>>()?;

        let total_pages = (total as f64 / page_size as f64).ceil() as u32;

        Ok(PaginatedLogs {
            logs,
            total,
            page,
            page_size,
            total_pages,
        })
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
