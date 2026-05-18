use std::path::{Path, PathBuf};

use crate::core::error::AppError;
use crate::core::types::{
    AudioCodec, AudioSource, DeviceCapabilityReport, MirrorConfig, MirrorOrientation,
    RecommendedConfig, RecordFormat, RecordMode, VideoCodec,
};
use crate::providers::android::parser;
use crate::sidecar::shell_runner::ShellRunner;

const ALLOWED_SIZES: &[&str] = &["720", "1080", "1440", "native"];
const ALLOWED_BITRATES: &[&str] = &["2M", "4M", "8M", "16M", "24M", "32M", "50M"];
const ALLOWED_FPS: &[&str] = &["30", "60", "90", "120"];
const ALLOWED_VIDEO_CODECS: &[&str] = &["h264", "h265", "av1"];

const DANGEROUS_CHARS: &[char] = &[
    ';', '|', '&', '$', '`', '(', ')', '{', '}', '<', '>', '!', '#', '~', '*',
];

pub fn validate_serial(serial: &str) -> Result<(), AppError> {
    if serial.is_empty() {
        return Err(AppError::invalid_config("Serial cannot be empty"));
    }
    if serial.contains(DANGEROUS_CHARS) {
        return Err(AppError::invalid_config("Serial contains invalid characters"));
    }
    Ok(())
}

pub fn validate_config(config: &MirrorConfig) -> Result<(), AppError> {
    if !ALLOWED_SIZES.contains(&config.max_size.as_str()) {
        return Err(AppError::invalid_config(&format!(
            "Invalid maxSize: {}, allowed: {:?}",
            config.max_size, ALLOWED_SIZES
        )));
    }
    if !ALLOWED_BITRATES.contains(&config.video_bit_rate.as_str()) {
        return Err(AppError::invalid_config(&format!(
            "Invalid videoBitRate: {}, allowed: {:?}",
            config.video_bit_rate, ALLOWED_BITRATES
        )));
    }
    if !ALLOWED_FPS.contains(&config.max_fps.as_str()) {
        return Err(AppError::invalid_config(&format!(
            "Invalid maxFps: {}, allowed: {:?}",
            config.max_fps, ALLOWED_FPS
        )));
    }
    if !ALLOWED_VIDEO_CODECS.contains(&config.video_codec.as_str()) {
        return Err(AppError::invalid_config(&format!(
            "Invalid videoCodec: {}, allowed: {:?}",
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
    if config.screen_black_mode {
        args.push("--turn-screen-off".into());
        args.push("--stay-awake".into());
    } else {
        if config.stay_awake {
            args.push("--stay-awake".into());
        }
        if config.turn_screen_off {
            args.push("--turn-screen-off".into());
        }
    }
    if config.record_mode != RecordMode::Off {
        args.extend(["--record".into(), build_record_path(serial, config)]);
        args.extend([
            "--record-format".into(),
            record_format_arg(config.record_format).into(),
        ]);
        if config.record_mode == RecordMode::Background {
            args.push("--no-window".into());
        }
    }
    if config.always_on_top {
        args.push("--always-on-top".into());
    }
    if config.window_borderless {
        args.push("--window-borderless".into());
    }
    if config.print_fps {
        args.push("--print-fps".into());
    }
    if config.orientation != MirrorOrientation::Unlocked {
        args.extend([
            "--orientation".into(),
            orientation_arg(config.orientation).into(),
        ]);
    }
    if config.audio_enabled {
        args.extend([
            "--audio-source".into(),
            audio_source_arg(config.audio_source).into(),
        ]);
        args.extend([
            "--audio-codec".into(),
            audio_codec_arg(config.audio_codec).into(),
        ]);
        if config.audio_duplicate {
            args.push("--audio-dup".into());
        }
        if config.require_audio {
            args.push("--require-audio".into());
        }
    } else {
        args.push("--no-audio".into());
    }

    Ok(args)
}

fn build_record_path(serial: &str, config: &MirrorConfig) -> String {
    let extension = record_format_arg(config.record_format);
    let safe_serial: String = serial
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '_'
            }
        })
        .collect();
    let timestamp = chrono::Local::now().format("%Y%m%d-%H%M%S");
    let file_name = format!("DeviceDeck-{safe_serial}-{timestamp}.{extension}");
    if config.record_directory.trim().is_empty() {
        file_name
    } else {
        PathBuf::from(config.record_directory.trim())
            .join(file_name)
            .to_string_lossy()
            .into_owned()
    }
}

fn record_format_arg(format: RecordFormat) -> &'static str {
    match format {
        RecordFormat::Mp4 => "mp4",
        RecordFormat::Mkv => "mkv",
    }
}

fn orientation_arg(orientation: MirrorOrientation) -> &'static str {
    match orientation {
        MirrorOrientation::Unlocked => "",
        MirrorOrientation::Portrait0 => "0",
        MirrorOrientation::Landscape90 => "90",
        MirrorOrientation::Portrait180 => "180",
        MirrorOrientation::Landscape270 => "270",
    }
}

fn audio_source_arg(source: AudioSource) -> &'static str {
    match source {
        AudioSource::Output => "output",
        AudioSource::Playback => "playback",
        AudioSource::Mic => "mic",
        AudioSource::MicCamcorder => "mic-camcorder",
        AudioSource::VoiceRecognition => "voice-recognition",
        AudioSource::VoiceCommunication => "voice-communication",
        AudioSource::VoicePerformance => "voice-performance",
    }
}

