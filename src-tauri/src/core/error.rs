use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppError {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggestion: Option<String>,
}

impl AppError {
    pub fn new(code: &str, message: &str) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            detail: None,
            suggestion: None,
        }
    }

    pub fn with_detail(mut self, detail: &str) -> Self {
        self.detail = Some(detail.into());
        self
    }

    pub fn with_suggestion(mut self, suggestion: &str) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }

    pub fn adb_not_found() -> Self {
        Self::new("ADB_NOT_FOUND", "adb not detected").with_suggestion(adb_not_found_suggestion())
    }

    pub fn scrcpy_not_found() -> Self {
        Self::new("SCRCPY_NOT_FOUND", "scrcpy not detected")
            .with_suggestion(scrcpy_not_found_suggestion())
    }

    pub fn device_not_found(serial: &str) -> Self {
        Self::new("DEVICE_NOT_FOUND", "Device not found").with_detail(serial)
    }

    pub fn device_unauthorized(serial: &str) -> Self {
        Self::new("DEVICE_UNAUTHORIZED", "Device unauthorized")
            .with_detail(serial)
            .with_suggestion("Confirm the USB debugging authorization dialog on your Android device, then rescan")
    }

    pub fn device_offline(serial: &str) -> Self {
        Self::new("DEVICE_OFFLINE", "Device offline")
            .with_detail(serial)
            .with_suggestion("Check USB connection or reconnect the cable")
    }

    pub fn mirror_already_running(serial: &str) -> Self {
        Self::new("MIRROR_ALREADY_RUNNING", "A mirror session is already running on this device").with_detail(serial)
    }

    pub fn mirror_start_failed(reason: &str) -> Self {
        Self::new("MIRROR_START_FAILED", "Failed to start mirroring").with_detail(reason)
    }

    pub fn mirror_stop_failed(reason: &str) -> Self {
        Self::new("MIRROR_STOP_FAILED", "Failed to stop mirroring").with_detail(reason)
    }

    pub fn wireless_connect_failed(reason: &str) -> Self {
        Self::new("WIRELESS_CONNECT_FAILED", "Wireless connection failed")
            .with_detail(reason)
            .with_suggestion("Make sure your phone and computer are on the same network, and USB/wireless debugging is enabled")
    }

    pub fn wireless_ip_not_found(serial: &str) -> Self {
        Self::new("WIRELESS_IP_NOT_FOUND", "Could not get device LAN IP")
            .with_detail(serial)
            .with_suggestion("Make sure your phone is connected to WiFi and on the same network as your computer")
    }

    pub fn wireless_discovery_failed(reason: &str) -> Self {
        Self::new("WIRELESS_DISCOVERY_FAILED", "Wireless device discovery failed")
            .with_detail(reason)
            .with_suggestion("Make sure ADB is available and wireless debugging is enabled in Android settings")
    }

    pub fn invalid_config(reason: &str) -> Self {
        Self::new("INVALID_CONFIG", "Invalid configuration").with_detail(reason)
    }

    pub fn invalid_tool_path(path: &str) -> Self {
        Self::new("INVALID_TOOL_PATH", "Invalid tool path").with_detail(path)
    }

    pub fn capability_detection_failed(reason: &str) -> Self {
        Self::new("CAPABILITY_DETECTION_FAILED", "Device capability detection failed")
            .with_detail(reason)
            .with_suggestion("Make sure the device is connected and scrcpy supports --list-encoders")
    }

    pub fn internal_error(reason: &str) -> Self {
        Self::new("INTERNAL_ERROR", "Internal error").with_detail(reason)
    }
}

#[cfg(all(target_os = "linux", target_arch = "aarch64"))]
fn adb_not_found_suggestion() -> &'static str {
    "Bundled adb is not available on this platform (Linux ARM64). Install via: sudo apt install adb, or configure a custom path in settings"
}

#[cfg(not(all(target_os = "linux", target_arch = "aarch64")))]
fn adb_not_found_suggestion() -> &'static str {
    "Use bundled adb, or configure adb path in settings"
}

#[cfg(all(target_os = "linux", target_arch = "aarch64"))]
fn scrcpy_not_found_suggestion() -> &'static str {
    "Bundled scrcpy is not available on this platform (Linux ARM64). Install via: sudo apt install scrcpy, or configure a custom path in settings"
}

#[cfg(not(all(target_os = "linux", target_arch = "aarch64")))]
fn scrcpy_not_found_suggestion() -> &'static str {
    "Use bundled scrcpy, or configure scrcpy path in settings"
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::internal_error(&err.to_string())
    }
}

impl From<rusqlite::Error> for AppError {
    fn from(err: rusqlite::Error) -> Self {
        AppError::internal_error(&format!("Database error: {err}"))
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::internal_error(&format!("Serialization error: {err}"))
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)
    }
}

impl std::error::Error for AppError {}
