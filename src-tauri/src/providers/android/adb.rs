use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};
use std::time::{Duration, Instant};

use super::parser;
use super::types::{AndroidDeviceProps, RawDevice, WirelessAdbService};
use crate::core::error::AppError;
use crate::core::types::{
    ConnectionType, DeviceActionResult, DeviceCapability, DeviceInfo, DeviceKeyAction,
    DevicePlatform, DeviceStatus, FileEntry, TransferProgress,
};
use crate::sidecar::process_command;
use crate::sidecar::shell_runner::ShellRunner;

const DEFAULT_PUSH_DIRECTORY: &str = "/sdcard/Download/DeviceDeck";
const TRANSFER_PROGRESS_INTERVAL: u64 = 262_144;

pub async fn execute_adb_devices(adb_path: &Path) -> Result<Vec<RawDevice>, AppError> {
    let output = ShellRunner::execute(adb_path, &["devices", "-l"]).await?;
    if !output.success {
        return Err(AppError::internal_error(&format!(
            "adb devices failed: {}",
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

pub async fn execute_screenshot(
    adb_path: &Path,
    serial: &str,
    output_directory: Option<&str>,
) -> Result<DeviceActionResult, AppError> {
    validate_device_serial(serial)?;
    let directory = resolve_output_directory(output_directory)?;
    tokio::fs::create_dir_all(&directory).await?;
    let file_name = format!(
        "DeviceDeck-{}-{}.png",
        sanitize_file_part(serial),
        timestamp()
    );
    let output_path = directory.join(file_name);

    let output = tokio::time::timeout(
        std::time::Duration::from_secs(15),
        process_command::new_tokio_command(adb_path)
            .args(["-s", serial, "exec-out", "screencap", "-p"])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .output(),
    )
    .await
    .map_err(|_| AppError::internal_error("Screenshot command timed out"))?
    .map_err(|e| AppError::internal_error(&format!("Screenshot command failed: {e}")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(AppError::internal_error(&format!(
            "Screenshot failed: {}",
            stderr.trim()
        )));
    }

    tokio::fs::write(&output_path, output.stdout).await?;

    Ok(DeviceActionResult {
        message: "Screenshot saved".into(),
        output_path: Some(output_path.to_string_lossy().into_owned()),
        stdout: None,
        stderr: None,
    })
}

pub async fn execute_install_apk(
    adb_path: &Path,
    serial: &str,
    apk_path: &str,
) -> Result<DeviceActionResult, AppError> {
    validate_device_serial(serial)?;
    let apk = PathBuf::from(apk_path);
    if !apk.is_file() || apk.extension().and_then(|value| value.to_str()) != Some("apk") {
        return Err(AppError::invalid_config("Please select a valid .apk file"));
    }
    let apk_arg = apk.to_string_lossy().into_owned();
    let output = ShellRunner::execute_with_timeout(
        adb_path,
        &["-s", serial, "install", "-r", &apk_arg],
        std::time::Duration::from_secs(120),
    )
    .await?;
    adb_action_result("APK installed", output)
}

pub async fn execute_push_file(
    adb_path: &Path,
    serial: &str,
    local_path: &str,
    remote_directory: &str,
) -> Result<DeviceActionResult, AppError> {
    validate_device_serial(serial)?;
    let local = PathBuf::from(local_path);
    if !local.is_file() {
        return Err(AppError::invalid_config("Please select a valid local file"));
    }
    let remote_directory = normalize_remote_push_directory(remote_directory)?;
    let mkdir_output = ShellRunner::execute_with_timeout(
        adb_path,
        &[
            "-s",
            serial,
            "shell",
            &format!("mkdir -p {}", remote_shell_quote(remote_directory)),
        ],
        std::time::Duration::from_secs(10),
    )
    .await?;
    adb_action_result("Remote directory created", mkdir_output)?;

    let remote = build_remote_push_target(&local, remote_directory)?;
    let local_arg = local.to_string_lossy().into_owned();
    let output = ShellRunner::execute_with_timeout(
        adb_path,
        &["-s", serial, "push", &local_arg, &remote],
        std::time::Duration::from_secs(120),
    )
    .await?;
    let mut result = adb_action_result("File sent", output)?;
    result.output_path = Some(remote.clone());
    result.stderr = merge_optional_output(
        result.stderr,
        refresh_android_file_index(adb_path, serial, &remote)
            .await
            .err(),
    );
    Ok(result)
}

pub async fn execute_key_action(
    adb_path: &Path,
    serial: &str,
    action: DeviceKeyAction,
) -> Result<DeviceActionResult, AppError> {
    validate_device_serial(serial)?;
    let timeout = std::time::Duration::from_secs(10);

    match action {
        DeviceKeyAction::ScreenBlack => {
            let primary = vec!["-s", serial, "shell", "cmd", "display", "power-off"];
            let result = ShellRunner::execute_with_timeout(adb_path, &primary, timeout).await;
            if let Ok(output) = result {
                if output.success {
                    return adb_action_result("Screen turned off", output);
                }
            }
            let fallback = vec![
                "-s",
                serial,
                "shell",
                "settings",
                "put",
                "system",
                "screen_brightness",
                "0",
            ];
            let output = ShellRunner::execute_with_timeout(adb_path, &fallback, timeout).await?;
            adb_action_result("Screen brightness set to minimum", output)
        }
        DeviceKeyAction::ScreenRestore => {
            let primary = vec!["-s", serial, "shell", "cmd", "display", "power-on"];
            let result = ShellRunner::execute_with_timeout(adb_path, &primary, timeout).await;
            if let Ok(output) = result {
                if output.success {
                    return adb_action_result("Screen restored", output);
                }
            }
            let fallback = vec![
                "-s",
                serial,
                "shell",
                "settings",
                "put",
                "system",
                "screen_brightness_mode",
                "1",
            ];
            let output = ShellRunner::execute_with_timeout(adb_path, &fallback, timeout).await?;
            adb_action_result("Screen brightness restored", output)
        }
        _ => {
            let args = match action {
                DeviceKeyAction::Home => {
                    vec!["-s", serial, "shell", "input", "keyevent", "HOME"]
                }
                DeviceKeyAction::Back => {
                    vec!["-s", serial, "shell", "input", "keyevent", "BACK"]
                }
                DeviceKeyAction::AppSwitch => {
                    vec!["-s", serial, "shell", "input", "keyevent", "APP_SWITCH"]
                }
                DeviceKeyAction::Menu => {
                    vec!["-s", serial, "shell", "input", "keyevent", "MENU"]
                }
                DeviceKeyAction::Power => {
                    vec!["-s", serial, "shell", "input", "keyevent", "POWER"]
                }
                DeviceKeyAction::VolumeUp => {
                    vec!["-s", serial, "shell", "input", "keyevent", "VOLUME_UP"]
                }
                DeviceKeyAction::VolumeDown => {
                    vec!["-s", serial, "shell", "input", "keyevent", "VOLUME_DOWN"]
                }
                DeviceKeyAction::ExpandNotifications => vec![
                    "-s",
                    serial,
                    "shell",
                    "cmd",
                    "statusbar",
                    "expand-notifications",
                ],
                DeviceKeyAction::CollapseNotifications => {
                    vec!["-s", serial, "shell", "cmd", "statusbar", "collapse"]
                }
                DeviceKeyAction::TurnScreenOff => {
                    vec!["-s", serial, "shell", "input", "keyevent", "SLEEP"]
                }
                _ => unreachable!(),
            };
            let output = ShellRunner::execute_with_timeout(adb_path, &args, timeout).await?;
            adb_action_result("Key action completed", output)
        }
    }
}

pub async fn execute_shell_command(
    adb_path: &Path,
    serial: &str,
    command: &str,
    timeout_ms: Option<u64>,
) -> Result<DeviceActionResult, AppError> {
    validate_device_serial(serial)?;
    let command = command.trim();
    if command.is_empty() {
        return Err(AppError::invalid_config(
            "ADB shell command cannot be empty",
        ));
    }
    let timeout =
        std::time::Duration::from_millis(timeout_ms.unwrap_or(30_000).clamp(1_000, 120_000));
    let output =
        ShellRunner::execute_with_timeout(adb_path, &["-s", serial, "shell", command], timeout)
            .await?;
    adb_action_result("ADB shell command executed", output)
}

pub async fn execute_list_directory(
    adb_path: &Path,
    serial: &str,
    remote_path: &str,
) -> Result<Vec<FileEntry>, AppError> {
    validate_device_serial(serial)?;
    let remote_path = remote_path.trim();
    if remote_path.is_empty() {
        return Err(AppError::invalid_config("Directory path cannot be empty"));
    }
    // Ensure trailing slash so ls follows symlinks (e.g. /sdcard -> /storage/self/primary)
    let path_arg = if remote_path.ends_with('/') {
        remote_path.to_string()
    } else {
        format!("{}/", remote_path)
    };
    let output = ShellRunner::execute_with_timeout(
        adb_path,
        &[
            "-s",
            serial,
            "shell",
            &format!("ls -la {}", remote_shell_quote(&path_arg)),
        ],
        std::time::Duration::from_secs(15),
    )
    .await
    .map_err(|e| {
        e.with_suggestion("Make sure the device is connected and USB debugging is enabled")
    })?;

    if !output.success {
        let detail = command_output_detail(&output);
        return Err(
            AppError::internal_error(&format!("ls -la failed: {}", detail))
                .with_suggestion("Check if the path exists and is accessible on the device"),
        );
    }

    let entries = parser::parse_ls_la(&output.stdout, remote_path);

    if entries.is_empty() {
        let combined = format!("{}\n{}", output.stdout, output.stderr).to_lowercase();
        if combined.contains("no such file")
            || combined.contains("not found")
            || combined.contains("does not exist")
        {
            return Err(AppError::internal_error(&format!(
                "Directory does not exist: {}",
                remote_path
            ))
            .with_suggestion("Try browsing from /sdcard or /storage"));
        }
        if combined.contains("permission denied") || combined.contains("not a directory") {
            return Err(
                AppError::internal_error(&format!("Cannot access: {}", remote_path))
                    .with_suggestion("This directory may require root access or does not exist"),
            );
        }
    }

    Ok(entries)
}

pub async fn execute_pull_file(
    adb_path: &Path,
    serial: &str,
    remote_path: &str,
    local_directory: &str,
) -> Result<DeviceActionResult, AppError> {
    validate_device_serial(serial)?;
    let remote = remote_path.trim();
    if remote.is_empty() {
        return Err(AppError::invalid_config("Remote file path cannot be empty"));
    }
    let local_dir = PathBuf::from(local_directory.trim());
    if !local_dir.is_dir() {
        tokio::fs::create_dir_all(&local_dir).await?;
    }

    // Check if remote path is a file or directory
    let is_dir = is_remote_directory(adb_path, serial, remote).await;

    if is_dir {
        pull_directory_recursive(adb_path, serial, remote, &local_dir, None).await
    } else {
        pull_single_file(adb_path, serial, remote, &local_dir).await
    }
}

/// Pull a single file using exec-out cat, streaming to disk (avoids buffering entire file in memory)
async fn pull_single_file(
    adb_path: &Path,
    serial: &str,
    remote: &str,
    local_dir: &Path,
) -> Result<DeviceActionResult, AppError> {
    let file_name = remote.rsplit('/').next().unwrap_or(remote).to_string();
    let local_file = local_dir.join(&file_name);

    stream_cat_to_file(adb_path, serial, remote, &local_file).await?;
    sync_and_refresh(&local_file).await;

    Ok(DeviceActionResult {
        message: "File pulled".into(),
        output_path: Some(local_file.to_string_lossy().into_owned()),
        stdout: None,
        stderr: None,
    })
}

/// Pull a directory using a single tar stream (much faster than per-file adb calls)
async fn pull_directory_recursive(
    adb_path: &Path,
    serial: &str,
    remote_dir: &str,
    local_dir: &Path,
    app_handle: Option<&tauri::AppHandle>,
) -> Result<DeviceActionResult, AppError> {
    let dir_name = remote_dir
        .trim_end_matches('/')
        .rsplit('/')
        .next()
        .unwrap_or("folder")
        .to_string();
    let target_dir = local_dir.join(&dir_name);
    tokio::fs::create_dir_all(&target_dir).await?;

    // Strategy: stream tar from device via `adb exec-out tar cf -`, then extract locally.
    // Fallback to per-file pull if tar is unavailable on device
    let total_bytes = remote_directory_size(adb_path, serial, remote_dir)
        .await
        .unwrap_or(0);
    let tar_result = pull_via_tar_stream(
        adb_path,
        serial,
        remote_dir,
        &dir_name,
        &target_dir,
        total_bytes,
        app_handle,
    )
    .await;

    match tar_result {
        Ok(count) => {
            sync_and_refresh(&target_dir).await;
            Ok(DeviceActionResult {
                message: format!("Directory pulled ({} files)", count),
                output_path: Some(target_dir.to_string_lossy().into_owned()),
                stdout: None,
                stderr: None,
            })
        }
        Err(_) => {
            // Fallback: per-file pull
            let mut pulled = 0u64;
            let mut errors = Vec::new();
            pull_directory_inner(
                adb_path,
                serial,
                remote_dir,
                &target_dir,
                &mut pulled,
                &mut errors,
            )
            .await?;
            sync_and_refresh(&target_dir).await;
            let message = if errors.is_empty() {
                format!("Directory pulled ({} files)", pulled)
            } else {
                format!(
                    "Directory pulled ({} files, {} errors)",
                    pulled,
                    errors.len()
                )
            };
            Ok(DeviceActionResult {
                message,
                output_path: Some(target_dir.to_string_lossy().into_owned()),
                stdout: None,
                stderr: if errors.is_empty() {
                    None
                } else {
                    Some(errors.join("\n"))
                },
            })
        }
    }
}

/// Stream directory via `adb exec-out tar cf - <dir>` and extract with Rust tar crate.
/// Streams directly from adb stdout to tar extraction — zero memory buffering.
/// Also avoids Windows system tar codepage encoding issues with Chinese filenames.
async fn pull_via_tar_stream(
    adb_path: &Path,
    serial: &str,
    remote_dir: &str,
    file_name: &str,
    target_dir: &Path,
    total_bytes: u64,
    app_handle: Option<&tauri::AppHandle>,
) -> Result<u64, AppError> {
    // Run in spawn_blocking to use std::process (sync I/O for tar crate) without blocking async runtime
    let adb_path = adb_path.to_path_buf();
    let serial = serial.to_string();
    let remote_dir = remote_dir.to_string();
    let file_name = file_name.to_string();
    let target_dir = target_dir.to_path_buf();
    let progress = app_handle.map(|handle| {
        (
            handle.clone(),
            uuid::Uuid::new_v4().to_string(),
            Instant::now(),
        )
    });
    let transferred_bytes = Arc::new(AtomicU64::new(0));

    let result = tokio::task::spawn_blocking(move || -> Result<u64, AppError> {
        let mut adb_child = std::process::Command::new(&adb_path)
            .args([
                "-s",
                &serial,
                "exec-out",
                "sh",
                "-c",
                &format!("tar cf - -C {} .", remote_shell_quote(&remote_dir)),
            ])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| AppError::internal_error(&format!("Failed to spawn adb tar: {e}")))?;

        let stdout = adb_child.stdout.take().unwrap();
        let stderr_task = adb_child.stderr.take().map(spawn_stderr_reader);
        let reader = std::io::BufReader::new(stdout);
        let reader = ProgressReader::new(
            reader,
            progress.clone(),
            file_name.clone(),
            total_bytes,
            transferred_bytes.clone(),
        );

        // Extract using Rust tar crate — handles UTF-8 filenames correctly on all platforms
        let mut archive = tar::Archive::new(reader);
        archive.set_preserve_permissions(false);
        archive
            .unpack(&target_dir)
            .map_err(|e| AppError::internal_error(&format!("tar extract failed: {e}")))?;

        let status = adb_child
            .wait()
            .map_err(|e| AppError::internal_error(&format!("adb process error: {e}")))?;
        let stderr = stderr_task
            .and_then(|task| task.join().ok())
            .unwrap_or_default();

        if !status.success() {
            return Err(AppError::internal_error(&format!(
                "adb tar command failed on device: {}",
                stderr.trim()
            )));
        }

        // Count extracted files
        let mut count = 0u64;
        count_files_sync(&target_dir, &mut count);
        if let Some((handle, id, started_at)) = &progress {
            let transferred = if total_bytes > 0 {
                total_bytes
            } else {
                transferred_bytes.load(Ordering::Relaxed)
            };
            emit_transfer_progress(
                handle,
                id,
                &file_name,
                transferred,
                total_bytes,
                started_at,
                true,
            );
        }
        Ok(count)
    })
    .await
    .map_err(|e| AppError::internal_error(&format!("tar task failed: {e}")))??;

    Ok(result)
}

#[derive(Clone)]
struct ProgressState {
    handle: tauri::AppHandle,
    id: String,
    file_name: String,
    total: u64,
    started_at: Instant,
    transferred: u64,
    last_reported: u64,
}

impl ProgressState {
    fn new(
        handle: tauri::AppHandle,
        id: String,
        file_name: String,
        total: u64,
        started_at: Instant,
    ) -> Self {
        Self {
            handle,
            id,
            file_name,
            total,
            started_at,
            transferred: 0,
            last_reported: 0,
        }
    }

    fn add(&mut self, bytes: u64) {
        self.transferred = self.transferred.saturating_add(bytes);
        if self.transferred.saturating_sub(self.last_reported) >= TRANSFER_PROGRESS_INTERVAL {
            self.last_reported = self.transferred;
            emit_transfer_progress(
                &self.handle,
                &self.id,
                &self.file_name,
                self.transferred,
                self.total,
                &self.started_at,
                false,
            );
        }
    }
}

struct ProgressReader<R> {
    inner: R,
    progress: Option<ProgressState>,
    transferred_bytes: Arc<AtomicU64>,
}

impl<R> ProgressReader<R> {
    fn new(
        inner: R,
        progress: Option<(tauri::AppHandle, String, Instant)>,
        file_name: String,
        total: u64,
        transferred_bytes: Arc<AtomicU64>,
    ) -> Self {
        Self {
            inner,
            progress: progress.map(|(handle, id, started_at)| {
                ProgressState::new(handle, id, file_name, total, started_at)
            }),
            transferred_bytes,
        }
    }
}

impl<R: Read> Read for ProgressReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let n = self.inner.read(buf)?;
        if n > 0 {
            self.transferred_bytes
                .fetch_add(n as u64, Ordering::Relaxed);
            if let Some(progress) = &mut self.progress {
                progress.add(n as u64);
            }
        }
        Ok(n)
    }
}

fn spawn_stderr_reader<R>(mut reader: R) -> std::thread::JoinHandle<String>
where
    R: Read + Send + 'static,
{
    std::thread::spawn(move || {
        let mut stderr = String::new();
        let _ = reader.read_to_string(&mut stderr);
        stderr
    })
}

fn emit_transfer_progress(
    app_handle: &tauri::AppHandle,
    id: &str,
    file_name: &str,
    transferred: u64,
    total: u64,
    started_at: &Instant,
    complete: bool,
) {
    let percent = progress_percent(transferred, total, complete);
    let _ = tauri::Emitter::emit(
        app_handle,
        "transfer://progress",
        TransferProgress {
            id: id.to_string(),
            file_name: file_name.to_string(),
            transferred,
            total,
            percent,
            speed: format_speed(transferred, started_at.elapsed()),
        },
    );
}

fn progress_percent(transferred: u64, total: u64, complete: bool) -> u8 {
    if complete {
        return 100;
    }
    if total == 0 {
        return 0;
    }
    ((transferred.saturating_mul(100) / total).min(99)) as u8
}

fn format_speed(bytes: u64, elapsed: Duration) -> String {
    let secs = elapsed.as_secs_f64();
    if secs <= 0.0 {
        return String::new();
    }
    format!("{}/s", format_bytes((bytes as f64 / secs) as u64))
}

fn format_bytes(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = 1024.0 * KB;
    const GB: f64 = 1024.0 * MB;
    let value = bytes as f64;
    if value >= GB {
        format!("{:.1} GB", value / GB)
    } else if value >= MB {
        format!("{:.1} MB", value / MB)
    } else if value >= KB {
        format!("{:.1} KB", value / KB)
    } else {
        format!("{} B", bytes)
    }
}

fn remote_shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}

fn count_files_sync(dir: &Path, count: &mut u64) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            if let Ok(meta) = entry.metadata() {
                if meta.is_file() {
                    *count += 1;
                } else if meta.is_dir() {
                    count_files_sync(&entry.path(), count);
                }
            }
        }
    }
}