fn audio_codec_arg(codec: AudioCodec) -> &'static str {
    match codec {
        AudioCodec::Opus => "opus",
        AudioCodec::Aac => "aac",
        AudioCodec::Flac => "flac",
        AudioCodec::Raw => "raw",
    }
}

pub async fn execute_list_encoders(
    scrcpy_path: &Path,
    serial: &str,
) -> Result<(Vec<String>, Vec<VideoCodec>), AppError> {
    validate_serial(serial)?;

    let output = ShellRunner::execute_with_timeout(
        scrcpy_path,
        &["--serial", serial, "--list-encoders"],
        std::time::Duration::from_secs(15),
    )
    .await?;

    if !output.success {
        return Err(AppError::capability_detection_failed(&format!(
            "scrcpy --list-encoders failed: {}",
            output.stderr
        )));
    }

    let (encoders, codecs) = parser::parse_scrcpy_encoders(&output.stdout);
    if codecs.is_empty() {
        return Err(AppError::capability_detection_failed(
            "No video encoders detected. scrcpy may be outdated or device unsupported",
        ));
    }

    Ok((encoders, codecs))
}

pub fn generate_recommendations(report: &DeviceCapabilityReport) -> Vec<RecommendedConfig> {
    let has_h265 = report.supported_codecs.contains(&VideoCodec::H265);
    let has_av1 = report.supported_codecs.contains(&VideoCodec::Av1);

    let best_codec = if has_av1 {
        "av1"
    } else if has_h265 {
        "h265"
    } else {
        "h264"
    };

    let max_dim = std::cmp::max(
        report.screen_width.unwrap_or(1080),
        report.screen_height.unwrap_or(1920),
    );

    let recommended_size = if max_dim >= 2560 {
        "1080"
    } else if max_dim >= 1920 {
        "1080"
    } else {
        "720"
    };

    let recommended_codec = if has_h265 { "h265" } else { "h264" };

    let recommended_bitrate = if has_h265 { "8M" } else { "8M" };

    vec![
        RecommendedConfig {
            label: "capability.recommended".into(),
            description: "capability.recommendedDesc".into(),
            config: MirrorConfig {
                max_size: recommended_size.into(),
                video_bit_rate: recommended_bitrate.into(),
                max_fps: "60".into(),
                video_codec: recommended_codec.into(),
                no_control: false,
                stay_awake: true,
                turn_screen_off: false,
                ..MirrorConfig::default()
            },
        },
        RecommendedConfig {
            label: "capability.highQuality".into(),
            description: "capability.highQualityDesc".into(),
            config: MirrorConfig {
                max_size: "native".into(),
                video_bit_rate: "50M".into(),
                max_fps: "60".into(),
                video_codec: best_codec.into(),
                no_control: false,
                stay_awake: true,
                turn_screen_off: false,
                ..MirrorConfig::default()
            },
        },
        RecommendedConfig {
            label: "capability.lowLatency".into(),
            description: "capability.lowLatencyDesc".into(),
            config: MirrorConfig {
                max_size: "720".into(),
                video_bit_rate: "4M".into(),
                max_fps: "60".into(),
                video_codec: "h264".into(),
                no_control: false,
                stay_awake: true,
                turn_screen_off: true,
                ..MirrorConfig::default()
            },
        },
    ]
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
            ..MirrorConfig::default()
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

    #[test]
    fn builds_recording_window_and_audio_args() {
        let config = MirrorConfig {
            record_mode: RecordMode::Window,
            record_format: RecordFormat::Mkv,
            record_directory: "D:\\captures".into(),
            always_on_top: true,
            window_borderless: true,
            print_fps: true,
            orientation: MirrorOrientation::Landscape90,
            audio_enabled: true,
            audio_source: AudioSource::Playback,
            audio_codec: AudioCodec::Aac,
            audio_duplicate: true,
            require_audio: true,
            ..MirrorConfig::default()
        };

        let args = build_scrcpy_args("device-serial", &config).unwrap();

        assert!(args
            .windows(2)
            .any(|pair| pair == ["--record-format", "mkv"]));
        assert!(args
            .windows(2)
            .any(|pair| pair[0] == "--record" && pair[1].ends_with(".mkv")));
        assert!(!args.contains(&"--no-window".to_string()));
        assert!(args.contains(&"--always-on-top".to_string()));
        assert!(args.contains(&"--window-borderless".to_string()));
        assert!(args.contains(&"--print-fps".to_string()));
        assert!(args.windows(2).any(|pair| pair == ["--orientation", "90"]));
        assert!(args
            .windows(2)
            .any(|pair| pair == ["--audio-source", "playback"]));
        assert!(args.windows(2).any(|pair| pair == ["--audio-codec", "aac"]));
        assert!(args.contains(&"--audio-dup".to_string()));
        assert!(args.contains(&"--require-audio".to_string()));
    }

    #[test]
    fn builds_background_recording_without_audio_playback_window() {
        let config = MirrorConfig {
            record_mode: RecordMode::Background,
            audio_enabled: false,
            ..MirrorConfig::default()
        };

        let args = build_scrcpy_args("device-serial", &config).unwrap();

        assert!(args.contains(&"--no-window".to_string()));
        assert!(args.contains(&"--no-audio".to_string()));
    }
}
