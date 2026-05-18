use std::path::PathBuf;

/// Returns the path to ~/.devicedeck/
pub fn devicedeck_data_dir() -> Result<PathBuf, String> {
    let home = dirs::home_dir().ok_or("无法获取用户主目录")?;
    Ok(home.join(".devicedeck"))
}

/// Ensures the directory exists and returns the path.
pub fn ensure_data_dir() -> Result<PathBuf, String> {
    let dir = devicedeck_data_dir()?;
    std::fs::create_dir_all(&dir)
        .map_err(|e| format!("无法创建数据目录 {}: {e}", dir.display()))?;
    Ok(dir)
}