fn pull_directory_inner<'a>(
    adb_path: &'a Path,
    serial: &'a str,
    remote_dir: &'a str,
    local_dir: &'a Path,
    pulled: &'a mut u64,
    errors: &'a mut Vec<String>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), AppError>> + Send + 'a>> {
    Box::pin(async move {
        let entries = match execute_list_directory(adb_path, serial, remote_dir).await {
            Ok(e) => e,
            Err(e) => {
                errors.push(format!("Failed to list {}: {}", remote_dir, e.message));
                return Ok(());
            }
        };

        for entry in entries {
            if entry.is_directory {
                let sub_local = local_dir.join(&entry.name);
                if let Err(e) = tokio::fs::create_dir_all(&sub_local).await {
                    errors.push(format!("Failed to create dir {}: {}", entry.name, e));
                    continue;
                }
                pull_directory_inner(adb_path, serial, &entry.path, &sub_local, pulled, errors)
                    .await?;
            } else {
                match pull_single_file_inner(adb_path, serial, &entry.path, local_dir).await {
                    Ok(_) => {
                        *pulled += 1;
                    }
                    Err(e) => {
                        errors.push(format!("Failed to pull {}: {}", entry.name, e.message));
                    }
                }
            }
        }
        Ok(())
    })
}

/// Inner pull without validate/redirect boilerplate (for recursive use) — streams to disk
async fn pull_single_file_inner(
    adb_path: &Path,
    serial: &str,
    remote: &str,
    local_dir: &Path,
) -> Result<(), AppError> {
    let file_name = remote.rsplit('/').next().unwrap_or(remote).to_string();
    let local_file = local_dir.join(&file_name);
    stream_cat_to_file(adb_path, serial, remote, &local_file).await
}

