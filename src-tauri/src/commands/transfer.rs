use crate::core::error::AppError;
use crate::core::types::{DeviceActionResult, FileEntry, WifiTransferStatus};
use crate::services::transfer::TransferService;

#[tauri::command]
pub async fn list_device_directory(
    transfer_service: tauri::State<'_, TransferService>,
    serial: String,
    path: String,
) -> Result<Vec<FileEntry>, AppError> {
    transfer_service.list_directory(&serial, &path).await
}

#[tauri::command]
pub async fn pull_device_file(
    transfer_service: tauri::State<'_, TransferService>,
    serial: String,
    remote_path: String,
    local_directory: String,
) -> Result<DeviceActionResult, AppError> {
    transfer_service
        .pull_file(&serial, &remote_path, &local_directory)
        .await
}

#[tauri::command]
pub async fn delete_device_file(
    transfer_service: tauri::State<'_, TransferService>,
    serial: String,
    path: String,
) -> Result<DeviceActionResult, AppError> {
    transfer_service.delete_device_file(&serial, &path).await
}

#[tauri::command]
pub async fn create_device_directory(
    transfer_service: tauri::State<'_, TransferService>,
    serial: String,
    path: String,
) -> Result<DeviceActionResult, AppError> {
    transfer_service.create_directory(&serial, &path).await
}

#[tauri::command]
pub async fn create_device_file(
    transfer_service: tauri::State<'_, TransferService>,
    serial: String,
    path: String,
) -> Result<DeviceActionResult, AppError> {
    transfer_service.create_file(&serial, &path).await
}

#[tauri::command]
pub async fn push_device_file_streaming(
    transfer_service: tauri::State<'_, TransferService>,
    app_handle: tauri::AppHandle,
    serial: String,
    local_path: String,
    remote_directory: String,
) -> Result<DeviceActionResult, AppError> {
    let adb_path = transfer_service.adb_path()?;
    crate::providers::android::adb::execute_push_file_streaming(
        &adb_path,
        &serial,
        &local_path,
        &remote_directory,
        &app_handle,
    )
    .await
}

#[tauri::command]
pub async fn pull_device_file_streaming(
    transfer_service: tauri::State<'_, TransferService>,
    app_handle: tauri::AppHandle,
    serial: String,
    remote_path: String,
    local_directory: String,
) -> Result<DeviceActionResult, AppError> {
    let adb_path = transfer_service.adb_path()?;
    crate::providers::android::adb::execute_pull_file_streaming(
        &adb_path,
        &serial,
        &remote_path,
        &local_directory,
        &app_handle,
    )
    .await
}

#[tauri::command]
pub async fn start_wifi_transfer(
    transfer_service: tauri::State<'_, TransferService>,
    port: Option<u16>,
) -> Result<WifiTransferStatus, AppError> {
    crate::services::wifi_transfer::start_server(&transfer_service, port).await
}

#[tauri::command]
pub async fn stop_wifi_transfer(
    transfer_service: tauri::State<'_, TransferService>,
) -> Result<(), AppError> {
    crate::services::wifi_transfer::stop_server(&transfer_service).await
}

#[tauri::command]
pub async fn get_wifi_transfer_status(
    transfer_service: tauri::State<'_, TransferService>,
) -> Result<WifiTransferStatus, AppError> {
    Ok(transfer_service.get_wifi_transfer_status())
}
