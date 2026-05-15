mod commands;
mod config;
mod core;
mod providers;
mod repositories;
mod services;
mod sidecar;

use std::sync::Arc;

use tauri::Manager;

use core::log_bus::LogBus;
use core::process_manager::ProcessManager;
use providers::android::provider::AndroidProvider;
use repositories::database::Database;
use repositories::log::LogRepository;
use repositories::settings::SettingsRepository;
use services::device::DeviceService;
use services::environment::EnvironmentService;
use services::mirror::MirrorService;
use services::settings::SettingsService;

use core::app_state::AppState;

const AUTOSTART_HIDDEN_ARG: &str = "--devicedeck-start-hidden";

fn is_hidden_autostart_launch() -> bool {
    std::env::args().any(|arg| arg == AUTOSTART_HIDDEN_ARG)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let start_hidden = is_hidden_autostart_launch();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec![AUTOSTART_HIDDEN_ARG]),
        ))
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .setup(move |app| {
            let data_dir = app.path().app_data_dir().expect("无法获取 app data 目录");

            let db = Arc::new(Database::open(&data_dir).expect("无法初始化数据库"));

            let settings_repo = SettingsRepository::new(&data_dir);
            let settings = settings_repo.load().unwrap_or_default();
            let log_retention_days = settings.log_retention_days;

            let log_bus = Arc::new(LogBus::new(app.handle().clone(), db.clone()));
            let process_manager = Arc::new(ProcessManager::new(
                app.handle().clone(),
                log_bus.clone(),
                db.clone(),
            ));

            let android_provider = Arc::new(AndroidProvider::new(settings.clone()));

            let environment_service = EnvironmentService::new(android_provider.clone());
            let device_service = DeviceService::new(android_provider.clone());
            let mirror_service = MirrorService::new(
                android_provider.clone(),
                process_manager.clone(),
                log_bus.clone(),
                settings,
            );
            let settings_service = SettingsService::new(settings_repo);

            let app_state = AppState::new(db.clone());

            app.manage(app_state);
            app.manage(android_provider.clone());
            app.manage(environment_service);
            app.manage(device_service);
            app.manage(mirror_service);
            app.manage(settings_service);

            // 启动时清理旧日志
            let log_repo = LogRepository::new(&db);
            if let Err(e) = log_repo.cleanup_old_logs(log_retention_days) {
                eprintln!("清理旧日志失败: {e}");
            }

            // 启动日志
            log_bus.system_info(&format!("DeviceDeck v{} 启动", config::APP_VERSION));

            // 窗口关闭时杀掉所有 scrcpy 进程
            if let Some(window) = app.get_webview_window("main") {
                let pm = process_manager.clone();
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { .. } = event {
                        let pm = pm.clone();
                        tokio::spawn(async move { pm.kill_all().await });
                    }
                });
            }

            if start_hidden {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.hide();
                }
            }

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
            commands::device::detect_device_capabilities,
            commands::device::take_device_screenshot,
            commands::device::install_device_apk,
            commands::device::push_device_file,
            commands::device::run_device_key_action,
            commands::device::run_adb_shell_command,
            commands::mirror::start_mirror,
            commands::mirror::start_wireless_mirror,
            commands::mirror::connect_wireless_and_start_mirror,
            commands::mirror::stop_mirror,
            commands::mirror::list_mirror_sessions,
            commands::log::get_recent_logs,
            commands::log::get_logs_paginated,
            commands::log::clear_logs,
            commands::settings::get_settings,
            commands::settings::update_settings,
            commands::settings::reset_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