/// Stream `exec-out cat <remote>` directly to a local file — zero memory buffering
async fn stream_cat_to_file(
    adb_path: &Path,
    serial: &str,
    remote: &str,
    local_file: &Path,
) -> Result<(), AppError> {
    let adb_path = adb_path.to_path_buf();
    let serial = serial.to_string();
    let remote = remote.to_string();
    let local_file = local_file.to_path_buf();

    tokio::task::spawn_blocking(move || -> Result<(), AppError> {
        let mut child = std::process::Command::new(&adb_path)
            .args([
                "-s",
                &serial,
                "exec-out",
                "sh",
                "-c",
                &format!("cat {}", remote_shell_quote(&remote)),
            ])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| AppError::internal_error(&format!("Failed to spawn adb cat: {e}")))?;

        let stdout = child.stdout.take().unwrap();
        let stderr_task = child.stderr.take().map(spawn_stderr_reader);
        let mut reader = std::io::BufReader::new(stdout);
        let mut file = std::fs::File::create(&local_file)
            .map_err(|e| AppError::internal_error(&format!("Failed to create file: {e}")))?;

        let mut buf = vec![0u8; 65536];
        loop {
            let n = match reader.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => n,
                Err(e) => return Err(AppError::internal_error(&format!("Read error: {e}"))),
            };
            file.write_all(&buf[..n])
                .map_err(|e| AppError::internal_error(&format!("Write error: {e}")))?;
        }

        let _ = file.flush();
        // Sync to disk so Windows Explorer picks it up immediately
        #[cfg(target_os = "windows")]
        {
            let _ = file.sync_data();
        }

        let status = child
            .wait()
            .map_err(|e| AppError::internal_error(&format!("Process error: {e}")))?;
        let stderr = stderr_task
            .and_then(|task| task.join().ok())
            .unwrap_or_default();
        if !status.success() {
            let _ = std::fs::remove_file(&local_file);
            return Err(AppError::internal_error(&format!(
                "Pull failed: {}",
                stderr.trim()
            )));
        }

        Ok(())
    })
    .await
    .map_err(|e| AppError::internal_error(&format!("cat task failed: {e}")))?
}

