use sqlx::sqlite::{SqlitePool, SqliteConnectOptions};
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

pub async fn run_migrations(pool: &SqlitePool) -> Result<()> {
    let migrator = Migrator::new(Path::new("./migrations")).await?;
    migrator.run(pool).await?;
    Ok(())
}
