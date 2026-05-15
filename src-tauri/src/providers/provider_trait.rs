use async_trait::async_trait;

use crate::core::error::AppError;
use crate::core::types::{DeviceInfo, EnvironmentStatus};

#[async_trait]
pub trait DeviceProvider: Send + Sync {
    async fn check_environment(&self) -> Result<EnvironmentStatus, AppError>;

    async fn scan_devices(&self) -> Result<Vec<DeviceInfo>, AppError>;

    async fn get_device_detail(&self, serial: &str) -> Result<DeviceInfo, AppError>;
}
