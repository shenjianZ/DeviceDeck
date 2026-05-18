use std::path::PathBuf;

/// Returns the path to ~/.devicedeck/
pub fn devicedeck_data_dir() -> Result<PathBuf, String> {
    let home = dirs::home_dir().ok_or("Cannot get user home directory")?;
    Ok(home.join(".devicedeck"))
}

/// Ensures the directory exists and returns the path.
pub fn ensure_data_dir() -> Result<PathBuf, String> {
    let dir = devicedeck_data_dir()?;
    std::fs::create_dir_all(&dir)
        .map_err(|e| format!("Failed to create data directory {}: {e}", dir.display()))?;
    Ok(dir)
}
