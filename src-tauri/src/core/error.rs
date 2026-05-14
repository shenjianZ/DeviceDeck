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

    // ---- Factory methods for common errors ----

    pub fn adb_not_found() -> Self {
        Self::new("ADB_NOT_FOUND", "未检测到 adb")
            .with_suggestion("请使用内置 adb，或在设置中配置 adb.exe 路径")
    }

    pub fn scrcpy_not_found() -> Self {
        Self::new("SCRCPY_NOT_FOUND", "未检测到 scrcpy")
            .with_suggestion("请使用内置 scrcpy，或在设置中配置 scrcpy.exe 路径")
    }

    pub fn device_not_found(serial: &str) -> Self {
        Self::new("DEVICE_NOT_FOUND", "设备未找到")
            .with_detail(serial)
    }

    pub fn device_unauthorized(serial: &str) -> Self {
        Self::new("DEVICE_UNAUTHORIZED", "设备未授权")
            .with_detail(serial)
            .with_suggestion("请在 Android 手机上确认 USB 调试授权弹窗，然后重新扫描设备")
    }

    pub fn device_offline(serial: &str) -> Self {
        Self::new("DEVICE_OFFLINE", "设备离线")
            .with_detail(serial)
            .with_suggestion("请检查 USB 连接或重新插拔数据线")
    }

    pub fn mirror_already_running(serial: &str) -> Self {
        Self::new("MIRROR_ALREADY_RUNNING", "该设备已有运行中的投屏会话")
            .with_detail(serial)
    }

    pub fn mirror_start_failed(reason: &str) -> Self {
        Self::new("MIRROR_START_FAILED", "投屏启动失败")
            .with_detail(reason)
    }

    pub fn mirror_stop_failed(reason: &str) -> Self {
        Self::new("MIRROR_STOP_FAILED", "投屏停止失败")
            .with_detail(reason)
    }

    pub fn wireless_connect_failed(reason: &str) -> Self {
        Self::new("WIRELESS_CONNECT_FAILED", "无线连接失败")
            .with_detail(reason)
            .with_suggestion("请确认手机和电脑在同一局域网，且手机已开启 USB 调试或无线调试")
    }

    pub fn wireless_ip_not_found(serial: &str) -> Self {
        Self::new("WIRELESS_IP_NOT_FOUND", "未获取到设备局域网 IP")
            .with_detail(serial)
            .with_suggestion("请确认手机已连接 WiFi，并与电脑处于同一局域网")
    }

    pub fn invalid_config(reason: &str) -> Self {
        Self::new("INVALID_CONFIG", "无效的配置参数")
            .with_detail(reason)
    }

    pub fn invalid_tool_path(path: &str) -> Self {
        Self::new("INVALID_TOOL_PATH", "无效的工具路径")
            .with_detail(path)
    }

    pub fn provider_not_implemented(platform: &str) -> Self {
        Self::new("PROVIDER_NOT_IMPLEMENTED", "该平台暂未实现")
            .with_detail(platform)
            .with_suggestion("Coming Soon")
    }

    pub fn internal_error(reason: &str) -> Self {
        Self::new("INTERNAL_ERROR", "内部错误")
            .with_detail(reason)
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::internal_error(&err.to_string())
    }
}

impl From<rusqlite::Error> for AppError {
    fn from(err: rusqlite::Error) -> Self {
        AppError::internal_error(&format!("数据库错误: {err}"))
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::internal_error(&format!("序列化错误: {err}"))
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)
    }
}

impl std::error::Error for AppError {}
