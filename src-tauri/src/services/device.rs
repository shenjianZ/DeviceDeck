use std::sync::Arc;

use crate::core::error::AppError;
use crate::core::types::DeviceInfo;
use crate::providers::android::provider::AndroidProvider;
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

    pub async fn enable_wireless_device(&self, serial: &str, port: u16) -> Result<DeviceInfo, AppError> {
        self.android_provider.enable_wireless_device(serial, port).await
    }

    pub async fn connect_wireless_device(&self, host: &str, port: u16) -> Result<DeviceInfo, AppError> {
        self.android_provider.connect_wireless_device(host, port).await
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
        self.android_provider.disconnect_wireless_device(serial).await
    }
}
