mod commands;
mod config;
mod core;
mod providers;
mod repositories;
mod services;
mod sidecar;

use std::sync::Arc;

use tauri::Manager;

use core::app_state::AppState;
use core::log_bus::LogBus;
use core::process_manager::ProcessManager;
use providers::android::provider::AndroidProvider;
use repositories::database::Database;
use repositories::settings::SettingsRepository;
use services::device::DeviceService;
use services::environment::EnvironmentService;
use services::mirror::MirrorService;
use services::settings::SettingsService;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let data_dir = app.path().app_data_dir().expect("无法获取 app data 目录");

            let db = Arc::new(Database::open(&data_dir).expect("无法初始化数据库"));

            let settings_repo = SettingsRepository::new(&data_dir);
            let settings = settings_repo.load().unwrap_or_default();

            let log_bus = Arc::new(LogBus::new(app.handle().clone(), db.clone()));
            let process_manager = Arc::new(ProcessManager::new(log_bus.clone(), db.clone()));

            let android_provider = Arc::new(AndroidProvider::new(settings.clone()));

            let environment_service = EnvironmentService::new(android_provider.clone());
            let device_service = DeviceService::new(android_provider.clone());
            let mirror_service = MirrorService::new(
                android_provider.clone(),
                process_manager,
                log_bus.clone(),
                settings,
            );
            let settings_service = SettingsService::new(settings_repo);

            let app_state = AppState::new(db, log_bus);

            app.manage(app_state);
            app.manage(android_provider.clone());
            app.manage(environment_service);
            app.manage(device_service);
            app.manage(mirror_service);
            app.manage(settings_service);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::environment::check_environment,
            commands::device::scan_devices,
            commands::device::get_device_detail,
            commands::device::enable_wireless_device,
            commands::device::connect_wireless_device,
            commands::device::discover_wireless_devices,
            commands::device::pair_wireless_device,
            commands::device::disconnect_wireless_device,
            commands::mirror::start_mirror,
            commands::mirror::start_wireless_mirror,
            commands::mirror::connect_wireless_and_start_mirror,
            commands::mirror::stop_mirror,
            commands::mirror::list_mirror_sessions,
            commands::log::get_recent_logs,
            commands::log::clear_logs,
            commands::settings::get_settings,
            commands::settings::update_settings,
            commands::settings::reset_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
