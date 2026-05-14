use crate::core::error::AppError;
use crate::core::types::MirrorConfig;

const ALLOWED_SIZES: &[&str] = &["720", "1080", "1440", "native"];
const ALLOWED_BITRATES: &[&str] = &["2M", "4M", "8M", "16M"];
const ALLOWED_FPS: &[&str] = &["30", "60", "90", "120"];

const DANGEROUS_CHARS: &[char] = &[';', '|', '&', '$', '`', '(', ')', '{', '}', '<', '>', '!', '#', '~', '*'];

pub fn validate_serial(serial: &str) -> Result<(), AppError> {
    if serial.is_empty() {
        return Err(AppError::invalid_config("serial 不能为空"));
    }
    if serial.contains(DANGEROUS_CHARS) {
        return Err(AppError::invalid_config("serial 包含非法字符"));
    }
    Ok(())
}

pub fn validate_config(config: &MirrorConfig) -> Result<(), AppError> {
    if !ALLOWED_SIZES.contains(&config.max_size.as_str()) {
        return Err(AppError::invalid_config(&format!(
            "无效的 maxSize: {}，允许值: {:?}",
            config.max_size, ALLOWED_SIZES
        )));
    }
    if !ALLOWED_BITRATES.contains(&config.video_bit_rate.as_str()) {
        return Err(AppError::invalid_config(&format!(
            "无效的 videoBitRate: {}，允许值: {:?}",
            config.video_bit_rate, ALLOWED_BITRATES
        )));
    }
    if !ALLOWED_FPS.contains(&config.max_fps.as_str()) {
        return Err(AppError::invalid_config(&format!(
            "无效的 maxFps: {}，允许值: {:?}",
            config.max_fps, ALLOWED_FPS
        )));
    }
    Ok(())
}

pub fn build_scrcpy_args(serial: &str, config: &MirrorConfig) -> Result<Vec<String>, AppError> {
    validate_serial(serial)?;
    validate_config(config)?;

    let mut args = vec!["--serial".into(), serial.into()];

    if config.max_size != "native" {
        args.extend(["--max-size".into(), config.max_size.clone()]);
    }
    args.extend(["--video-bit-rate".into(), config.video_bit_rate.clone()]);
    args.extend(["--max-fps".into(), config.max_fps.clone()]);

    if config.no_control {
        args.push("--no-control".into());
    }
    if config.stay_awake {
        args.push("--stay-awake".into());
    }
    if config.turn_screen_off {
        args.push("--turn-screen-off".into());
    }

    Ok(args)
}
