use crate::core::error::AppError;
use crate::core::types::{DeviceActionResult, DeviceInfo, DeviceKeyAction, RecommendedConfig};
use crate::providers::android::types::WirelessAdbService;
use crate::services::device::DeviceService;

#[tauri::command]
pub async fn scan_devices(
    device_service: tauri::State<'_, DeviceService>,
) -> Result<Vec<DeviceInfo>, AppError> {
    device_service.scan_devices().await
}

#[tauri::command]
pub async fn get_device_detail(
    device_service: tauri::State<'_, DeviceService>,
    serial: String,
) -> Result<DeviceInfo, AppError> {
    device_service.get_device_detail(&serial).await
}

#[tauri::command]
pub async fn enable_wireless_device(
    device_service: tauri::State<'_, DeviceService>,
    serial: String,
    port: u16,
) -> Result<DeviceInfo, AppError> {
    device_service.enable_wireless_device(&serial, port).await
}

#[tauri::command]
pub async fn connect_wireless_device(
    device_service: tauri::State<'_, DeviceService>,
    host: String,
    port: u16,
) -> Result<DeviceInfo, AppError> {
    device_service.connect_wireless_device(&host, port).await
}

#[tauri::command]
pub async fn discover_wireless_devices(
    device_service: tauri::State<'_, DeviceService>,
) -> Result<Vec<WirelessAdbService>, AppError> {
    device_service.discover_wireless_services().await
}

#[tauri::command]
pub async fn pair_wireless_device(
    device_service: tauri::State<'_, DeviceService>,
    host: String,
    port: u16,
    pairing_code: String,
) -> Result<String, AppError> {
    device_service
        .pair_wireless_device(&host, port, &pairing_code)
        .await
}

#[tauri::command]
pub async fn disconnect_wireless_device(
    device_service: tauri::State<'_, DeviceService>,
    serial: String,
) -> Result<(), AppError> {
    device_service.disconnect_wireless_device(&serial).await
}

#[tauri::command]
pub async fn detect_device_capabilities(
    device_service: tauri::State<'_, DeviceService>,
    serial: String,
) -> Result<Vec<RecommendedConfig>, AppError> {
    device_service.detect_capabilities(&serial).await
}

#[tauri::command]
pub async fn take_device_screenshot(
    device_service: tauri::State<'_, DeviceService>,
    serial: String,
    output_directory: Option<String>,
) -> Result<DeviceActionResult, AppError> {
    device_service
        .take_screenshot(&serial, output_directory.as_deref())
        .await
}

#[tauri::command]
pub async fn install_device_apk(
    device_service: tauri::State<'_, DeviceService>,
    serial: String,
    apk_path: String,
) -> Result<DeviceActionResult, AppError> {
    device_service.install_apk(&serial, &apk_path).await
}

#[tauri::command]
pub async fn push_device_file(
    device_service: tauri::State<'_, DeviceService>,
    serial: String,
    local_path: String,
    remote_directory: String,
) -> Result<DeviceActionResult, AppError> {
    device_service
        .push_file(&serial, &local_path, &remote_directory)
        .await
}

#[tauri::command]
pub async fn run_device_key_action(
    device_service: tauri::State<'_, DeviceService>,
    serial: String,
    action: DeviceKeyAction,
) -> Result<DeviceActionResult, AppError> {
    device_service.run_key_action(&serial, action).await
}

#[tauri::command]
pub async fn run_adb_shell_command(
    device_service: tauri::State<'_, DeviceService>,
    serial: String,
    command: String,
    timeout_ms: Option<u64>,
) -> Result<DeviceActionResult, AppError> {
    device_service
        .run_shell_command(&serial, &command, timeout_ms)
        .await
}
