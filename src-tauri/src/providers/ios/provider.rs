use async_trait::async_trait;

use crate::core::error::AppError;
use crate::core::types::{
    DeviceInfo, DevicePlatform, EnvironmentStatus, MirrorConfig, MirrorSession, ToolStatus,
};
use crate::providers::provider_trait::DeviceProvider;

pub struct IosProvider;

impl IosProvider {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl DeviceProvider for IosProvider {
    fn platform(&self) -> DevicePlatform {
        DevicePlatform::Ios
    }

    async fn check_environment(&self) -> Result<EnvironmentStatus, AppError> {
        Ok(EnvironmentStatus {
            adb: ToolStatus {
                name: "N/A".into(),
                available: false,
                path: None,
                version: None,
                message: Some("iOS 平台暂不支持".into()),
            },
            scrcpy: ToolStatus {
                name: "N/A".into(),
                available: false,
                path: None,
                version: None,
                message: Some("iOS 平台暂不支持".into()),
            },
            provider_status: "Coming Soon".into(),
        })
    }

    async fn scan_devices(&self) -> Result<Vec<DeviceInfo>, AppError> {
        Ok(vec![])
    }

    async fn get_device_detail(&self, _serial: &str) -> Result<DeviceInfo, AppError> {
        Err(AppError::provider_not_implemented("iOS"))
    }

    async fn start_mirror(
        &self,
        _serial: &str,
        _config: &MirrorConfig,
        _scrcpy_path: &std::path::Path,
    ) -> Result<MirrorSession, AppError> {
        Err(AppError::provider_not_implemented("iOS"))
    }

    async fn stop_mirror(&self, _session_id: &str) -> Result<(), AppError> {
        Err(AppError::provider_not_implemented("iOS"))
    }
}
