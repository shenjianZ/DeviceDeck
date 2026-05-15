use std::sync::Arc;

use crate::core::error::AppError;
use crate::core::types::{DeviceActionResult, DeviceInfo, DeviceKeyAction, RecommendedConfig};
use crate::providers::android::provider::AndroidProvider;
use crate::providers::android::types::WirelessAdbService;
use crate::providers::provider_trait::DeviceProvider;

pub struct DeviceService {
    android_provider: Arc<AndroidProvider>,
}

impl DeviceService {
    pub fn new(android_provider: Arc<AndroidProvider>) -> Self {
        Self { android_provider }
    }

    pub async fn scan_devices(&self) -> Result<Vec<DeviceInfo>, AppError> {
        self.android_provider.scan_devices().await
    }

    pub async fn get_device_detail(&self, serial: &str) -> Result<DeviceInfo, AppError> {
        self.android_provider.get_device_detail(serial).await
    }

    pub async fn enable_wireless_device(
        &self,
        serial: &str,
        port: u16,
    ) -> Result<DeviceInfo, AppError> {
        self.android_provider
            .enable_wireless_device(serial, port)
            .await
    }

    pub async fn connect_wireless_device(
        &self,
        host: &str,
        port: u16,
    ) -> Result<DeviceInfo, AppError> {
        self.android_provider
            .connect_wireless_device(host, port)
            .await
    }

    pub async fn discover_wireless_services(&self) -> Result<Vec<WirelessAdbService>, AppError> {
        self.android_provider.discover_wireless_services().await
    }

    pub async fn pair_wireless_device(
        &self,
        host: &str,
        port: u16,
        pairing_code: &str,
    ) -> Result<String, AppError> {
        self.android_provider
            .pair_wireless_device(host, port, pairing_code)
            .await
    }

    pub async fn disconnect_wireless_device(&self, serial: &str) -> Result<(), AppError> {
        self.android_provider
            .disconnect_wireless_device(serial)
            .await
    }

    pub async fn detect_capabilities(
        &self,
        serial: &str,
    ) -> Result<Vec<RecommendedConfig>, AppError> {
        self.android_provider.detect_capabilities(serial).await
    }

    pub async fn take_screenshot(
        &self,
        serial: &str,
        output_directory: Option<&str>,
    ) -> Result<DeviceActionResult, AppError> {
        self.android_provider
            .take_screenshot(serial, output_directory)
            .await
    }

    pub async fn install_apk(
        &self,
        serial: &str,
        apk_path: &str,
    ) -> Result<DeviceActionResult, AppError> {
        self.android_provider.install_apk(serial, apk_path).await
    }

    pub async fn push_file(
        &self,
        serial: &str,
        local_path: &str,
        remote_directory: &str,
    ) -> Result<DeviceActionResult, AppError> {
        self.android_provider
            .push_file(serial, local_path, remote_directory)
            .await
    }

    pub async fn run_key_action(
        &self,
        serial: &str,
        action: DeviceKeyAction,
    ) -> Result<DeviceActionResult, AppError> {
        self.android_provider.run_key_action(serial, action).await
    }

    pub async fn run_shell_command(
        &self,
        serial: &str,
        command: &str,
        timeout_ms: Option<u64>,
    ) -> Result<DeviceActionResult, AppError> {
        self.android_provider
            .run_shell_command(serial, command, timeout_ms)
            .await
    }
}
