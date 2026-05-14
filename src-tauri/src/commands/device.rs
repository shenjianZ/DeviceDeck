use crate::core::error::AppError;
use crate::core::types::DeviceInfo;
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