/// Check if a remote path is a directory via `ls -ld`
async fn is_remote_directory(adb_path: &Path, serial: &str, remote_path: &str) -> bool {
    let output = ShellRunner::execute_with_timeout(
        adb_path,
        &[
            "-s",
            serial,
            "shell",
            &format!("ls -ld {}", remote_shell_quote(remote_path)),
        ],
        std::time::Duration::from_secs(10),
    )
    .await;

    match output {
        Ok(o) if o.success => {
            let first_line = o.stdout.lines().next().unwrap_or("");
            first_line.trim_start().starts_with('d')
        }
        _ => false,
    }
}

async fn remote_file_size(
    adb_path: &Path,
    serial: &str,
    remote_path: &str,
) -> Result<u64, AppError> {
    let output = ShellRunner::execute_with_timeout(
        adb_path,
        &[
            "-s",
            serial,
            "shell",
            &format!("stat -c %s {}", remote_shell_quote(remote_path)),
        ],
        std::time::Duration::from_secs(10),
    )
    .await?;

    if !output.success {
        return Err(AppError::internal_error(&command_output_detail(&output)));
    }
    output
        .stdout
        .lines()
        .next()
        .unwrap_or("")
        .trim()
        .parse::<u64>()
        .map_err(|e| AppError::internal_error(&format!("Cannot parse remote file size: {e}")))
}

