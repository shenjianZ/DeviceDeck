use crate::core::app_state::AppState;
use crate::core::error::AppError;
use crate::core::types::{MirrorConfig, MirrorSession};
use crate::repositories::session::SessionRepository;
use crate::services::mirror::MirrorService;

#[tauri::command]
pub async fn start_mirror(
    state: tauri::State<'_, AppState>,
    mirror_service: tauri::State<'_, MirrorService>,
    serial: String,
    config: MirrorConfig,
) -> Result<MirrorSession, AppError> {
    let repo = SessionRepository::new(&state.db);
    mirror_service.start_mirror(&serial, &config, &repo).await
}

#[tauri::command]
pub async fn start_wireless_mirror(
    state: tauri::State<'_, AppState>,
    mirror_service: tauri::State<'_, MirrorService>,
    serial: String,
    config: MirrorConfig,
    port: u16,
) -> Result<MirrorSession, AppError> {
    let repo = SessionRepository::new(&state.db);
    mirror_service
        .start_wireless_mirror(&serial, &config, port, &repo)
        .await
}

#[tauri::command]
pub async fn connect_wireless_and_start_mirror(
    state: tauri::State<'_, AppState>,
    mirror_service: tauri::State<'_, MirrorService>,
    host: String,
    port: u16,
    config: MirrorConfig,
) -> Result<MirrorSession, AppError> {
    let repo = SessionRepository::new(&state.db);
    mirror_service
        .connect_wireless_and_start_mirror(&host, port, &config, &repo)
        .await
}

#[tauri::command]
pub async fn stop_mirror(
    state: tauri::State<'_, AppState>,
    mirror_service: tauri::State<'_, MirrorService>,
    session_id: String,
) -> Result<(), AppError> {
    let repo = SessionRepository::new(&state.db);
    mirror_service.stop_mirror(&session_id, &repo).await
}

#[tauri::command]
pub async fn list_mirror_sessions(
    state: tauri::State<'_, AppState>,
    mirror_service: tauri::State<'_, MirrorService>,
) -> Result<Vec<MirrorSession>, AppError> {
    let repo = SessionRepository::new(&state.db);
    mirror_service.list_sessions(&repo).await
}
