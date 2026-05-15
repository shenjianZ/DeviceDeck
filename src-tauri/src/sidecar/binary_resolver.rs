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
                for file_name in executable_file_names(name) {
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
        let extension = executable_extension();
        let prefix = format!("{name}-");

        std::fs::read_dir(binaries_dir)
            .ok()?
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .find(|path| {
                let Some(file_name) = path.file_name().and_then(|value| value.to_str()) else {
                    return false;
                };
                file_name.starts_with(&prefix) && file_name.ends_with(extension) && path.is_file()
            })
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

    if let Some(target) = windows_target_triple() {
        names.push(format!("{name}-{target}.exe"));
    }

    names
}

#[cfg(not(windows))]
fn executable_file_names(name: &str) -> Vec<String> {
    vec![executable_name(name)]
}

#[cfg(all(windows, target_arch = "x86_64"))]
fn windows_target_triple() -> Option<&'static str> {
    Some("x86_64-pc-windows-msvc")
}

#[cfg(all(windows, target_arch = "aarch64"))]
fn windows_target_triple() -> Option<&'static str> {
    Some("aarch64-pc-windows-msvc")
}

#[cfg(all(windows, target_arch = "x86"))]
fn windows_target_triple() -> Option<&'static str> {
    Some("i686-pc-windows-msvc")
}

#[cfg(all(
    windows,
    not(any(target_arch = "x86_64", target_arch = "aarch64", target_arch = "x86"))
))]
fn windows_target_triple() -> Option<&'static str> {
    None
}

#[cfg(windows)]
fn executable_extension() -> &'static str {
    ".exe"
}

#[cfg(not(windows))]
fn executable_extension() -> &'static str {
    ""
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
}
