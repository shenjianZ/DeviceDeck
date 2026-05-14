use std::path::Path;

use async_trait::async_trait;
use uuid::Uuid;

use super::adb;
use super::scrcpy;
use crate::core::error::AppError;
use crate::core::types::{
    AppSettings, DeviceInfo, DevicePlatform, DeviceStatus, EnvironmentStatus, MirrorConfig,
    MirrorSession, SessionStatus, ToolStatus,
};
use crate::providers::provider_trait::DeviceProvider;
use crate::sidecar::binary_resolver::BinaryResolver;
use crate::sidecar::shell_runner::ShellRunner;

pub struct AndroidProvider {
    settings: std::sync::Mutex<AppSettings>,
}

impl AndroidProvider {
    pub fn new(settings: AppSettings) -> Self {
        Self {
            settings: std::sync::Mutex::new(settings),
        }
    }

    pub fn update_settings(&self, settings: AppSettings) {
        if let Ok(mut s) = self.settings.lock() {
            *s = settings;
        }
    }

    fn current_settings(&self) -> AppSettings {
        self.settings
            .lock()
            .map(|s| s.clone())
            .unwrap_or_default()
    }

    fn resolve_adb(&self) -> Result<std::path::PathBuf, AppError> {
        BinaryResolver::resolve_adb(&self.current_settings())
    }

    fn resolve_scrcpy(&self) -> Result<std::path::PathBuf, AppError> {
        BinaryResolver::resolve_scrcpy(&self.current_settings())
    }

    pub async fn enable_wireless_device(
        &self,
        serial: &str,
        port: u16,
    ) -> Result<DeviceInfo, AppError> {
        validate_wireless_port(port)?;
        let adb_path = self.resolve_adb()?;
        let ip = adb::execute_get_device_ip(&adb_path, serial).await?;
        adb::execute_adb_tcpip(&adb_path, serial, port).await?;
        tokio::time::sleep(std::time::Duration::from_millis(1500)).await;

        let endpoint = format!("{ip}:{port}");
        adb::execute_adb_connect(&adb_path, &endpoint).await?;
        self.get_device_detail(&endpoint).await
    }

    pub async fn connect_wireless_device(
        &self,
        host: &str,
        port: u16,
    ) -> Result<DeviceInfo, AppError> {
        validate_wireless_host(host)?;
        validate_wireless_port(port)?;
        let adb_path = self.resolve_adb()?;
        let endpoint = format!("{}:{}", host.trim(), port);
        adb::execute_adb_connect(&adb_path, &endpoint).await?;
        self.get_device_detail(&endpoint).await
    }

    pub async fn pair_wireless_device(
        &self,
        host: &str,
        port: u16,
        pairing_code: &str,
    ) -> Result<String, AppError> {
        validate_wireless_host(host)?;
        validate_wireless_port(port)?;
        if pairing_code.trim().is_empty() {
            return Err(AppError::invalid_config("配对码不能为空"));
        }

        let adb_path = self.resolve_adb()?;
        let endpoint = format!("{}:{}", host.trim(), port);
        adb::execute_adb_pair(&adb_path, &endpoint, pairing_code.trim()).await
    }

    pub async fn disconnect_wireless_device(&self, serial: &str) -> Result<(), AppError> {
        validate_wireless_endpoint(serial)?;
        let adb_path = self.resolve_adb()?;
        adb::execute_adb_disconnect(&adb_path, serial).await
    }
}

fn validate_wireless_host(host: &str) -> Result<(), AppError> {
    let host = host.trim();
    if host.is_empty() {
        return Err(AppError::invalid_config("IP 地址不能为空"));
    }
    if host.contains([';', '|', '&', '$', '`', '<', '>', '"', '\'']) {
        return Err(AppError::invalid_config("IP 地址包含非法字符"));
    }
    Ok(())
}

fn validate_wireless_endpoint(endpoint: &str) -> Result<(), AppError> {
    if endpoint.trim().is_empty() {
        return Err(AppError::invalid_config("无线设备地址不能为空"));
    }
    if endpoint.contains([';', '|', '&', '$', '`', '<', '>', '"', '\'']) {
        return Err(AppError::invalid_config("无线设备地址包含非法字符"));
    }
    Ok(())
}

fn validate_wireless_port(port: u16) -> Result<(), AppError> {
    if port == 0 {
        return Err(AppError::invalid_config("端口不能为 0"));
    }
    Ok(())
}

#[async_trait]
impl DeviceProvider for AndroidProvider {
    fn platform(&self) -> DevicePlatform {
        DevicePlatform::Android
    }

