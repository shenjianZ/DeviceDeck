use super::types::{RawDevice, WirelessAdbService, WirelessAdbServiceType};
use crate::core::types::VideoCodec;

pub fn parse_adb_devices(output: &str) -> Vec<RawDevice> {
    let mut devices = Vec::new();

    for line in output.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("List of devices") {
            continue;
        }

        let parts: Vec<&str> = line.splitn(2, char::is_whitespace).collect();
        if parts.len() < 2 {
            continue;
        }

        let serial = parts[0].trim().to_string();
        let rest = parts[1].trim();

        let state = rest
            .split_whitespace()
            .next()
            .unwrap_or("unknown")
            .to_string();

        let mut model = None;

        for token in rest.split_whitespace().skip(1) {
            if let Some(val) = token.strip_prefix("model:") {
                model = Some(val.to_string());
            }
        }

        devices.push(RawDevice {
            serial,
            state,
            model,
        });
    }

    devices
}

pub fn parse_getprop(output: &str) -> std::collections::HashMap<String, String> {
    let mut props = std::collections::HashMap::new();

    for line in output.lines() {
        let line = line.trim();
        // Format: [ro.product.model]: [Pixel 8 Pro]
        if !line.starts_with('[') {
            continue;
        }
        if let Some(close_bracket) = line.find("]: [") {
            let key = line[1..close_bracket].to_string();
            let rest = &line[close_bracket + 4..];
            let value = rest.trim_end_matches(']').to_string();
            props.insert(key, value);
        }
    }

    props
}

pub fn parse_wm_size(output: &str) -> Option<String> {
    for line in output.lines() {
        if let Some(idx) = line.find("Physical size:") {
            let size = line[idx + "Physical size:".len()..].trim();
            if !size.is_empty() {
                return Some(size.replace('x', " × "));
            }
        }
    }
    None
}

pub fn parse_battery_level(output: &str) -> Option<i32> {
    for line in output.lines() {
        let line = line.trim();
        if let Some(val) = line.strip_prefix("level:") {
            if let Ok(level) = val.trim().parse::<i32>() {
                return Some(level);
            }
        }
    }
    None
}

pub fn parse_ip_route_source(output: &str) -> Option<String> {
    for line in output.lines() {
        let mut tokens = line.split_whitespace();
        while let Some(token) = tokens.next() {
            if token == "src" {
                if let Some(ip) = tokens.next() {
                    if is_ipv4(ip) {
                        return Some(ip.to_string());
                    }
                }
            }
        }
    }
    None
}

pub fn parse_wlan_ip(output: &str) -> Option<String> {
    for line in output.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("inet ") {
            if let Some(ip_with_mask) = rest.split_whitespace().next() {
                let ip = ip_with_mask.split('/').next().unwrap_or("");
                if is_ipv4(ip) {
                    return Some(ip.to_string());
                }
            }
        }
    }
    None
}

pub fn parse_adb_mdns_services(output: &str) -> Vec<WirelessAdbService> {
    output
        .lines()
        .filter_map(parse_adb_mdns_service_line)
        .collect()
}

fn parse_adb_mdns_service_line(line: &str) -> Option<WirelessAdbService> {
    let line = line.trim();
    if line.is_empty() || line.starts_with("List of discovered") {
        return None;
    }

    let service_type = if line.contains("_adb-tls-pairing._tcp") {
        WirelessAdbServiceType::Pairing
    } else if line.contains("_adb-tls-connect._tcp") {
        WirelessAdbServiceType::Connect
    } else {
        return None;
    };

    let endpoint = line.split_whitespace().rev().find_map(parse_endpoint)?;

    let name = line
        .split_whitespace()
        .next()
        .unwrap_or("Android")
        .trim_end_matches('.')
        .to_string();

    let service_prefix = match service_type {
        WirelessAdbServiceType::Pairing => "pairing",
        WirelessAdbServiceType::Connect => "connect",
    };

    Some(WirelessAdbService {
        id: format!("{service_prefix}:{}:{}", endpoint.0, endpoint.1),
        name,
        host: endpoint.0,
        port: endpoint.1,
        service_type,
    })
}

