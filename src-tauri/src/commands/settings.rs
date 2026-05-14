use std::sync::Arc;

use crate::core::error::AppError;
use crate::core::types::AppSettings;
use crate::providers::android::provider::AndroidProvider;
use crate::services::mirror::MirrorService;
use crate::services::settings::SettingsService;

#[tauri::command]
pub fn get_settings(
    settings_service: tauri::State<'_, SettingsService>,
) -> Result<AppSettings, AppError> {
    settings_service.get_settings()
}

#[tauri::command]
pub fn update_settings(
    settings_service: tauri::State<'_, SettingsService>,
    android_provider: tauri::State<'_, Arc<AndroidProvider>>,
    mirror_service: tauri::State<'_, MirrorService>,
    settings: AppSettings,
) -> Result<(), AppError> {
    settings_service.update_settings(&settings)?;
    android_provider.update_settings(settings.clone());
    mirror_service.update_settings(settings)?;
    Ok(())
}

#[tauri::command]
pub fn reset_settings(
    settings_service: tauri::State<'_, SettingsService>,
    android_provider: tauri::State<'_, Arc<AndroidProvider>>,
    mirror_service: tauri::State<'_, MirrorService>,
) -> Result<AppSettings, AppError> {
    let settings = settings_service.reset_settings()?;
    android_provider.update_settings(settings.clone());
    mirror_service.update_settings(settings.clone())?;
    Ok(settings)
}
