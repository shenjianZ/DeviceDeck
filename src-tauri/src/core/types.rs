use serde::{Deserialize, Serialize};

// ---- Device ----

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceInfo {
    pub id: String,
    pub serial: String,
    pub name: String,
    pub model: String,
    pub brand: String,
    pub platform: DevicePlatform,
    pub status: DeviceStatus,
    pub connection_type: ConnectionType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub android_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub screen_size: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub battery_level: Option<i32>,
    pub capabilities: Vec<DeviceCapability>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DevicePlatform {
    Android,
    Ios,
    #[serde(rename = "androidTv")]
    AndroidTv,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DeviceStatus {
    Online,
    Offline,
    Unauthorized,
    Busy,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ConnectionType {
    Usb,
    Wifi,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DeviceCapability {
    Mirror,
    Control,
    Screenshot,
    Recording,
    Wireless,
    #[serde(rename = "installApp")]
    InstallApp,
    #[serde(rename = "uninstallApp")]
    UninstallApp,
    Logs,
    #[serde(rename = "fileTransfer")]
    FileTransfer,
    Automation,
}

// ---- Mirror ----

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MirrorConfig {
    pub max_size: String,
    pub video_bit_rate: String,
    pub max_fps: String,
    pub no_control: bool,
    pub stay_awake: bool,
    pub turn_screen_off: bool,
}

impl Default for MirrorConfig {
    fn default() -> Self {
        Self {
            max_size: "1080".into(),
            video_bit_rate: "8M".into(),
            max_fps: "60".into(),
            no_control: false,
            stay_awake: true,
            turn_screen_off: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MirrorSession {
    pub id: String,
    pub device_serial: String,
    pub platform: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub process_id: Option<u32>,
    pub status: SessionStatus,
    pub started_at: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stopped_at: Option<u64>,
    pub config: MirrorConfig,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SessionStatus {
    Running,
    Stopped,
    Failed,
}

// ---- Environment ----

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolStatus {
    pub name: String,
    pub available: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvironmentStatus {
    pub adb: ToolStatus,
    pub scrcpy: ToolStatus,
    pub provider_status: String,
}

// ---- Log ----

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppLog {
    pub id: String,
    pub time: u64,
    pub source: LogSource,
    pub level: LogLevel,
    pub device_serial: String,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LogSource {
    System,
    Adb,
    Scrcpy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LogLevel {
    Info,
    Warn,
    Error,
}

// ---- Settings ----

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub use_bundled_adb: bool,
    pub use_bundled_scrcpy: bool,
    pub custom_adb_path: String,
    pub custom_scrcpy_path: String,
    pub default_mirror_config: MirrorConfig,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_mirror_config: Option<MirrorConfig>,
    pub theme: String,
    pub log_retention_days: u32,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            use_bundled_adb: true,
            use_bundled_scrcpy: true,
            custom_adb_path: String::new(),
            custom_scrcpy_path: String::new(),
            default_mirror_config: MirrorConfig::default(),
            last_mirror_config: None,
            theme: "dark".into(),
            log_retention_days: 7,
        }
    }
}
