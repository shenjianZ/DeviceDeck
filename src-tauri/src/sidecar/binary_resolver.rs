use std::path::PathBuf;

use crate::core::error::AppError;
use crate::core::types::AppSettings;
use crate::sidecar::process_command;

pub struct BinaryResolver;

impl BinaryResolver {
    pub fn resolve_adb(settings: &AppSettings) -> Result<PathBuf, AppError> {
        Self::resolve_tool(
            "adb",
            settings.use_bundled_adb,
            &settings.custom_adb_path,
            AppError::adb_not_found,
        )
    }

    pub fn resolve_scrcpy(settings: &AppSettings) -> Result<PathBuf, AppError> {
        Self::resolve_tool(
            "scrcpy",
            settings.use_bundled_scrcpy,
            &settings.custom_scrcpy_path,
            AppError::scrcpy_not_found,
        )
    }

    fn resolve_tool(
        name: &str,
        use_bundled: bool,
        custom_path: &str,
        not_found: fn() -> AppError,
    ) -> Result<PathBuf, AppError> {
        if use_bundled {
            if let Some(path) = Self::find_bundled(name) {
                return Ok(path);
            }
        }

        if !custom_path.trim().is_empty() {
            let path = PathBuf::from(custom_path.trim());
            if path.is_file() {
                return Ok(path);
            }
            return Err(AppError::invalid_tool_path(custom_path));
        }

        if let Some(path) = Self::find_in_path(name) {
            return Ok(path);
        }

        Err(not_found())
    }

    fn find_bundled(name: &str) -> Option<PathBuf> {
        let mut candidates = Vec::new();

        if let Some(dev_binary) = Self::find_dev_sidecar(name) {
            candidates.push(dev_binary);
        }

        if let Ok(current_exe) = std::env::current_exe() {
            if let Some(dir) = current_exe.parent() {
                for file_name in bundled_file_names(name) {
                    candidates.push(dir.join(&file_name));
                    candidates.push(dir.join("sidecar").join(&file_name));
                    candidates.push(dir.join("binaries").join(&file_name));
                    candidates.push(dir.join("resources").join(&file_name));
                    candidates.push(dir.join("resources").join("binaries").join(&file_name));
                    candidates.push(dir.join("_up_").join("binaries").join(&file_name));
                }
            }
        }

        candidates.into_iter().find(|path| path.is_file())
    }

    fn find_dev_sidecar(name: &str) -> Option<PathBuf> {
        let binaries_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("binaries");

        dev_sidecar_file_names(name)
            .into_iter()
            .map(|file_name| binaries_dir.join(file_name))
            .find(|path| path.is_file())
    }

    fn find_in_path(name: &str) -> Option<PathBuf> {
        let exe_name = executable_name(name);
        let output = process_command::new_std_command(which_command())
            .arg(&exe_name)
            .output()
            .ok()?;

        if !output.status.success() {
            return None;
        }

        String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(PathBuf::from)
            .find(|path| path.is_file())
    }
}

#[cfg(windows)]
fn executable_name(name: &str) -> String {
    format!("{name}.exe")
}

#[cfg(not(windows))]
fn executable_name(name: &str) -> String {
    name.to_string()
}

#[cfg(windows)]
fn executable_file_names(name: &str) -> Vec<String> {
    let mut names = vec![executable_name(name)];

    if let Some(target) = target_triple() {
        names.push(format!("{name}-{target}.exe"));
    }

    names
}

#[cfg(not(windows))]
fn executable_file_names(name: &str) -> Vec<String> {
    let mut names = vec![executable_name(name)];

    if let Some(target) = target_triple() {
        names.push(format!("{name}-{target}"));
    }

    names
}

fn dev_sidecar_file_names(name: &str) -> Vec<String> {
    bundled_file_names(name)
}

fn bundled_file_names(name: &str) -> Vec<String> {
    let mut names = executable_file_names(name);

    if let Some(dir) = platform_binary_dir() {
        names.push(format!("{dir}/{}", executable_name(name)));
    }

    names
}