async fn remote_directory_size(
    adb_path: &Path,
    serial: &str,
    remote_path: &str,
) -> Result<u64, AppError> {
    let output = ShellRunner::execute_with_timeout(
        adb_path,
        &[
            "-s",
            serial,
            "shell",
            &format!("du -sb {}", remote_shell_quote(remote_path)),
        ],
        std::time::Duration::from_secs(15),
    )
    .await?;

    if !output.success {
        return Err(AppError::internal_error(&command_output_detail(&output)));
    }
    output
        .stdout
        .split_whitespace()
        .next()
        .unwrap_or("")
        .parse::<u64>()
        .map_err(|e| AppError::internal_error(&format!("Cannot parse remote directory size: {e}")))
}

pub async fn execute_delete_file(
    adb_path: &Path,
    serial: &str,
    remote_path: &str,
) -> Result<DeviceActionResult, AppError> {
    validate_device_serial(serial)?;
    let remote = remote_path.trim();
    if remote.is_empty() {
        return Err(AppError::invalid_config("File path cannot be empty"));
    }
    if remote == "/" || remote == "/sdcard" || remote == "/storage" {
        return Err(AppError::invalid_config("Cannot delete system directories"));
    }
    let output = ShellRunner::execute_with_timeout(
        adb_path,
        &[
            "-s",
            serial,
            "shell",
            &format!("rm -rf {}", remote_shell_quote(remote)),
        ],
        std::time::Duration::from_secs(30),
    )
    .await?;
    let mut result = adb_action_result("File deleted", output)?;
    // Refresh parent directory so Android file managers update
    let parent = parent_path(remote);
    result.stderr = merge_optional_output(
        result.stderr,
        refresh_android_file_index(adb_path, serial, parent)
            .await
            .err(),
    );
    Ok(result)
}

pub async fn execute_create_directory(
    adb_path: &Path,
    serial: &str,
    path: &str,
) -> Result<DeviceActionResult, AppError> {
    validate_device_serial(serial)?;
    let path = path.trim();
    if path.is_empty() {
        return Err(AppError::invalid_config("Directory path cannot be empty"));
    }
    if path == "/" || path == "/sdcard" || path == "/storage" {
        return Err(AppError::invalid_config("Cannot create system directories"));
    }
    let output = ShellRunner::execute_with_timeout(
        adb_path,
        &[
            "-s",
            serial,
            "shell",
            &format!("mkdir -p {}", remote_shell_quote(path)),
        ],
        std::time::Duration::from_secs(10),
    )
    .await?;
    let mut result = adb_action_result("Directory created", output)?;
    let parent = parent_path(path);
    result.stderr = merge_optional_output(
        result.stderr,
        refresh_android_file_index(adb_path, serial, parent)
            .await
            .err(),
    );
    Ok(result)
}

pub async fn execute_create_file(
    adb_path: &Path,
    serial: &str,
    path: &str,
) -> Result<DeviceActionResult, AppError> {
    validate_device_serial(serial)?;
    let path = path.trim();
    if path.is_empty() {
        return Err(AppError::invalid_config("File path cannot be empty"));
    }
    if path == "/" || path == "/sdcard" || path == "/storage" {
        return Err(AppError::invalid_config("Cannot create system files"));
    }

    let command = format!(
        "target={}; if [ -e \"$target\" ]; then echo \"File already exists\" >&2; exit 1; fi; : > \"$target\"",
        remote_shell_quote(path)
    );
    let output = ShellRunner::execute_with_timeout(
        adb_path,
        &["-s", serial, "shell", &command],
        std::time::Duration::from_secs(10),
    )
    .await?;
    let mut result = adb_action_result("File created", output)?;
    let parent = parent_path(path);
    result.stderr = merge_optional_output(
        result.stderr,
        refresh_android_file_index(adb_path, serial, parent)
            .await
            .err(),
    );
    Ok(result)
}

pub async fn execute_push_file_streaming(
    adb_path: &Path,
    serial: &str,
    local_path: &str,
    remote_directory: &str,
    app_handle: &tauri::AppHandle,
) -> Result<DeviceActionResult, AppError> {
    validate_device_serial(serial)?;
    let local = PathBuf::from(local_path);
    if !local.is_file() {
        return Err(AppError::invalid_config("Please select a valid local file"));
    }
    let total_bytes = tokio::fs::metadata(&local)
        .await
        .map(|m| m.len())
        .unwrap_or(0);
    let remote_directory = normalize_remote_push_directory(remote_directory)?;

    // mkdir first (non-streaming, fast)
    let mkdir_output = ShellRunner::execute_with_timeout(
        adb_path,
        &[
            "-s",
            serial,
            "shell",
            &format!("mkdir -p {}", remote_shell_quote(remote_directory)),
        ],
        std::time::Duration::from_secs(10),
    )
    .await?;
    adb_action_result("Remote directory created", mkdir_output)?;

    let remote = build_remote_push_target(&local, remote_directory)?;
    let file_name = local
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("file")
        .to_string();
    let transfer_id = uuid::Uuid::new_v4().to_string();

    let output = push_file_via_exec_in(
        adb_path,
        serial,
        &local,
        &remote,
        &transfer_id,
        &file_name,
        total_bytes,
        app_handle,
    )
    .await?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(AppError::internal_error(&format!(
            "Push failed: {}",
            stderr.trim()
        )));
    }

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let combined = format!("{}\n{}", stdout, stderr);
    if combined.to_lowercase().contains("adb: error:") {
        return Err(AppError::internal_error(&format!(
            "Push failed: {}",
            combined.trim()
        )));
    }

    let mut result = DeviceActionResult {
        message: "File sent".into(),
        output_path: Some(remote.clone()),
        stdout: Some(stdout.trim().to_string()),
        stderr: if stderr.trim().is_empty() {
            None
        } else {
            Some(stderr.trim().to_string())
        },
    };
    result.stderr = merge_optional_output(
        result.stderr,
        refresh_android_file_index(adb_path, serial, &remote)
            .await
            .err(),
    );
    Ok(result)
}

