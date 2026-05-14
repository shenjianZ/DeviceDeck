use std::sync::Arc;

use crate::core::error::AppError;
use crate::core::types::EnvironmentStatus;
use crate::providers::android::provider::AndroidProvider;
use crate::providers::provider_trait::DeviceProvider;

pub struct EnvironmentService {
    android_provider: Arc<AndroidProvider>,
}

impl EnvironmentService {
    pub fn new(android_provider: Arc<AndroidProvider>) -> Self {
        Self { android_provider }
    }

    pub async fn check(&self) -> Result<EnvironmentStatus, AppError> {
        self.android_provider.check_environment().await
    }
}
