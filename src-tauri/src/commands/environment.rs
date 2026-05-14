use crate::core::error::AppError;
use crate::core::types::EnvironmentStatus;
use crate::services::environment::EnvironmentService;

#[tauri::command]
pub async fn check_environment(
    environment_service: tauri::State<'_, EnvironmentService>,
) -> Result<EnvironmentStatus, AppError> {
    environment_service.check().await
}
