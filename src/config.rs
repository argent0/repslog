use std::path::PathBuf;
use directories::ProjectDirs;
use crate::error::{RepslogError, Result};

pub fn get_db_path() -> Result<PathBuf> {
    let proj_dirs = ProjectDirs::from("", "", "repslog")
        .ok_or_else(|| RepslogError::Config("Could not determine project directories".to_string()))?;
    
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
