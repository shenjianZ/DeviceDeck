use std::sync::Arc;

use crate::core::error::AppError;
use crate::core::log_bus::LogBus;
use crate::core::process_manager::ProcessManager;
use crate::core::types::AppSettings;
use crate::core::types::{MirrorConfig, MirrorSession, SessionStatus};
use crate::providers::android::provider::AndroidProvider;
use crate::providers::android::scrcpy;
use crate::providers::provider_trait::DeviceProvider;
use crate::repositories::session::SessionRepository;
use crate::sidecar::binary_resolver::BinaryResolver;
use uuid::Uuid;

pub struct MirrorService {
    android_provider: Arc<AndroidProvider>,
    process_manager: Arc<ProcessManager>,
    log_bus: Arc<LogBus>,
    settings: std::sync::Mutex<AppSettings>,
}

impl MirrorService {
    pub fn new(
        android_provider: Arc<AndroidProvider>,
        process_manager: Arc<ProcessManager>,
        log_bus: Arc<LogBus>,
        settings: AppSettings,
    ) -> Self {
        Self {
            android_provider,
            process_manager,
            log_bus,
            settings: std::sync::Mutex::new(settings),
        }
    }

    pub fn update_settings(&self, settings: AppSettings) -> Result<(), AppError> {
        let mut current = self
            .settings
            .lock()
            .map_err(|e| AppError::internal_error(&e.to_string()))?;
        *current = settings;
        Ok(())
    }

    pub async fn start_mirror(
        &self,
        serial: &str,
        config: &MirrorConfig,
        session_repo: &SessionRepository<'_>,
    ) -> Result<MirrorSession, AppError> {
        if self.process_manager.is_running(serial).await {
            return Err(AppError::mirror_already_running(serial));
        }

        let settings = self
            .settings
            .lock()
            .map_err(|e| AppError::internal_error(&e.to_string()))?
            .clone();
        let scrcpy_path = BinaryResolver::resolve_scrcpy(&settings)?;
        let _device = self.android_provider.get_device_detail(serial).await?;
        let args = scrcpy::build_scrcpy_args(serial, config)?;
        let session_id = Uuid::new_v4().to_string();

        let session = self
            .process_manager
            .spawn(&session_id, serial, scrcpy_path, args, config.clone())
            .await?;

        session_repo.save_session(&session)?;

        self.log_bus.scrcpy_info(
            serial,
            &format!(
                "Mirror started — {}p / {} / {}fps",
                config.max_size, config.video_bit_rate, config.max_fps
            ),
        );

        Ok(session)
    }

    pub async fn start_wireless_mirror(
        &self,
        serial: &str,
        config: &MirrorConfig,
        port: u16,
        session_repo: &SessionRepository<'_>,
    ) -> Result<MirrorSession, AppError> {
        let wireless_device = if serial.contains(':') {
            self.android_provider.get_device_detail(serial).await?
        } else {
            self.log_bus
                .adb_info(serial, &format!("Enabling wireless debugging port {port}"));
            self.android_provider
                .enable_wireless_device(serial, port)
                .await?
        };

        self.log_bus.adb_info(
            &wireless_device.serial,
            "Wireless ADB connected, starting mirror",
        );
        self.start_mirror(&wireless_device.serial, config, session_repo)
            .await
    }

    pub async fn connect_wireless_and_start_mirror(
        &self,
        host: &str,
        port: u16,
        config: &MirrorConfig,
        session_repo: &SessionRepository<'_>,
    ) -> Result<MirrorSession, AppError> {
        let wireless_device = self
            .android_provider
            .connect_wireless_device(host, port)
            .await?;

        self.start_mirror(&wireless_device.serial, config, session_repo)
            .await
    }

    pub async fn stop_mirror(
        &self,
        session_id: &str,
        session_repo: &SessionRepository<'_>,
    ) -> Result<(), AppError> {
        if self.process_manager.has_session(session_id).await {
            self.process_manager.stop(session_id).await?;
        } else {
            self.log_bus.scrcpy_info(
                "",
                &format!("Session {session_id} has no running scrcpy process, synced to stopped"),
            );
        }

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        session_repo.update_session_status(session_id, SessionStatus::Stopped, Some(now))?;

        Ok(())
    }

    pub async fn list_sessions(
        &self,
        session_repo: &SessionRepository<'_>,
    ) -> Result<Vec<MirrorSession>, AppError> {
        let mut sessions = session_repo.get_all_sessions()?;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        for session in &mut sessions {
            if session.status == SessionStatus::Running
                && !self.process_manager.has_session(&session.id).await
            {
                session.status = SessionStatus::Failed;
                session.stopped_at = Some(now);
                session_repo.update_session_status(
                    &session.id,
                    SessionStatus::Failed,
                    Some(now),
                )?;
            }
        }

        Ok(sessions)
    }
}