fn parse_endpoint(token: &str) -> Option<(String, u16)> {
    let endpoint = token.trim().trim_end_matches('.');
    let (host, port) = endpoint.rsplit_once(':')?;
    if !is_ipv4(host) {
        return None;
    }
    let port = port.parse::<u16>().ok()?;
    Some((host.to_string(), port))
}

fn is_ipv4(value: &str) -> bool {
    let parts: Vec<&str> = value.split('.').collect();
    parts.len() == 4 && parts.iter().all(|part| part.parse::<u8>().is_ok())
}

pub fn parse_scrcpy_encoders(output: &str) -> (Vec<String>, Vec<VideoCodec>) {
    let mut encoders = Vec::new();
    let mut codec_set = std::collections::HashSet::new();

    for line in output.lines() {
        let line = line.trim();
        if let Some(idx) = line.find("--video-codec=") {
            let rest = &line[idx + "--video-codec=".len()..];
            let tag = rest
                .split_whitespace()
                .next()
                .unwrap_or("")
                .trim_end_matches(',');
            if let Some(codec) = parse_video_codec_tag(tag) {
                codec_set.insert(codec);
            }
            if let Some(name) = rest.split_whitespace().nth(1) {
                let name = name.trim_end_matches(',');
                if !name.is_empty() {
                    encoders.push(name.to_string());
                }
            }
        }
    }

    let mut codecs: Vec<VideoCodec> = codec_set.into_iter().collect();
    codecs.sort_by(|a, b| {
        let order = |c: &VideoCodec| match c {
            VideoCodec::Av1 => 0,
            VideoCodec::H265 => 1,
            VideoCodec::H264 => 2,
        };
        order(a).cmp(&order(b))
    });

    (encoders, codecs)
}

pub fn parse_video_codec_tag(tag: &str) -> Option<VideoCodec> {
    match tag {
        "h264" => Some(VideoCodec::H264),
        "h265" => Some(VideoCodec::H265),
        "av1" => Some(VideoCodec::Av1),
        _ => None,
    }
}

pub fn parse_screen_resolution(s: &str) -> Option<(u32, u32)> {
    let parts: Vec<&str> = s.split(" × ").collect();
    if parts.len() == 2 {
        let w = parts[0].trim().parse::<u32>().ok()?;
        let h = parts[1].trim().parse::<u32>().ok()?;
        return Some((w, h));
    }
    let parts: Vec<&str> = s.split('x').collect();
    if parts.len() == 2 {
        let w = parts[0].trim().parse::<u32>().ok()?;
        let h = parts[1].trim().parse::<u32>().ok()?;
        return Some((w, h));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::android::types::WirelessAdbServiceType;

    #[test]
    fn parses_wireless_pairing_and_connect_services() {
        let output = r#"
List of discovered mdns services
adb-123456	_adb-tls-pairing._tcp.	192.168.1.23:37123
adb-123456	_adb-tls-connect._tcp.	192.168.1.23:42123
"#;

        let services = parse_adb_mdns_services(output);

        assert_eq!(services.len(), 2);
        assert_eq!(services[0].name, "adb-123456");
        assert_eq!(services[0].host, "192.168.1.23");
        assert_eq!(services[0].port, 37123);
        assert_eq!(services[0].service_type, WirelessAdbServiceType::Pairing);
        assert_eq!(services[1].service_type, WirelessAdbServiceType::Connect);
        assert_eq!(services[1].port, 42123);
    }

    #[test]
    fn ignores_non_adb_mdns_rows_and_invalid_endpoints() {
        let output = r#"
List of discovered mdns services
printer	_ipp._tcp.	192.168.1.50:631
adb-bad	_adb-tls-connect._tcp.	not-an-endpoint
adb-bad-port	_adb-tls-pairing._tcp.	192.168.1.24:abc
adb-ok	_adb-tls-connect._tcp.	192.168.1.25:39001
"#;

        let services = parse_adb_mdns_services(output);

        assert_eq!(services.len(), 1);
        assert_eq!(services[0].id, "connect:192.168.1.25:39001");
        assert_eq!(services[0].service_type, WirelessAdbServiceType::Connect);
    }
}
