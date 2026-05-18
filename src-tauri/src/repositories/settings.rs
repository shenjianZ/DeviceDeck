use std::path::PathBuf;

use crate::core::error::AppError;
use crate::core::types::AppSettings;

pub struct SettingsRepository {
    config_path: PathBuf,
}

impl SettingsRepository {
    pub fn new(data_dir: &std::path::Path) -> Self {
        Self {
            config_path: data_dir.join("config.json"),
        }
    }

    pub fn load(&self) -> Result<AppSettings, AppError> {
        if !self.config_path.exists() {
            return Ok(AppSettings::default());
        }

        let content = std::fs::read_to_string(&self.config_path)?;
        if content.trim().is_empty() {
            return Ok(AppSettings::default());
        }

        let settings: AppSettings = serde_json::from_str(&content).unwrap_or_else(|e| {
            eprintln!("Config file corrupted, using defaults: {e}");
            AppSettings::default()
        });

        Ok(settings)
    }

    pub fn save(&self, settings: &AppSettings) -> Result<(), AppError> {
        if let Some(parent) = self.config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(settings)?;
        std::fs::write(&self.config_path, content)?;
        Ok(())
    }

    pub fn reset(&self) -> Result<AppSettings, AppError> {
        let default = AppSettings::default();
        self.save(&default)?;
        Ok(default)
    }
}
