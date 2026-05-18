use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::Mutex;

use super::error::AppError;
use super::log_bus::LogBus;
use super::types::{MirrorConfig, MirrorSession, SessionStatus};
use crate::repositories::database::Database;
use crate::repositories::session::SessionRepository;
use crate::sidecar::process_command;

struct ManagedProcess {
    device_serial: String,
    process_id: u32,
}

pub struct ProcessManager {
    processes: Arc<Mutex<HashMap<String, ManagedProcess>>>,
    log_bus: Arc<LogBus>,
    db: Arc<Database>,
    app_handle: AppHandle,
}

impl ProcessManager {
    pub fn new(app_handle: AppHandle, log_bus: Arc<LogBus>, db: Arc<Database>) -> Self {
        Self {
            processes: Arc::new(Mutex::new(HashMap::new())),
            log_bus,
            db,
            app_handle,
        }
    }

    pub async fn spawn(
        &self,
        session_id: &str,
        device_serial: &str,
        scrcpy_path: PathBuf,
        args: Vec<String>,
        config: MirrorConfig,
    ) -> Result<MirrorSession, AppError> {
        let working_dir = resolve_working_dir(&scrcpy_path);

        let mut child = process_command::new_tokio_command(&scrcpy_path)
            .args(&args)
            .current_dir(&working_dir)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| AppError::mirror_start_failed(&e.to_string()))?;

        let pid = child
            .id()
            .ok_or_else(|| AppError::mirror_start_failed("无法获取 scrcpy 进程 ID"))?;

        self.log_bus.scrcpy_info(
            device_serial,
            &format!(
                "scrcpy 启动: pid={pid}, path={}, cwd={}, args={}",
                scrcpy_path.display(),
                working_dir.display(),
                args.join(" ")
            ),
        );

        {
            let mut processes = self.processes.lock().await;
            processes.insert(
                session_id.into(),
                ManagedProcess {
                    device_serial: device_serial.into(),
                    process_id: pid,
                },
            );
        }

        let sid = session_id.to_string();
        let serial = device_serial.to_string();
        let stdout = child.stdout.take();
        let stderr = child.stderr.take();
        let bus = self.log_bus.clone();
        let db = self.db.clone();
        let processes = self.processes.clone();
        let app_handle = self.app_handle.clone();

        tokio::spawn(async move {
            let stdout_task = stdout.map(|stream| {
                let bus = bus.clone();
                let serial = serial.clone();
                tokio::spawn(async move {
                    let mut lines = BufReader::new(stream).lines();
                    while let Ok(Some(line)) = lines.next_line().await {
                        bus.scrcpy_info(&serial, &line);
                    }
                })
            });

            let stderr_task = stderr.map(|stream| {
                let bus = bus.clone();
                let serial = serial.clone();
                tokio::spawn(async move {
                    let mut lines = BufReader::new(stream).lines();
                    while let Ok(Some(line)) = lines.next_line().await {
                        let upper = line.to_uppercase();
                        if upper.contains("ERROR:")
                            || upper.contains("FATAL:")
                            || upper.contains("FAILED")
                        {
                            bus.scrcpy_error(&serial, &line);
                        } else {
                            bus.scrcpy_info(&serial, &line);
                        }
                    }
                })
            });

            let status = child.wait().await;

            if let Some(task) = stdout_task {
                let _ = task.await;
            }
            if let Some(task) = stderr_task {
                let _ = task.await;
            }

            let exit_ok = status.as_ref().map(|s| s.success()).unwrap_or(false);
            let exit_desc = match &status {
                Ok(status) => format!("code={:?}", status.code()),
                Err(error) => format!("wait_error={error}"),
            };
            if exit_ok {
                bus.scrcpy_info(&serial, &format!("scrcpy 进程已退出 ({exit_desc})"));
            } else {
                bus.scrcpy_error(&serial, &format!("scrcpy 进程异常退出 ({exit_desc})"));
            }

            let repo = SessionRepository::new(&db);
            let status = if exit_ok {
                SessionStatus::Stopped
            } else {
                SessionStatus::Failed
            };
            if let Err(error) = repo.update_session_status(&sid, status, Some(now_millis())) {
                bus.scrcpy_error(&serial, &format!("更新会话状态失败: {error}"));
            }

            if let Err(e) = app_handle.emit("mirror://session-updated", ()) {
                eprintln!("Failed to emit session-updated event: {e}");
            }

            let mut processes = processes.lock().await;
            processes.remove(&sid);
        });

