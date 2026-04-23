use thiserror::Error;

#[derive(Error, Debug)]
pub enum RepslogError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Migration error: {0}")]
    Migration(#[from] sqlx::migrate::MigrateError),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("CLI error: {0}")]
    Cli(String),

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