    async fn check_environment(&self) -> Result<EnvironmentStatus, AppError> {
        let adb_status = match self.resolve_adb() {
            Ok(path) => {
                let version_result = ShellRunner::execute(&path, &["version"]).await;
                match version_result {
                    Ok(output) => {
                        let ver = output
                            .stdout
                            .lines()
                            .next()
                            .unwrap_or("")
                            .trim()
                            .to_string();
                        ToolStatus {
                            name: "ADB".into(),
                            available: true,
                            path: Some(path.to_string_lossy().into()),
                            version: Some(ver),
                            message: None,
                        }
                    }
                    Err(e) => ToolStatus {
                        name: "ADB".into(),
                        available: false,
                        path: None,
                        version: None,
                        message: Some(e.message),
                    },
                }
            }
            Err(e) => ToolStatus {
                name: "ADB".into(),
                available: false,
                path: None,
                version: None,
                message: Some(e.message),
            },
        };

        let scrcpy_status = match self.resolve_scrcpy() {
            Ok(path) => {
                let version_result = ShellRunner::execute(&path, &["--version"]).await;
                match version_result {
                    Ok(output) => {
                        let ver = output
                            .stdout
                            .lines()
                            .next()
                            .unwrap_or("")
                            .trim()
                            .to_string();
                        ToolStatus {
                            name: "Scrcpy".into(),
                            available: true,
                            path: Some(path.to_string_lossy().into()),
                            version: Some(ver),
                            message: None,
                        }
                    }
                    Err(e) => ToolStatus {
                        name: "Scrcpy".into(),
                        available: false,
                        path: None,
                        version: None,
                        message: Some(e.message),
                    },
                }
            }
            Err(e) => ToolStatus {
                name: "Scrcpy".into(),
                available: false,
                path: None,
                version: None,
                message: Some(e.message),
            },
        };

        Ok(EnvironmentStatus {
            adb: adb_status,
            scrcpy: scrcpy_status,
            provider_status: "运行中".into(),
        })
    }

    async fn scan_devices(&self) -> Result<Vec<DeviceInfo>, AppError> {
        let adb_path = self.resolve_adb()?;
        let raw_devices = adb::execute_adb_devices(&adb_path).await?;

        let mut devices = Vec::new();
        for raw in &raw_devices {
            let mut info = adb::raw_device_to_info(raw);

            if info.status == DeviceStatus::Online {
                if let Ok(props) =
                    adb::execute_get_device_props(&adb_path, &info.serial).await
                {
                    info.name = if props.model.is_empty() {
                        info.name
                    } else {
                        props.model.clone()
                    };
                    info.model = props.model;
                    info.brand = props.brand;
                    info.android_version = Some(props.android_version);
                    info.screen_size = props.screen_size;
                    info.battery_level = props.battery_level;
                }
            }

            devices.push(info);
        }

        Ok(devices)
    }

    async fn get_device_detail(&self, serial: &str) -> Result<DeviceInfo, AppError> {
        let adb_path = self.resolve_adb()?;
        let raw_devices = adb::execute_adb_devices(&adb_path).await?;

        let raw = raw_devices
            .iter()
            .find(|d| d.serial == serial)
            .ok_or_else(|| AppError::device_not_found(serial))?;

        let mut info = adb::raw_device_to_info(raw);

        if info.status == DeviceStatus::Online {
            let props = adb::execute_get_device_props(&adb_path, serial).await?;
            info.name = if props.model.is_empty() {
                info.name
            } else {
                props.model.clone()
            };
            info.model = props.model;
            info.brand = props.brand;
            info.android_version = Some(props.android_version);
            info.screen_size = props.screen_size;
            info.battery_level = props.battery_level;
        } else if info.status == DeviceStatus::Unauthorized {
            return Err(AppError::device_unauthorized(serial));
        } else if info.status == DeviceStatus::Offline {
            return Err(AppError::device_offline(serial));
        }

        Ok(info)
    }

    async fn start_mirror(
        &self,
        serial: &str,
        config: &MirrorConfig,
        scrcpy_path: &Path,
    ) -> Result<MirrorSession, AppError> {
        let args = scrcpy::build_scrcpy_args(serial, config)?;

        let session_id = Uuid::new_v4().to_string();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let mut child = tokio::process::Command::new(scrcpy_path)
            .args(&args)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| AppError::mirror_start_failed(&e.to_string()))?;

        let pid = child.id();

        let sid = session_id.clone();
        let s = serial.to_string();
        tokio::spawn(async move {
            let _ = child.wait().await;
        });

        Ok(MirrorSession {
            id: session_id,
            device_serial: serial.into(),
            platform: "android".into(),
            process_id: pid,
            status: SessionStatus::Running,
            started_at: now,
            stopped_at: None,
            config: config.clone(),
        })
    }

    async fn stop_mirror(&self, session_id: &str) -> Result<(), AppError> {
        // The ProcessManager handles actual process killing
        // This is a placeholder that the service layer will use
        Ok(())
    }
}
