use async_trait::async_trait;

use super::adb;
use super::parser;
use super::scrcpy;
use crate::config::APP_VERSION;
use crate::core::error::AppError;
use crate::core::types::{
    AppSettings, DeviceActionResult, DeviceCapabilityReport, DeviceInfo, DeviceKeyAction,
    DeviceStatus, EnvironmentStatus, FileEntry, RecommendedConfig, ToolStatus,
};
use crate::providers::android::types::WirelessAdbService;
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
        self.settings.lock().map(|s| s.clone()).unwrap_or_default()
    }

    fn resolve_adb(&self) -> Result<std::path::PathBuf, AppError> {
        BinaryResolver::resolve_adb(&self.current_settings())
    }

    pub fn get_adb_path(&self) -> Result<std::path::PathBuf, AppError> {
        self.resolve_adb()
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

    pub async fn discover_wireless_services(&self) -> Result<Vec<WirelessAdbService>, AppError> {
        let adb_path = self.resolve_adb()?;
        adb::execute_adb_mdns_services(&adb_path).await
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
            return Err(AppError::invalid_config("Pairing code cannot be empty"));
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

    pub async fn detect_capabilities(
        &self,
        serial: &str,
    ) -> Result<Vec<RecommendedConfig>, AppError> {
        let scrcpy_path = self.resolve_scrcpy()?;
        let adb_path = self.resolve_adb()?;

        let (encoders, codecs) = scrcpy::execute_list_encoders(&scrcpy_path, serial).await?;

        let props = adb::execute_get_device_props(&adb_path, serial).await?;

        let (screen_width, screen_height) = props
            .screen_size
            .as_deref()
            .and_then(parser::parse_screen_resolution)
            .unzip();

        let report = DeviceCapabilityReport {
            serial: serial.into(),
            supported_encoders: encoders,
            supported_codecs: codecs,
            screen_width,
            screen_height,
            android_version: Some(props.android_version),
        };

        Ok(scrcpy::generate_recommendations(&report))
    }

    pub async fn take_screenshot(
        &self,
        serial: &str,
        output_directory: Option<&str>,
    ) -> Result<DeviceActionResult, AppError> {
        let adb_path = self.resolve_adb()?;
        adb::execute_screenshot(&adb_path, serial, output_directory).await
    }

    pub async fn install_apk(
        &self,
        serial: &str,
        apk_path: &str,
    ) -> Result<DeviceActionResult, AppError> {
        let adb_path = self.resolve_adb()?;
        adb::execute_install_apk(&adb_path, serial, apk_path).await
    }

    pub async fn push_file(
        &self,
        serial: &str,
        local_path: &str,
        remote_directory: &str,
    ) -> Result<DeviceActionResult, AppError> {
        let adb_path = self.resolve_adb()?;
        adb::execute_push_file(&adb_path, serial, local_path, remote_directory).await
    }

    pub async fn run_key_action(
        &self,
        serial: &str,
        action: DeviceKeyAction,
    ) -> Result<DeviceActionResult, AppError> {
        let adb_path = self.resolve_adb()?;
        adb::execute_key_action(&adb_path, serial, action).await
    }

    pub async fn run_shell_command(
        &self,
        serial: &str,
        command: &str,
        timeout_ms: Option<u64>,
    ) -> Result<DeviceActionResult, AppError> {
        let adb_path = self.resolve_adb()?;
        adb::execute_shell_command(&adb_path, serial, command, timeout_ms).await
    }

    pub async fn list_directory(
        &self,
        serial: &str,
        path: &str,
    ) -> Result<Vec<FileEntry>, AppError> {
        let adb_path = self.resolve_adb()?;
        adb::execute_list_directory(&adb_path, serial, path).await
    }

    pub async fn pull_file(
        &self,
        serial: &str,
        remote_path: &str,
        local_directory: &str,
    ) -> Result<DeviceActionResult, AppError> {
        let adb_path = self.resolve_adb()?;
        adb::execute_pull_file(&adb_path, serial, remote_path, local_directory).await
    }

    pub async fn delete_file(
        &self,
        serial: &str,
        path: &str,
    ) -> Result<DeviceActionResult, AppError> {
        let adb_path = self.resolve_adb()?;
        adb::execute_delete_file(&adb_path, serial, path).await
    }

    pub async fn create_directory(
        &self,
        serial: &str,
        path: &str,
    ) -> Result<DeviceActionResult, AppError> {
        let adb_path = self.resolve_adb()?;
        adb::execute_create_directory(&adb_path, serial, path).await
    }

    pub async fn create_file(
        &self,
        serial: &str,
        path: &str,
    ) -> Result<DeviceActionResult, AppError> {
        let adb_path = self.resolve_adb()?;
        adb::execute_create_file(&adb_path, serial, path).await
    }
}

fn validate_wireless_host(host: &str) -> Result<(), AppError> {
    let host = host.trim();
    if host.is_empty() {
        return Err(AppError::invalid_config("IP address cannot be empty"));
    }
    if host.contains([';', '|', '&', '$', '`', '<', '>', '"', '\'']) {
        return Err(AppError::invalid_config(
            "IP address contains invalid characters",
        ));
    }
    Ok(())
}

fn validate_wireless_endpoint(endpoint: &str) -> Result<(), AppError> {
    if endpoint.trim().is_empty() {
        return Err(AppError::invalid_config(
            "Wireless device address cannot be empty",
        ));
    }
    if endpoint.contains([';', '|', '&', '$', '`', '<', '>', '"', '\'']) {
        return Err(AppError::invalid_config(
            "Wireless device address contains invalid characters",
        ));
    }
    Ok(())
}

fn validate_wireless_port(port: u16) -> Result<(), AppError> {
    if port == 0 {
        return Err(AppError::invalid_config("Port cannot be 0"));
    }
    Ok(())
}

#[async_trait]
impl DeviceProvider for AndroidProvider {
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
            provider_status: format!("Running v{APP_VERSION}"),
        })
    }

    async fn scan_devices(&self) -> Result<Vec<DeviceInfo>, AppError> {
        let adb_path = self.resolve_adb()?;
        let raw_devices = adb::execute_adb_devices(&adb_path).await?;

        let mut devices = Vec::new();
        for raw in &raw_devices {
            let mut info = adb::raw_device_to_info(raw);

            if info.status == DeviceStatus::Online {
                if let Ok(props) = adb::execute_get_device_props(&adb_path, &info.serial).await {
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
}
