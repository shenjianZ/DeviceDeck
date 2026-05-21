use std::sync::Arc;

use crate::core::error::AppError;
use crate::core::types::{DeviceActionResult, FileEntry, WifiTransferStatus};
use crate::providers::android::provider::AndroidProvider;

pub struct TransferService {
    android_provider: Arc<AndroidProvider>,
    app_handle: tauri::AppHandle,
    wifi_status: Arc<std::sync::Mutex<WifiTransferStatus>>,
    shutdown_tx: Arc<std::sync::Mutex<Option<tokio::sync::watch::Sender<bool>>>>,
}

impl TransferService {
    pub fn new(android_provider: Arc<AndroidProvider>, app_handle: tauri::AppHandle) -> Self {
        Self {
            android_provider,
            app_handle,
            wifi_status: Arc::new(std::sync::Mutex::new(WifiTransferStatus {
                running: false,
                url: None,
                token: None,
                qr_code_data_url: None,
                port: 0,
            })),
            shutdown_tx: Arc::new(std::sync::Mutex::new(None)),
        }
    }

    pub async fn list_directory(
        &self,
        serial: &str,
        path: &str,
    ) -> Result<Vec<FileEntry>, AppError> {
        self.android_provider.list_directory(serial, path).await
    }

    pub async fn pull_file(
        &self,
        serial: &str,
        remote_path: &str,
        local_directory: &str,
    ) -> Result<DeviceActionResult, AppError> {
        self.android_provider
            .pull_file(serial, remote_path, local_directory)
            .await
    }

    pub async fn delete_device_file(
        &self,
        serial: &str,
        path: &str,
    ) -> Result<DeviceActionResult, AppError> {
        self.android_provider.delete_file(serial, path).await
    }

    pub async fn create_directory(
        &self,
        serial: &str,
        path: &str,
    ) -> Result<DeviceActionResult, AppError> {
        self.android_provider.create_directory(serial, path).await
    }

    pub async fn create_file(
        &self,
        serial: &str,
        path: &str,
    ) -> Result<DeviceActionResult, AppError> {
        self.android_provider.create_file(serial, path).await
    }

    pub fn get_wifi_transfer_status(&self) -> WifiTransferStatus {
        self.wifi_status
            .lock()
            .map(|s| s.clone())
            .unwrap_or(WifiTransferStatus {
                running: false,
                url: None,
                token: None,
                qr_code_data_url: None,
                port: 0,
            })
    }

    pub fn update_wifi_status(&self, status: WifiTransferStatus) {
        if let Ok(mut s) = self.wifi_status.lock() {
            *s = status;
        }
    }

    pub fn app_handle(&self) -> tauri::AppHandle {
        self.app_handle.clone()
    }

    pub fn set_shutdown_tx(&self, tx: tokio::sync::watch::Sender<bool>) {
        if let Ok(mut s) = self.shutdown_tx.lock() {
            *s = Some(tx);
        }
    }

    pub fn send_shutdown(&self) -> Result<(), AppError> {
        if let Ok(mut s) = self.shutdown_tx.lock() {
            if let Some(tx) = s.take() {
                let _ = tx.send(true);
            }
        }
        Ok(())
    }

    pub fn adb_path(&self) -> Result<std::path::PathBuf, AppError> {
        self.android_provider.get_adb_path()
    }
}