pub async fn execute_pull_file_streaming(
    adb_path: &Path,
    serial: &str,
    remote_path: &str,
    local_directory: &str,
    app_handle: &tauri::AppHandle,
) -> Result<DeviceActionResult, AppError> {
    validate_device_serial(serial)?;
    let remote = remote_path.trim();
    if remote.is_empty() {
        return Err(AppError::invalid_config("Remote file path cannot be empty"));
    }
    let local_dir = PathBuf::from(local_directory.trim());
    if !local_dir.is_dir() {
        tokio::fs::create_dir_all(&local_dir).await?;
    }

    // For directories, delegate to recursive pull
    let is_dir = is_remote_directory(adb_path, serial, remote).await;
    if is_dir {
        return pull_directory_recursive(adb_path, serial, remote, &local_dir, Some(app_handle))
            .await;
    }

    let file_name = remote.rsplit('/').next().unwrap_or(remote).to_string();
    let local_file = local_dir.join(&file_name);
    let transfer_id = uuid::Uuid::new_v4().to_string();
    let total_bytes = remote_file_size(adb_path, serial, remote)
        .await
        .unwrap_or(0);
    let started_at = Instant::now();

    // Use exec-out cat to avoid Windows ADB encoding issues with non-ASCII paths
    let mut child = process_command::new_tokio_command(adb_path)
        .args([
            "-s",
            serial,
            "exec-out",
            "sh",
            "-c",
            &format!("cat {}", remote_shell_quote(remote)),
        ])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| AppError::internal_error(&format!("Failed to spawn adb cat: {e}")))?;

    // Stream stdout to local file
    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();
    let file_name_clone = file_name.clone();
    let transfer_id_clone = transfer_id.clone();
    let handle = app_handle.clone();
    let local_file_clone = local_file.clone();
    let started_at_clone = started_at;

    let write_task = tokio::spawn(async move {
        use tokio::io::AsyncReadExt;
        let mut reader = tokio::io::BufReader::new(stdout);
        let mut file = tokio::fs::File::create(&local_file_clone)
            .await
            .map_err(|e| format!("Failed to create local file: {e}"))?;
        let mut buf = vec![0u8; 65536];
        let mut total: u64 = 0;
        let mut last_reported: u64 = 0;

        loop {
            let n = match reader.read(&mut buf).await {
                Ok(0) => break,
                Ok(n) => n,
                Err(e) => return Err(format!("Read error: {e}")),
            };
            use tokio::io::AsyncWriteExt;
            if let Err(e) = file.write_all(&buf[..n]).await {
                return Err(format!("Write error: {e}"));
            }
            total += n as u64;

            // Report progress every 256KB
            if total - last_reported >= 262144 {
                last_reported = total;
                let _ = tauri::Emitter::emit(
                    &handle,
                    "transfer://progress",
                    TransferProgress {
                        id: transfer_id_clone.clone(),
                        file_name: file_name_clone.clone(),
                        transferred: total,
                        total: total_bytes,
                        percent: progress_percent(total, total_bytes, false),
                        speed: format_speed(total, started_at_clone.elapsed()),
                    },
                );
            }
        }

        use tokio::io::AsyncWriteExt;
        let _ = file.flush().await;
        let _ = file.sync_all().await;

        Ok::<u64, String>(total)
    });
    let stderr_task = tokio::spawn(async move {
        use tokio::io::AsyncReadExt;
        let mut reader = tokio::io::BufReader::new(stderr);
        let mut buf = String::new();
        let _ = reader.read_to_string(&mut buf).await;
        buf
    });

    // Wait for both write task and process exit
    let (write_result, status, stderr_output) = tokio::join!(write_task, child.wait(), stderr_task);
    let bytes_written = write_result
        .map_err(|e| AppError::internal_error(&e.to_string()))
        .and_then(|r| r.map_err(|e| AppError::internal_error(&e)))?;
    let stderr_output = stderr_output.unwrap_or_default();

    let exit_status =
        status.map_err(|e| AppError::internal_error(&format!("Process error: {e}")))?;
    if !exit_status.success() {
        let _ = tokio::fs::remove_file(&local_file).await;
        return Err(AppError::internal_error(&format!(
            "Pull failed: {}",
            stderr_output.trim()
        )));
    }

    // Emit 100% completion
    let _ = tauri::Emitter::emit(
        app_handle,
        "transfer://progress",
        TransferProgress {
            id: transfer_id,
            file_name,
            transferred: bytes_written,
            total: total_bytes.max(bytes_written),
            percent: 100,
            speed: format_speed(bytes_written, started_at.elapsed()),
        },
    );

    sync_and_refresh(&local_file).await;

    Ok(DeviceActionResult {
        message: "File pulled".into(),
        output_path: Some(local_file.to_string_lossy().into_owned()),
        stdout: Some(format!("{} bytes transferred", bytes_written)),
        stderr: None,
    })
}

