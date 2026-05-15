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

// ---- Device Capability Detection ----

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum VideoCodec {
    H264,
    H265,
    Av1,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceCapabilityReport {
    pub serial: String,
    pub supported_encoders: Vec<String>,
    pub supported_codecs: Vec<VideoCodec>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub screen_width: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub screen_height: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub android_version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecommendedConfig {
    pub label: String,
    pub description: String,
    pub config: MirrorConfig,
}

// ---- Mirror ----

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RecordMode {
    Off,
    Window,
    Background,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RecordFormat {
    Mp4,
    Mkv,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MirrorOrientation {
    Unlocked,
    #[serde(rename = "0")]
    Portrait0,
    #[serde(rename = "90")]
    Landscape90,
    #[serde(rename = "180")]
    Portrait180,
    #[serde(rename = "270")]
    Landscape270,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AudioSource {
    Output,
    Playback,
    Mic,
    MicCamcorder,
    VoiceRecognition,
    VoiceCommunication,
    VoicePerformance,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AudioCodec {
    Opus,
    Aac,
    Flac,
    Raw,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MirrorConfig {
    pub max_size: String,
    pub video_bit_rate: String,
    pub max_fps: String,
    #[serde(default = "default_video_codec")]
    pub video_codec: String,
    pub no_control: bool,
    pub stay_awake: bool,
    pub turn_screen_off: bool,
    #[serde(default)]
    pub screen_black_mode: bool,
    #[serde(default)]
    pub record_mode: RecordMode,
    #[serde(default)]
    pub record_format: RecordFormat,
    #[serde(default)]
    pub record_directory: String,
    #[serde(default)]
    pub always_on_top: bool,
    #[serde(default)]
    pub window_borderless: bool,
    #[serde(default)]
    pub print_fps: bool,
    #[serde(default)]
    pub orientation: MirrorOrientation,
    #[serde(default = "default_audio_enabled")]
    pub audio_enabled: bool,
    #[serde(default)]
    pub audio_source: AudioSource,
    #[serde(default)]
    pub audio_codec: AudioCodec,
    #[serde(default)]
    pub audio_duplicate: bool,
    #[serde(default)]
    pub require_audio: bool,
}

impl Default for MirrorConfig {
    fn default() -> Self {
        Self {
            max_size: "1080".into(),
            video_bit_rate: "8M".into(),
            max_fps: "60".into(),
            video_codec: default_video_codec(),
            no_control: false,
            stay_awake: true,
            turn_screen_off: false,
            screen_black_mode: false,
            record_mode: RecordMode::Off,
            record_format: RecordFormat::Mp4,
            record_directory: String::new(),
            always_on_top: false,
            window_borderless: false,
            print_fps: false,
            orientation: MirrorOrientation::Unlocked,
            audio_enabled: true,
            audio_source: AudioSource::Output,
            audio_codec: AudioCodec::Opus,
            audio_duplicate: false,
            require_audio: false,
        }
    }
}

impl Default for RecordMode {
    fn default() -> Self {
        Self::Off
    }
}

impl Default for RecordFormat {
    fn default() -> Self {
        Self::Mp4
    }
}

impl Default for MirrorOrientation {
    fn default() -> Self {
        Self::Unlocked
    }
}

impl Default for AudioSource {
    fn default() -> Self {
        Self::Output
    }
}

impl Default for AudioCodec {
    fn default() -> Self {
        Self::Opus
    }
}

fn default_video_codec() -> String {
    "h264".into()
}

fn default_audio_enabled() -> bool {
    true
}

// ---- Device Tools ----

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceActionResult {
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stdout: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stderr: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DeviceKeyAction {
    Home,
    Back,
    AppSwitch,
    Menu,
    Power,
    VolumeUp,
    VolumeDown,
    ExpandNotifications,
    CollapseNotifications,
    TurnScreenOff,
    ScreenBlack,
    ScreenRestore,
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
    #[serde(default = "default_auto_scan_devices")]
    pub auto_scan_devices: bool,
    #[serde(default = "default_device_scan_interval_seconds")]
    pub device_scan_interval_seconds: u32,
    #[serde(default = "default_font_size")]
    pub font_size: u32,
    #[serde(default = "default_locale")]
    pub locale: String,
    #[serde(default)]
    pub auto_start: bool,
    #[serde(default = "default_auto_update_enabled")]
    pub auto_update_enabled: bool,
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
            auto_scan_devices: default_auto_scan_devices(),
            device_scan_interval_seconds: default_device_scan_interval_seconds(),
            font_size: default_font_size(),
            locale: default_locale(),
            auto_start: false,
            auto_update_enabled: default_auto_update_enabled(),
        }
    }
}

fn default_auto_scan_devices() -> bool {
    true
}

fn default_device_scan_interval_seconds() -> u32 {
    30
}

fn default_font_size() -> u32 {
    14
}

fn default_locale() -> String {
    "zh-CN".into()
}

fn default_auto_update_enabled() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_settings_default_enables_device_auto_scan() {
        let settings = AppSettings::default();

        assert!(settings.auto_scan_devices);
        assert_eq!(settings.device_scan_interval_seconds, 30);
    }

    #[test]
    fn app_settings_deserializes_old_config_without_scan_fields() {
        let json = r#"{
            "useBundledAdb": true,
            "useBundledScrcpy": true,
            "customAdbPath": "",
            "customScrcpyPath": "",
            "defaultMirrorConfig": {
                "maxSize": "1080",
                "videoBitRate": "8M",
                "maxFps": "60",
                "noControl": false,
                "stayAwake": true,
                "turnScreenOff": false
            },
            "theme": "dark",
            "logRetentionDays": 7
        }"#;

        let settings: AppSettings = serde_json::from_str(json).expect("old config should load");

        assert!(settings.auto_scan_devices);
        assert_eq!(settings.device_scan_interval_seconds, 30);
    }

    #[test]
    fn mirror_config_deserializes_old_config_without_video_codec() {
        let json = r#"{
            "maxSize": "1080",
            "videoBitRate": "8M",
            "maxFps": "60",
            "noControl": false,
            "stayAwake": true,
            "turnScreenOff": false
        }"#;

        let config: MirrorConfig =
            serde_json::from_str(json).expect("old mirror config should load");

        assert_eq!(config.video_codec, "h264");
    }

    #[test]
    fn mirror_config_deserializes_old_config_with_advanced_defaults() {
        let json = r#"{
            "maxSize": "1080",
            "videoBitRate": "8M",
            "maxFps": "60",
            "videoCodec": "h264",
            "noControl": false,
            "stayAwake": true,
            "turnScreenOff": false
        }"#;

        let config: MirrorConfig =
            serde_json::from_str(json).expect("old mirror config should load");

        assert_eq!(config.record_mode, RecordMode::Off);
        assert_eq!(config.record_format, RecordFormat::Mp4);
        assert!(config.record_directory.is_empty());
        assert!(!config.always_on_top);
        assert!(!config.window_borderless);
        assert!(!config.print_fps);
        assert_eq!(config.orientation, MirrorOrientation::Unlocked);
        assert!(config.audio_enabled);
        assert_eq!(config.audio_source, AudioSource::Output);
        assert_eq!(config.audio_codec, AudioCodec::Opus);
        assert!(!config.audio_duplicate);
        assert!(!config.require_audio);
    }
}
