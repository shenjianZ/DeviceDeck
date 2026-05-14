use crate::core::error::AppError;
use crate::core::types::MirrorConfig;

const ALLOWED_SIZES: &[&str] = &["720", "1080", "1440", "native"];
const ALLOWED_BITRATES: &[&str] = &["2M", "4M", "8M", "16M", "24M", "32M", "50M"];
const ALLOWED_FPS: &[&str] = &["30", "60", "90", "120"];
const ALLOWED_VIDEO_CODECS: &[&str] = &["h264", "h265", "av1"];

const DANGEROUS_CHARS: &[char] = &[
    ';', '|', '&', '$', '`', '(', ')', '{', '}', '<', '>', '!', '#', '~', '*',
];

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
    if !ALLOWED_VIDEO_CODECS.contains(&config.video_codec.as_str()) {
        return Err(AppError::invalid_config(&format!(
            "无效的 videoCodec: {}，允许值: {:?}",
            config.video_codec, ALLOWED_VIDEO_CODECS
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
    args.extend(["--video-codec".into(), config.video_codec.clone()]);

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_native_high_quality_h265_args() {
        let config = MirrorConfig {
            max_size: "native".into(),
            video_bit_rate: "50M".into(),
            max_fps: "60".into(),
            video_codec: "h265".into(),
            no_control: false,
            stay_awake: true,
            turn_screen_off: true,
        };

        let args = build_scrcpy_args("192.168.43.187:41457", &config).unwrap();

        assert!(!args.contains(&"--max-size".to_string()));
        assert!(args
            .windows(2)
            .any(|pair| pair == ["--video-bit-rate", "50M"]));
        assert!(args
            .windows(2)
            .any(|pair| pair == ["--video-codec", "h265"]));
        assert!(args.contains(&"--turn-screen-off".to_string()));
    }

    #[test]
    fn rejects_unsupported_video_codec() {
        let config = MirrorConfig {
            video_codec: "vp9".into(),
            ..MirrorConfig::default()
        };

        let error = build_scrcpy_args("device-serial", &config).unwrap_err();

        assert_eq!(error.code, "INVALID_CONFIG");
    }
}
