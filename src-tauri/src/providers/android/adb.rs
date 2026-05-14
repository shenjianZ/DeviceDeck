use std::path::Path;

use super::parser;
use super::types::{AndroidDeviceProps, RawDevice, WirelessAdbService};
use crate::core::error::AppError;
use crate::core::types::{
    ConnectionType, DeviceCapability, DeviceInfo, DevicePlatform, DeviceStatus,
};
use crate::sidecar::shell_runner::ShellRunner;

pub async fn execute_adb_version(adb_path: &Path) -> Result<String, AppError> {
    let output = ShellRunner::execute(adb_path, &["version"]).await?;
    if !output.success {
        return Err(AppError::adb_not_found());
    }
    Ok(output.stdout.trim().to_string())
}

pub async fn execute_adb_devices(adb_path: &Path) -> Result<Vec<RawDevice>, AppError> {
    let output = ShellRunner::execute(adb_path, &["devices", "-l"]).await?;
    if !output.success {
        return Err(AppError::internal_error(&format!(
            "adb devices 执行失败: {}",
            output.stderr
        )));
    }
    Ok(parser::parse_adb_devices(&output.stdout))
}

pub async fn execute_adb_mdns_services(
    adb_path: &Path,
) -> Result<Vec<WirelessAdbService>, AppError> {
    let output = ShellRunner::execute_with_timeout(
        adb_path,
        &["mdns", "services"],
        std::time::Duration::from_secs(8),
    )
    .await?;

    if !output.success {
        return Err(AppError::wireless_discovery_failed(&output.stderr));
    }

    Ok(parser::parse_adb_mdns_services(&output.stdout))
}

pub async fn execute_get_device_props(
    adb_path: &Path,
    serial: &str,
) -> Result<AndroidDeviceProps, AppError> {
    let getprop_output = ShellRunner::execute_with_timeout(
        adb_path,
        &["-s", serial, "shell", "getprop"],
        std::time::Duration::from_secs(10),
    )
    .await?;

    let props = parser::parse_getprop(&getprop_output.stdout);

    let model = props.get("ro.product.model").cloned().unwrap_or_default();
    let brand = props.get("ro.product.brand").cloned().unwrap_or_default();
    let android_version = props
        .get("ro.build.version.release")
        .cloned()
        .unwrap_or_default();

    let wm_size_output = ShellRunner::execute_with_timeout(
        adb_path,
        &["-s", serial, "shell", "wm", "size"],
        std::time::Duration::from_secs(5),
    )
    .await;

    let screen_size = wm_size_output
        .ok()
        .and_then(|o| parser::parse_wm_size(&o.stdout));

    let battery_output = ShellRunner::execute_with_timeout(
        adb_path,
        &["-s", serial, "shell", "dumpsys", "battery"],
        std::time::Duration::from_secs(5),
    )
    .await;

    let battery_level = battery_output
        .ok()
        .and_then(|o| parser::parse_battery_level(&o.stdout));

    Ok(AndroidDeviceProps {
        model,
        brand,
        android_version,
        screen_size,
        battery_level,
    })
}

pub async fn execute_get_device_ip(adb_path: &Path, serial: &str) -> Result<String, AppError> {
    let route_output = ShellRunner::execute_with_timeout(
        adb_path,
        &["-s", serial, "shell", "ip", "route"],
        std::time::Duration::from_secs(5),
    )
    .await;

    if let Ok(output) = route_output {
        if let Some(ip) = parser::parse_ip_route_source(&output.stdout) {
            return Ok(ip);
        }
    }

    let wlan_output = ShellRunner::execute_with_timeout(
        adb_path,
        &[
            "-s", serial, "shell", "ip", "-f", "inet", "addr", "show", "wlan0",
        ],
        std::time::Duration::from_secs(5),
    )
    .await?;

    parser::parse_wlan_ip(&wlan_output.stdout)
        .ok_or_else(|| AppError::wireless_ip_not_found(serial))
}

