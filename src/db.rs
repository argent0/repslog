use sqlx::sqlite::{SqlitePool, SqliteConnectOptions, SqlitePoolOptions};
use sqlx::migrate::Migrator;
use crate::error::Result;
use crate::config::get_db_url;
use std::str::FromStr;
use std::path::Path;

pub async fn setup_db() -> Result<SqlitePool> {
    let db_url = get_db_url()?;
    let options = SqliteConnectOptions::from_str(&db_url)?
        .create_if_missing(true)
        .foreign_keys(true);
    
    let pool = SqlitePool::connect_with(options).await?;
    
    Ok(pool)
}

pub async fn setup_test_db() -> Result<SqlitePool> {
    // Unique in-memory DB for each call to prevent interference
    let options = SqliteConnectOptions::from_str("sqlite::memory:")?
        .foreign_keys(true);
    
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await?;
    
    run_migrations(&pool).await?;
    Ok(pool)
}

pub async fn run_migrations(pool: &SqlitePool) -> Result<()> {
    let migrator = Migrator::new(Path::new("./migrations")).await?;
    migrator.run(pool).await?;
    Ok(())
}
