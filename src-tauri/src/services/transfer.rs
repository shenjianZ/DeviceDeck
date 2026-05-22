use std::sync::Arc;

use crate::core::error::AppError;
use crate::core::types::{DeviceActionResult, FileEntry, WifiTransferStatus};
use crate::providers::android::provider::AndroidProvider;

pub struct TransferService {
    android_provider: Arc<AndroidProvider>,
    app_handle: tauri::AppHandle,
    wifi_status: Arc<std::sync::Mutex<WifiTransferStatus>>,
    shutdown_tx: Arc<std::sync::Mutex<Option<tokio::sync::watch::Sender<bool>>>>,
    wifi_upload_dir: Arc<std::sync::Mutex<Option<std::path::PathBuf>>>,
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
            wifi_upload_dir: Arc::new(std::sync::Mutex::new(None)),
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

    pub fn set_wifi_upload_dir(&self, dir: std::path::PathBuf) {
        if let Ok(mut d) = self.wifi_upload_dir.lock() {
            *d = Some(dir);
        }
    }

    pub fn clear_wifi_upload_dir(&self) {
        if let Ok(mut d) = self.wifi_upload_dir.lock() {
            *d = None;
        }
    }

    pub fn get_wifi_upload_dir(&self) -> Option<std::path::PathBuf> {
        self.wifi_upload_dir.lock().ok().and_then(|d| d.clone())
    }

    pub fn list_wifi_received_files(&self) -> Result<Vec<FileEntry>, AppError> {
        let dir = self
            .get_wifi_upload_dir()
            .ok_or_else(|| AppError::internal_error("WiFi transfer not running"))?;
        let mut files = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    if let Some(name) = entry.file_name().to_str() {
                        files.push(FileEntry {
                            name: name.to_string(),
                            path: entry.path().to_string_lossy().into_owned(),
                            is_directory: metadata.is_dir(),
                            size: if metadata.is_file() {
                                Some(metadata.len())
                            } else {
                                None
                            },
                            modified: metadata.modified().ok().map(|t| {
                                let datetime: chrono::DateTime<chrono::Local> = t.into();
                                datetime.format("%Y-%m-%d %H:%M").to_string()
                            }),
                            permissions: None,
                        });
                    }
                }
            }
        }
        files.sort_by(|a, b| {
            b.is_directory
                .cmp(&a.is_directory)
                .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
        });
        Ok(files)
    }

    pub fn delete_wifi_received_file(&self, name: &str) -> Result<(), AppError> {
        let dir = self
            .get_wifi_upload_dir()
            .ok_or_else(|| AppError::internal_error("WiFi transfer not running"))?;
        let sanitized: String = name
            .chars()
            .map(|c| {
                if c.is_control()
                    || matches!(
                        c,
                        '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' | '\'' | '`'
                    )
                {
                    '_'
                } else {
                    c
                }
            })
            .collect();
        let sanitized = sanitized.trim_matches([' ', '.']).to_string();
        let name = if sanitized.is_empty() {
            "unknown".into()
        } else {
            sanitized
        };
        let path = dir.join(&name);
        if !path.starts_with(&dir) {
            return Err(AppError::internal_error("Invalid file path"));
        }
        if path.is_dir() {
            std::fs::remove_dir_all(&path)?;
        } else {
            std::fs::remove_file(&path)?;
        }
        Ok(())
    }

    pub fn clear_wifi_received_files(&self) -> Result<usize, AppError> {
        let dir = self
            .get_wifi_upload_dir()
            .ok_or_else(|| AppError::internal_error("WiFi transfer not running"))?;
        let mut count = 0usize;
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    let removed = if metadata.is_dir() {
                        std::fs::remove_dir_all(entry.path()).is_ok()
                    } else if metadata.is_file() {
                        std::fs::remove_file(entry.path()).is_ok()
                    } else {
                        false
                    };
                    if removed {
                        count += 1;
                    }
                }
            }
        }
        Ok(count)
    }

    pub fn open_wifi_upload_dir(&self) -> Result<(), AppError> {
        let dir = self
            .get_wifi_upload_dir()
            .ok_or_else(|| AppError::internal_error("WiFi transfer not running"))?;
        #[cfg(target_os = "windows")]
        std::process::Command::new("explorer").arg(&dir).spawn()?;
        #[cfg(target_os = "macos")]
        std::process::Command::new("open").arg(&dir).spawn()?;
        #[cfg(target_os = "linux")]
        std::process::Command::new("xdg-open").arg(&dir).spawn()?;
        Ok(())
    }
}
