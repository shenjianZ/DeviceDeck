use super::types::{RawDevice, WirelessAdbService, WirelessAdbServiceType};
use crate::core::types::{FileEntry, VideoCodec};

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

/// Parse `ls -la` or `ls -l` output from Android shell into FileEntry list.
/// Supports multiple Android ls formats:
///   drwxrwx--x  15 system system 4096 2025-01-15 10:30 dirname
///   -rw-rw----   1 root   sdcard_rw 1234 2025-01-15 10:30 file.txt
///   drwxrwx--x  15 system system 4096 2025-01-15-10:30 dirname
///   -rw-rw----   1 root   root 1234 file.txt  (minimal format)
pub fn parse_ls_la(output: &str, base_path: &str) -> Vec<FileEntry> {
    let base = base_path.trim_end_matches('/');
    let mut entries = Vec::new();

    for line in output.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("total") {
            continue;
        }

        // Must start with a permission-like string (d/l/-/b/c followed by rwx etc.)
        let first = match line.split_whitespace().next() {
            Some(f) => f,
            None => continue,
        };
        if first.len() < 2
            || !first
                .chars()
                .next()
                .map_or(false, |c| matches!(c, 'd' | '-' | 'l' | 'b' | 'c'))
        {
            continue;
        }

        if let Some(entry) = parse_ls_line(line, base) {
            entries.push(entry);
        }
    }

    entries.sort_by(|a, b| {
        b.is_directory
            .cmp(&a.is_directory)
            .then(a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });

    entries
}

fn parse_ls_line(line: &str, base: &str) -> Option<FileEntry> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 6 {
        return None;
    }

    let permissions = parts[0];
    let is_directory = permissions.starts_with('d');
    let is_symlink = permissions.starts_with('l');

    // Try standard format: perms links owner group size date... name
    // Find name start by detecting the date/time boundary
    if let Some(name_start) = find_name_start_standard(&parts) {
        let name = parts[name_start..].join(" ");
        let clean_name = if is_symlink {
            name.split(" -> ").next().unwrap_or(&name).to_string()
        } else {
            name
        };
        if clean_name == "." || clean_name == ".." {
            return None;
        }
        let size = parts.get(4).and_then(|s| s.parse::<u64>().ok());
        let modified = if name_start > 5 {
            Some(parts[5..name_start].join(" "))
        } else {
            None
        };
        let full_path = format!("{}/{}", base, clean_name);
        return Some(FileEntry {
            name: clean_name,
            path: full_path,
            is_directory,
            size,
            modified,
            permissions: Some(permissions.to_string()),
        });
    }

    // Fallback: minimal format: perms ? ? size name
    // Try to find the file name by scanning backwards from the end
    if let Some(size_idx) = find_size_index(&parts) {
        if size_idx + 1 < parts.len() {
            let name = parts[size_idx + 1..].join(" ");
            let clean_name = if is_symlink {
                name.split(" -> ").next().unwrap_or(&name).to_string()
            } else {
                name
            };
            if clean_name == "." || clean_name == ".." {
                return None;
            }
            let size = parts[size_idx].parse::<u64>().ok();
            let full_path = format!("{}/{}", base, clean_name);
            return Some(FileEntry {
                name: clean_name,
                path: full_path,
                is_directory,
                size,
                modified: None,
                permissions: Some(permissions.to_string()),
            });
        }
    }

    None
}

/// Standard ls -l format detection: find where date ends and filename begins.
/// Scans from index 5 onwards looking for date/time patterns.
fn find_name_start_standard(parts: &[&str]) -> Option<usize> {
    // parts layout: [0:perms] [1:links] [2:owner] [3:group] [4:size] [5+:date...] [N:name]
    if parts.len() < 7 {
        return None;
    }

    // Validate that parts[4] looks like a number (size field)
    if parts[4].parse::<u64>().is_err() {
        return None;
    }

    // Scan for date/time boundary starting from index 5
    for i in 5..parts.len() {
        // "HH:MM" time → next token is the name
        if parts[i].contains(':') && !parts[i].starts_with('-') {
            return Some(i + 1);
        }
        // "YYYY-MM-DD" date → check if next is time or name
        if is_date_token(parts[i]) {
            if i + 1 < parts.len() && parts[i + 1].contains(':') {
                // Date followed by time → name is after time
                return Some(i + 2);
            }
            // Date without time → name is next
            return Some(i + 1);
        }
        // "YYYY-MM-DD-HH:MM" combined format
        if parts[i].contains('-') && parts[i].contains(':') {
            return Some(i + 1);
        }
    }

    // If we can't find date patterns but have enough parts, try assuming 2 date tokens after size
    if parts.len() >= 7 {
        let candidate = 7;
        if candidate < parts.len() {
            return Some(candidate);
        }
    }

    None
}

fn find_size_index(parts: &[&str]) -> Option<usize> {
    // Search for a numeric field that could be the size
    for i in 4..parts.len().saturating_sub(1) {
        if parts[i].parse::<u64>().is_ok() {
            return Some(i);
        }
    }
    None
}

fn is_date_token(token: &str) -> bool {
    // "2025-01-15" style
    if token.chars().filter(|c| *c == '-').count() == 2 {
        let parts: Vec<&str> = token.split('-').collect();
        if parts.len() == 3 && parts.iter().all(|p| p.parse::<u32>().is_ok()) {
            return true;
        }
    }
    // "Jan".."Dec" month abbreviations
    matches!(
        token,
        "Jan"
            | "Feb"
            | "Mar"
            | "Apr"
            | "May"
            | "Jun"
            | "Jul"
            | "Aug"
            | "Sep"
            | "Oct"
            | "Nov"
            | "Dec"
    )
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
