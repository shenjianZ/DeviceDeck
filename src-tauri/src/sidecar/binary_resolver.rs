use std::path::PathBuf;

use crate::core::error::AppError;
use crate::core::types::AppSettings;

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
        let exe_name = executable_name(name);
        let mut candidates = Vec::new();

        if let Some(dev_binary) = Self::find_dev_sidecar(name) {
            candidates.push(dev_binary);
        }

        if let Ok(current_exe) = std::env::current_exe() {
            if let Some(dir) = current_exe.parent() {
                candidates.push(dir.join(&exe_name));
                candidates.push(dir.join("sidecar").join(&exe_name));
                candidates.push(dir.join("binaries").join(&exe_name));
                candidates.push(dir.join("resources").join(&exe_name));
                candidates.push(dir.join("resources").join("binaries").join(&exe_name));
                candidates.push(dir.join("_up_").join("binaries").join(&exe_name));
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
        let output = std::process::Command::new(which_command())
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
