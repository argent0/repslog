use thiserror::Error;

#[derive(Error, Debug)]
pub enum RepslogError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Database schema is outdated (version {0}). Please run `repslog migrate` first. (Latest version: {1})")]
    MigrationRequired(i32, i32),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("CLI error: {0}")]
    Cli(String),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Other: {0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, RepslogError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = RepslogError::Config("test config error".into());
        assert_eq!(format!("{}", err), "Configuration error: test config error");
    }
}