async fn push_file_via_exec_in(
    adb_path: &Path,
    serial: &str,
    local: &Path,
    remote: &str,
    transfer_id: &str,
    file_name: &str,
    total_bytes: u64,
    app_handle: &tauri::AppHandle,
) -> Result<std::process::Output, AppError> {
    let temp_remote = format!("{remote}.devicedeck-upload-{}", uuid::Uuid::new_v4());
    let command = format!("cat > {}", remote_shell_quote(&temp_remote));

    let mut child = process_command::new_tokio_command(adb_path)
        .args(["-s", serial, "exec-in", "sh", "-c", &command])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| AppError::internal_error(&format!("Failed to spawn adb upload: {e}")))?;

    let mut stdin = child
        .stdin
        .take()
        .ok_or_else(|| AppError::internal_error("Failed to open adb upload stdin"))?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| AppError::internal_error("Failed to open adb upload stdout"))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| AppError::internal_error("Failed to open adb upload stderr"))?;

    let stdout_task = tokio::spawn(async move {
        use tokio::io::AsyncReadExt;
        let mut reader = tokio::io::BufReader::new(stdout);
        let mut buf = Vec::new();
        let _ = reader.read_to_end(&mut buf).await;
        buf
    });
    let stderr_task = tokio::spawn(async move {
        use tokio::io::AsyncReadExt;
        let mut reader = tokio::io::BufReader::new(stderr);
        let mut buf = Vec::new();
        let _ = reader.read_to_end(&mut buf).await;
        buf
    });

    let started_at = Instant::now();
    emit_transfer_progress(
        app_handle,
        transfer_id,
        file_name,
        0,
        total_bytes,
        &started_at,
        false,
    );

    let mut file = tokio::fs::File::open(local)
        .await
        .map_err(|e| AppError::internal_error(&format!("Failed to open local file: {e}")))?;
    let mut buf = vec![0u8; 64 * 1024];
    let mut transferred = 0u64;
    let mut last_reported = 0u64;

    loop {
        use tokio::io::AsyncReadExt;
        let n = file
            .read(&mut buf)
            .await
            .map_err(|e| AppError::internal_error(&format!("Failed to read local file: {e}")))?;
        if n == 0 {
            break;
        }

        use tokio::io::AsyncWriteExt;
        stdin.write_all(&buf[..n]).await.map_err(|e| {
            AppError::internal_error(&format!("Failed to write upload stream: {e}"))
        })?;
        transferred = transferred.saturating_add(n as u64);

        if transferred.saturating_sub(last_reported) >= TRANSFER_PROGRESS_INTERVAL
            || transferred == total_bytes
        {
            last_reported = transferred;
            emit_transfer_progress(
                app_handle,
                transfer_id,
                file_name,
                transferred,
                total_bytes,
                &started_at,
                false,
            );
        }
    }

    use tokio::io::AsyncWriteExt;
    stdin
        .shutdown()
        .await
        .map_err(|e| AppError::internal_error(&format!("Failed to finish upload stream: {e}")))?;
    drop(stdin);

    let status = child
        .wait()
        .await
        .map_err(|e| AppError::internal_error(&format!("Failed to wait for adb upload: {e}")))?;
    let stdout = stdout_task.await.unwrap_or_default();
    let stderr = stderr_task.await.unwrap_or_default();

    let rename_ok = if status.success() {
        let mv_output = ShellRunner::execute_with_timeout(
            adb_path,
            &[
                "-s",
                serial,
                "shell",
                &format!(
                    "mv -f {} {}",
                    remote_shell_quote(&temp_remote),
                    remote_shell_quote(remote)
                ),
            ],
            std::time::Duration::from_secs(15),
        )
        .await;
        match mv_output {
            Ok(o) if o.success => true,
            _ => {
                let _ = ShellRunner::execute_with_timeout(
                    adb_path,
                    &[
                        "-s",
                        serial,
                        "shell",
                        &format!("rm -f {}", remote_shell_quote(&temp_remote)),
                    ],
                    std::time::Duration::from_secs(10),
                )
                .await;
                false
            }
        }
    } else {
        let _ = ShellRunner::execute_with_timeout(
            adb_path,
            &[
                "-s",
                serial,
                "shell",
                &format!("rm -f {}", remote_shell_quote(&temp_remote)),
            ],
            std::time::Duration::from_secs(10),
        )
        .await;
        false
    };

    if rename_ok {
        emit_transfer_progress(
            app_handle,
            transfer_id,
            file_name,
            total_bytes,
            total_bytes,
            &started_at,
            true,
        );
    }

    Ok(std::process::Output {
        status,
        stdout,
        stderr,
    })
}

/// Flush file to disk and notify OS shell to refresh the directory view.
async fn sync_and_refresh(path: &Path) {
    if let Some(parent) = path.parent() {
        #[cfg(target_os = "windows")]
        {
            let parent_str = parent.to_string_lossy().into_owned();
            let _ = tokio::process::Command::new("powershell")
                .args([
                    "-NoProfile", "-Command",
                    &format!(
                        "(New-Object -ComObject Shell.Application).Namespace(0).Self() | Out-Null; [System.IO.Directory]::GetFiles('{}') | Out-Null",
                        parent_str.replace('\'', "''")
                    ),
                ])
                .output()
                .await;
        }
        let _ = parent;
    }
}

fn adb_action_result(
    message: &str,
    output: crate::sidecar::shell_runner::CommandOutput,
) -> Result<DeviceActionResult, AppError> {
    if output.success && !command_output_has_adb_error(&output) {
        Ok(DeviceActionResult {
            message: message.into(),
            output_path: None,
            stdout: Some(output.stdout.trim().to_string()),
            stderr: if output.stderr.trim().is_empty() {
                None
            } else {
                Some(output.stderr.trim().to_string())
            },
        })
    } else {
        let detail = command_output_detail(&output);
        Err(AppError::internal_error(&format!(
            "{}: {}",
            message, detail
        )))
    }
}

fn normalize_remote_push_directory(remote_directory: &str) -> Result<&str, AppError> {
    let remote_directory = remote_directory.trim();
    let remote_directory = if remote_directory.is_empty() {
        DEFAULT_PUSH_DIRECTORY
    } else {
        remote_directory.trim_end_matches('/')
    };
    if remote_directory.is_empty() {
        return Err(AppError::invalid_config("Remote directory cannot be empty"));
    }
    Ok(remote_directory)
}

fn build_remote_push_target(local: &Path, remote_directory: &str) -> Result<String, AppError> {
    let file_name = local_file_name(local)
        .ok_or_else(|| AppError::invalid_config("Cannot read local file name"))?;
    let remote_directory = normalize_remote_push_directory(remote_directory)?;
    Ok(format!("{}/{}", remote_directory, file_name))
}

