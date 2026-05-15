use std::path::{Path, PathBuf};

use super::parser;
use super::types::{AndroidDeviceProps, RawDevice, WirelessAdbService};
use crate::core::error::AppError;
use crate::core::types::{
    ConnectionType, DeviceActionResult, DeviceCapability, DeviceInfo, DeviceKeyAction,
    DevicePlatform, DeviceStatus,
};
use crate::sidecar::process_command;
use crate::sidecar::shell_runner::ShellRunner;

const DEFAULT_PUSH_DIRECTORY: &str = "/sdcard/Download/DeviceDeck";

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
    .map_err(|_| AppError::internal_error("截图命令执行超时"))?
    .map_err(|e| AppError::internal_error(&format!("截图命令执行失败: {e}")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(AppError::internal_error(&format!(
            "截图失败: {}",
            stderr.trim()
        )));
    }

    tokio::fs::write(&output_path, output.stdout).await?;

    Ok(DeviceActionResult {
        message: "截图已保存".into(),
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
        return Err(AppError::invalid_config("请选择有效的 .apk 文件"));
    }
    let apk_arg = apk.to_string_lossy().into_owned();
    let output = ShellRunner::execute_with_timeout(
        adb_path,
        &["-s", serial, "install", "-r", &apk_arg],
        std::time::Duration::from_secs(120),
    )
    .await?;
    adb_action_result("APK 安装完成", output)
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
        return Err(AppError::invalid_config("请选择有效的本地文件"));
    }
    let remote_directory = normalize_remote_push_directory(remote_directory)?;
    let mkdir_output = ShellRunner::execute_with_timeout(
        adb_path,
        &["-s", serial, "shell", "mkdir", "-p", &remote_directory],
        std::time::Duration::from_secs(10),
    )
    .await?;
    adb_action_result("远端目录创建完成", mkdir_output)?;

    let remote = build_remote_push_target(&local, remote_directory)?;
    let local_arg = local.to_string_lossy().into_owned();
    let output = ShellRunner::execute_with_timeout(
        adb_path,
        &["-s", serial, "push", &local_arg, &remote],
        std::time::Duration::from_secs(120),
    )
    .await?;
    let mut result = adb_action_result("文件已发送", output)?;
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
                    return adb_action_result("屏幕已关闭", output);
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
            adb_action_result("屏幕亮度已调至最低", output)
        }
        DeviceKeyAction::ScreenRestore => {
            let primary = vec!["-s", serial, "shell", "cmd", "display", "power-on"];
            let result = ShellRunner::execute_with_timeout(adb_path, &primary, timeout).await;
            if let Ok(output) = result {
                if output.success {
                    return adb_action_result("屏幕已恢复", output);
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
            adb_action_result("屏幕亮度已恢复", output)
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
            adb_action_result("快捷操作已执行", output)
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
        return Err(AppError::invalid_config("ADB shell 命令不能为空"));
    }
    let timeout =
        std::time::Duration::from_millis(timeout_ms.unwrap_or(30_000).clamp(1_000, 120_000));
    let output =
        ShellRunner::execute_with_timeout(adb_path, &["-s", serial, "shell", command], timeout)
            .await?;
    adb_action_result("ADB shell 命令已执行", output)
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
        return Err(AppError::invalid_config("远端目录不能为空"));
    }
    Ok(remote_directory)
}

fn build_remote_push_target(local: &Path, remote_directory: &str) -> Result<String, AppError> {
    let file_name =
        local_file_name(local).ok_or_else(|| AppError::invalid_config("无法读取本地文件名"))?;
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
            "文件索引刷新失败: {}",
            command_output_detail(&output)
        ))
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
        "adb 未返回详细错误".into()
    } else {
        detail.into()
    }
}

fn validate_device_serial(serial: &str) -> Result<(), AppError> {
    if serial.trim().is_empty() {
        return Err(AppError::invalid_config("设备序列号不能为空"));
    }
    if serial.contains([';', '|', '&', '$', '`', '<', '>', '"', '\'']) {
        return Err(AppError::invalid_config("设备序列号包含非法字符"));
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
