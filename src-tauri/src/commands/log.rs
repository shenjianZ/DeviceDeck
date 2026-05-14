use crate::core::app_state::AppState;
use crate::core::error::AppError;
use crate::core::types::AppLog;
use crate::repositories::log::{LogRepository, PaginatedLogs};

#[tauri::command]
pub fn get_recent_logs(
    state: tauri::State<'_, AppState>,
    limit: Option<u32>,
) -> Result<Vec<AppLog>, AppError> {
    let repo = LogRepository::new(&state.db);
    let mut logs = repo.get_recent_logs(limit.unwrap_or(500).clamp(1, 5000))?;
    logs.reverse();
    Ok(logs)
}

#[tauri::command]
pub fn get_logs_paginated(
    state: tauri::State<'_, AppState>,
    page: Option<u32>,
    page_size: Option<u32>,
    source_filter: Option<String>,
    level_filter: Option<String>,
) -> Result<PaginatedLogs, AppError> {
    let repo = LogRepository::new(&state.db);
    repo.get_logs_paginated(
        page.unwrap_or(1),
        page_size.unwrap_or(50),
        source_filter.as_deref(),
        level_filter.as_deref(),
    )
}

#[tauri::command]
pub fn clear_logs(state: tauri::State<'_, AppState>) -> Result<(), AppError> {
    let repo = LogRepository::new(&state.db);
    repo.clear_logs()
}