fn local_file_name(local: &Path) -> Option<&str> {
    local
        .file_name()
        .and_then(|value| value.to_str())
        .and_then(|value| value.rsplit(['/', '\\']).next())
        .filter(|value| !value.trim().is_empty())
}

async fn refresh_android_file_index(
    adb_path: &Path,
    serial: &str,
    remote_file: &str,
) -> Result<(), String> {
    let file_uri = remote_file_uri(remote_file);
    let output = ShellRunner::execute_with_timeout(
        adb_path,
        &[
            "-s",
            serial,
            "shell",
            "am",
            "broadcast",
            "-a",
            "android.intent.action.MEDIA_SCANNER_SCAN_FILE",
            "-d",
            &file_uri,
        ],
        std::time::Duration::from_secs(10),
    )
    .await
    .map_err(|error| error.detail.unwrap_or(error.message))?;

    if output.success && !command_output_has_adb_error(&output) {
        Ok(())
    } else {
        Err(format!(
            "File index refresh failed: {}",
            command_output_detail(&output)
        ))
    }
}

fn parent_path(path: &str) -> &str {
    match path.trim_end_matches('/').rsplit_once('/') {
        Some((parent, _)) if !parent.is_empty() => parent,
        _ => "/",
    }
}

fn remote_file_uri(remote_file: &str) -> String {
    let mut uri = String::from("file://");
    for byte in remote_file.as_bytes() {
        let ch = *byte as char;
        if ch.is_ascii_alphanumeric() || matches!(ch, '/' | '-' | '_' | '.' | '~') {
            uri.push(ch);
        } else {
            uri.push_str(&format!("%{byte:02X}"));
        }
    }
    uri
}

fn merge_optional_output(current: Option<String>, additional: Option<String>) -> Option<String> {
    match (current, additional) {
        (Some(current), Some(additional)) if !additional.trim().is_empty() => {
            Some(format!("{}\n{}", current, additional))
        }
        (None, Some(additional)) if !additional.trim().is_empty() => Some(additional),
        (current, _) => current,
    }
}

fn command_output_has_adb_error(output: &crate::sidecar::shell_runner::CommandOutput) -> bool {
    let combined = format!("{}\n{}", output.stdout, output.stderr).to_lowercase();
    combined.contains("adb: error:")
        || combined.contains("error: failed")
        || combined.contains("failed to ")
}

fn command_output_detail(output: &crate::sidecar::shell_runner::CommandOutput) -> String {
    let combined = format!("{}\n{}", output.stdout.trim(), output.stderr.trim());
    let detail = combined.trim();
    if detail.is_empty() {
        "adb returned no detailed error".into()
    } else {
        detail.into()
    }
}

fn validate_device_serial(serial: &str) -> Result<(), AppError> {
    if serial.trim().is_empty() {
        return Err(AppError::invalid_config("Device serial cannot be empty"));
    }
    if serial.contains([';', '|', '&', '$', '`', '<', '>', '"', '\'']) {
        return Err(AppError::invalid_config(
            "Device serial contains invalid characters",
        ));
    }
    Ok(())
}

fn resolve_output_directory(output_directory: Option<&str>) -> Result<PathBuf, AppError> {
    if let Some(dir) = output_directory
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        return Ok(PathBuf::from(dir));
    }
    std::env::current_dir()
        .map(|dir| dir.join("screenshots"))
        .map_err(AppError::from)
}

fn sanitize_file_part(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '_'
            }
        })
        .collect()
}

fn timestamp() -> String {
    chrono::Local::now().format("%Y%m%d-%H%M%S").to_string()
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sidecar::shell_runner::CommandOutput;

    #[test]
    fn build_remote_push_target_appends_file_name_to_directory() {
        let target = build_remote_push_target(
            Path::new(r"C:\Users\example\Desktop\example.docx"),
            "/sdcard/Download/",
        )
        .unwrap();

        assert_eq!(target, "/sdcard/Download/example.docx");
    }

    #[test]
    fn build_remote_push_target_uses_default_download_directory() {
        let target =
            build_remote_push_target(Path::new(r"C:\Users\example\Desktop\demo.apk"), "").unwrap();

        assert_eq!(target, "/sdcard/Download/DeviceDeck/demo.apk");
    }

    #[test]
    fn build_remote_push_target_keeps_devicedeck_directory_without_trailing_slash() {
        let target = build_remote_push_target(
            Path::new(r"C:\Users\example\Desktop\demo.docx"),
            "/sdcard/Download/DeviceDeck",
        )
        .unwrap();

        assert_eq!(target, "/sdcard/Download/DeviceDeck/demo.docx");
    }

    #[test]
    fn remote_file_uri_percent_encodes_spaces_and_non_ascii_names() {
        let uri = remote_file_uri("/sdcard/Download/DeviceDeck/example 中文.docx");

        assert_eq!(
            uri,
            "file:///sdcard/Download/DeviceDeck/example%20%E4%B8%AD%E6%96%87.docx"
        );
    }

    #[test]
    fn remote_shell_quote_escapes_single_quotes() {
        assert_eq!(
            remote_shell_quote("/sdcard/My Files/a'b.txt"),
            "'/sdcard/My Files/a'\"'\"'b.txt'"
        );
    }

    #[test]
    fn progress_percent_caps_incomplete_progress() {
        assert_eq!(progress_percent(50, 100, false), 50);
        assert_eq!(progress_percent(120, 100, false), 99);
        assert_eq!(progress_percent(0, 0, false), 0);
        assert_eq!(progress_percent(0, 0, true), 100);
    }

    #[test]
    fn format_speed_uses_human_readable_units() {
        assert_eq!(
            format_speed(2048, std::time::Duration::from_secs(2)),
            "1.0 KB/s"
        );
    }

    #[test]
    fn adb_action_result_rejects_adb_error_output_even_with_success_status() {
        let output = CommandOutput {
            success: true,
            stdout: "C:\\Users\\example\\Desktop\\demo.docx: 1 file pushed, 0 skipped".into(),
            stderr: "adb: error: failed to copy 'demo.docx' to '/sdcard/Download/.': remote couldn't create file: Is a directory".into(),
        };

        let error = adb_action_result("文件已发送", output).unwrap_err();

        assert_eq!(error.code, "INTERNAL_ERROR");
        assert!(error.detail.unwrap().contains("adb: error:"));
    }
}