#[cfg(all(windows, target_arch = "x86_64"))]
fn platform_binary_dir() -> Option<&'static str> {
    Some("windows-x64")
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
fn platform_binary_dir() -> Option<&'static str> {
    Some("linux-x64")
}

#[cfg(all(target_os = "macos", target_arch = "x86_64"))]
fn platform_binary_dir() -> Option<&'static str> {
    Some("macos-x64")
}

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn platform_binary_dir() -> Option<&'static str> {
    Some("macos-aarch64")
}

#[cfg(all(not(any(
    all(windows, target_arch = "x86_64"),
    all(target_os = "linux", target_arch = "x86_64"),
    all(target_os = "macos", target_arch = "x86_64"),
    all(target_os = "macos", target_arch = "aarch64"),
))))]
fn platform_binary_dir() -> Option<&'static str> {
    None
}

#[cfg(all(windows, target_arch = "x86_64"))]
fn target_triple() -> Option<&'static str> {
    Some("x86_64-pc-windows-msvc")
}

#[cfg(all(windows, target_arch = "aarch64"))]
fn target_triple() -> Option<&'static str> {
    Some("aarch64-pc-windows-msvc")
}

#[cfg(all(windows, target_arch = "x86"))]
fn target_triple() -> Option<&'static str> {
    Some("i686-pc-windows-msvc")
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
fn target_triple() -> Option<&'static str> {
    Some("x86_64-unknown-linux-gnu")
}

#[cfg(all(target_os = "linux", target_arch = "aarch64"))]
fn target_triple() -> Option<&'static str> {
    Some("aarch64-unknown-linux-gnu")
}

#[cfg(all(target_os = "macos", target_arch = "x86_64"))]
fn target_triple() -> Option<&'static str> {
    Some("x86_64-apple-darwin")
}

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn target_triple() -> Option<&'static str> {
    Some("aarch64-apple-darwin")
}

#[cfg(all(not(any(
    all(windows, target_arch = "x86_64"),
    all(windows, target_arch = "aarch64"),
    all(windows, target_arch = "x86"),
    all(target_os = "linux", target_arch = "x86_64"),
    all(target_os = "linux", target_arch = "aarch64"),
    all(target_os = "macos", target_arch = "x86_64"),
    all(target_os = "macos", target_arch = "aarch64"),
))))]
fn target_triple() -> Option<&'static str> {
    None
}

#[cfg(windows)]
fn which_command() -> &'static str {
    "where"
}

#[cfg(not(windows))]
fn which_command() -> &'static str {
    "which"
}

#[cfg(all(test, windows))]
mod tests {
    use super::*;

    #[test]
    fn executable_file_names_include_plain_and_target_specific_windows_names() {
        let names = executable_file_names("adb");

        assert!(names.contains(&"adb.exe".to_string()));
        assert!(names.contains(&"adb-x86_64-pc-windows-msvc.exe".to_string()));
    }

    #[test]
    fn target_triple_returns_current_windows_target() {
        #[cfg(target_arch = "x86_64")]
        assert_eq!(target_triple(), Some("x86_64-pc-windows-msvc"));

        #[cfg(target_arch = "aarch64")]
        assert_eq!(target_triple(), Some("aarch64-pc-windows-msvc"));

        #[cfg(target_arch = "x86")]
        assert_eq!(target_triple(), Some("i686-pc-windows-msvc"));
    }

    #[test]
    fn dev_sidecar_file_names_include_platform_directory_candidates() {
        let names = dev_sidecar_file_names("adb");

        assert!(names.contains(&"adb.exe".to_string()));
        assert!(names.contains(&"windows-x64/adb.exe".to_string()));
    }
}

#[cfg(all(
    test,
    not(windows),
    any(target_os = "linux", target_os = "macos"),
    any(target_arch = "x86_64", target_arch = "aarch64")
))]
mod tests {
    use super::*;

    #[test]
    fn executable_file_names_include_plain_and_target_specific_unix_names() {
        let names = executable_file_names("adb");
        let target = target_triple().expect("supported non-Windows target triple");

        assert!(names.contains(&"adb".to_string()));
        assert!(names.contains(&format!("adb-{target}")));
    }
}