pub async fn execute_adb_tcpip(adb_path: &Path, serial: &str, port: u16) -> Result<(), AppError> {
    let port_string = port.to_string();
    let output = ShellRunner::execute_with_timeout(
        adb_path,
        &["-s", serial, "tcpip", &port_string],
        std::time::Duration::from_secs(10),
    )
    .await?;

    if output.success {
        Ok(())
    } else {
        Err(AppError::wireless_connect_failed(&output.stderr))
    }
}

pub async fn execute_adb_connect(adb_path: &Path, endpoint: &str) -> Result<(), AppError> {
    let output = ShellRunner::execute_with_timeout(
        adb_path,
        &["connect", endpoint],
        std::time::Duration::from_secs(12),
    )
    .await?;

    let combined = format!("{}\n{}", output.stdout, output.stderr);
    if output.success && combined.to_lowercase().contains("connected") {
        Ok(())
    } else if output.success && combined.to_lowercase().contains("already connected") {
        Ok(())
    } else {
        Err(AppError::wireless_connect_failed(combined.trim()))
    }
}

pub async fn execute_adb_disconnect(adb_path: &Path, endpoint: &str) -> Result<(), AppError> {
    let output = ShellRunner::execute_with_timeout(
        adb_path,
        &["disconnect", endpoint],
        std::time::Duration::from_secs(8),
    )
    .await?;

    if output.success {
        Ok(())
    } else {
        Err(AppError::wireless_connect_failed(&output.stderr))
    }
}

pub async fn execute_adb_pair(
    adb_path: &Path,
    endpoint: &str,
    pairing_code: &str,
) -> Result<String, AppError> {
    let output = ShellRunner::execute_with_timeout(
        adb_path,
        &["pair", endpoint, pairing_code],
        std::time::Duration::from_secs(20),
    )
    .await?;

    let combined = format!("{}\n{}", output.stdout, output.stderr);
    if output.success && combined.to_lowercase().contains("success") {
        Ok(combined.trim().to_string())
    } else {
        Err(AppError::wireless_connect_failed(combined.trim()))
    }
}

pub fn map_state_to_status(state: &str) -> DeviceStatus {
    match state {
        "device" => DeviceStatus::Online,
        "offline" => DeviceStatus::Offline,
        "unauthorized" => DeviceStatus::Unauthorized,
        "recovery" | "fastboot" => DeviceStatus::Busy,
        _ => DeviceStatus::Unknown,
    }
}

pub fn detect_connection_type(serial: &str) -> ConnectionType {
    if serial.contains(':') && serial.contains('.') {
        ConnectionType::Wifi
    } else {
        ConnectionType::Usb
    }
}

pub fn default_android_capabilities() -> Vec<DeviceCapability> {
    vec![
        DeviceCapability::Mirror,
        DeviceCapability::Control,
        DeviceCapability::Screenshot,
        DeviceCapability::Recording,
        DeviceCapability::Wireless,
        DeviceCapability::InstallApp,
        DeviceCapability::UninstallApp,
        DeviceCapability::Logs,
    ]
}

pub fn raw_device_to_info(raw: &RawDevice) -> DeviceInfo {
    let status = map_state_to_status(&raw.state);
    let connection_type = detect_connection_type(&raw.serial);

    let capabilities = if status == DeviceStatus::Online {
        default_android_capabilities()
    } else {
        vec![]
    };

    let name = raw.model.clone().unwrap_or_else(|| raw.serial.clone());

    DeviceInfo {
        id: raw.serial.clone(),
        serial: raw.serial.clone(),
        name,
        model: raw.model.clone().unwrap_or_default(),
        brand: String::new(),
        platform: DevicePlatform::Android,
        status,
        connection_type,
        android_version: None,
        screen_size: None,
        battery_level: None,
        capabilities,
    }
}
