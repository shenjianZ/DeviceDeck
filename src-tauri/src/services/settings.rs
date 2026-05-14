use crate::core::error::AppError;
use crate::core::types::AppSettings;
use crate::repositories::settings::SettingsRepository;

pub struct SettingsService {
    repo: SettingsRepository,
}

impl SettingsService {
    pub fn new(repo: SettingsRepository) -> Self {
        Self { repo }
    }

    pub fn get_settings(&self) -> Result<AppSettings, AppError> {
        self.repo.load()
    }

    pub fn update_settings(&self, settings: &AppSettings) -> Result<(), AppError> {
        self.repo.save(settings)
    }

    pub fn reset_settings(&self) -> Result<AppSettings, AppError> {
        self.repo.reset()
    }
}
