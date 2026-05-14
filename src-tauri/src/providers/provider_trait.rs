use async_trait::async_trait;

use crate::core::error::AppError;
use crate::core::types::{
    DeviceInfo, DevicePlatform, EnvironmentStatus, MirrorConfig, MirrorSession,
};

#[async_trait]
pub trait DeviceProvider: Send + Sync {
    fn platform(&self) -> DevicePlatform;

    async fn check_environment(&self) -> Result<EnvironmentStatus, AppError>;

    async fn scan_devices(&self) -> Result<Vec<DeviceInfo>, AppError>;

    async fn get_device_detail(&self, serial: &str) -> Result<DeviceInfo, AppError>;

    async fn start_mirror(
        &self,
        serial: &str,
        config: &MirrorConfig,
        scrcpy_path: &std::path::Path,
    ) -> Result<MirrorSession, AppError>;

    async fn stop_mirror(&self, session_id: &str) -> Result<(), AppError>;
}
