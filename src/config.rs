use crate::error::{RepslogError, Result};
use directories::ProjectDirs;
use std::path::PathBuf;

pub fn get_db_path() -> Result<PathBuf> {
    let proj_dirs = ProjectDirs::from("", "", "repslog").ok_or_else(|| {
        RepslogError::Config("Could not determine project directories".to_string())
    })?;

    let data_dir = proj_dirs.data_dir();
    if !data_dir.exists() {
        std::fs::create_dir_all(data_dir)?;
    }

    Ok(data_dir.join("repslog.db"))
}

pub fn get_db_url() -> Result<String> {
    let path = get_db_path()?;
    // For absolute paths on Unix, we need sqlite:///path/to/db
    Ok(format!("sqlite://{}", path.to_string_lossy()))
}

/// Resolve DB URL, using override path if provided (for --db flag).
/// Supports plain file paths (relative or absolute), ":memory:", or full "sqlite:..." URLs.
/// For file paths, ensures parent directory exists.
pub fn get_db_url_with_override(db_override: Option<&str>) -> Result<String> {
    if let Some(p) = db_override {
        if p == ":memory:" {
            return Ok("sqlite::memory:".to_string());
        }
        if p.starts_with("sqlite:") {
            return Ok(p.to_string());
        }
        // Treat as filesystem path
        let path = PathBuf::from(p);
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() && !parent.exists() {
                std::fs::create_dir_all(parent)?;
            }
        }
        Ok(format!("sqlite://{}", path.to_string_lossy()))
    } else {
        get_db_url()
    }
}