        Ok(MirrorSession {
            id: session_id.into(),
            device_serial: device_serial.into(),
            platform: "android".into(),
            process_id: Some(pid),
            status: SessionStatus::Running,
            started_at: now_millis(),
            stopped_at: None,
            config,
        })
    }

    pub async fn stop(&self, session_id: &str) -> Result<(), AppError> {
        let managed = {
            let mut processes = self.processes.lock().await;
            processes.remove(session_id)
        };

        let Some(process) = managed else {
            return Err(AppError::mirror_stop_failed(&format!(
                "Session {session_id} not found"
            )));
        };

        kill_process_tree(process.process_id).await?;
        self.log_bus
            .scrcpy_info(&process.device_serial, "投屏已停止");
        Ok(())
    }

    pub async fn has_session(&self, session_id: &str) -> bool {
        let processes = self.processes.lock().await;
        processes.contains_key(session_id)
    }

    pub async fn is_running(&self, device_serial: &str) -> bool {
        let processes = self.processes.lock().await;
        processes
            .values()
            .any(|process| process.device_serial == device_serial)
    }

    pub async fn kill_all(&self) {
        let mut processes = self.processes.lock().await;
        for (_, process) in processes.drain() {
            let _ = kill_process_tree(process.process_id).await;
            self.log_bus
                .scrcpy_info(&process.device_serial, "应用关闭，投屏已停止");
        }
    }
}

#[cfg(windows)]
async fn kill_process_tree(pid: u32) -> Result<(), AppError> {
    let output = process_command::new_tokio_command("taskkill")
        .args(["/F", "/T", "/PID", &pid.to_string()])
        .output()
        .await
        .map_err(|e| AppError::mirror_stop_failed(&e.to_string()))?;

    if output.status.success() {
        Ok(())
    } else {
        Err(AppError::mirror_stop_failed(&String::from_utf8_lossy(
            &output.stderr,
        )))
    }
}

#[cfg(not(windows))]
async fn kill_process_tree(pid: u32) -> Result<(), AppError> {
    let output = process_command::new_tokio_command("kill")
        .args(["-TERM", &pid.to_string()])
        .output()
        .await
        .map_err(|e| AppError::mirror_stop_failed(&e.to_string()))?;

    if output.status.success() {
        Ok(())
    } else {
        Err(AppError::mirror_stop_failed(&String::from_utf8_lossy(
            &output.stderr,
        )))
    }
}

fn now_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

fn resolve_working_dir(program_path: &std::path::Path) -> PathBuf {
    let program_dir = program_path
        .parent()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));

    if program_dir.join("scrcpy-server").is_file() {
        return program_dir;
    }

    let candidates = [
        program_dir
            .parent()
            .map(PathBuf::from)
            .unwrap_or_else(|| program_dir.clone()),
        program_dir.join("binaries"),
        program_dir.join("resources").join("binaries"),
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("binaries"),
    ];

    candidates
        .into_iter()
        .find(|path| path.join("scrcpy-server").is_file())
        .unwrap_or(program_dir)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_working_dir_uses_program_dir_when_server_is_next_to_scrcpy() {
        let root = unique_temp_dir("same-dir-server");
        let program_dir = root.join("binaries").join("macos-aarch64");
        std::fs::create_dir_all(&program_dir).unwrap();
        std::fs::write(program_dir.join("scrcpy-server"), b"server").unwrap();

        assert_eq!(
            resolve_working_dir(&program_dir.join("scrcpy")),
            program_dir
        );

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn resolve_working_dir_uses_parent_binaries_dir_for_shared_server() {
        let root = unique_temp_dir("shared-server");
        let binaries_dir = root.join("binaries");
        let program_dir = binaries_dir.join("linux-x64");
        std::fs::create_dir_all(&program_dir).unwrap();
        std::fs::write(binaries_dir.join("scrcpy-server"), b"server").unwrap();

        assert_eq!(
            resolve_working_dir(&program_dir.join("scrcpy")),
            binaries_dir
        );

        let _ = std::fs::remove_dir_all(root);
    }

    fn unique_temp_dir(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "devicedeck-{name}-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }
}
