use sqlx::sqlite::{SqlitePool, SqliteConnectOptions, SqlitePoolOptions};
use crate::error::{Result, RepslogError};
use crate::config::get_db_url;
use std::str::FromStr;
use std::path::Path;
use std::fs;
use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct Migration {
    pub version: i32,
    pub name: String,
    pub sql: String,
}

pub async fn setup_db() -> Result<SqlitePool> {
    let db_url = get_db_url()?;
    let options = SqliteConnectOptions::from_str(&db_url)?
        .create_if_missing(true)
        .foreign_keys(true);
    
    let pool = SqlitePool::connect_with(options).await?;
    
    // Ensure migrations table exists
    ensure_migrations_table(&pool).await?;
    
    Ok(pool)
}

pub async fn setup_test_db() -> Result<SqlitePool> {
    let options = SqliteConnectOptions::from_str("sqlite::memory:")?
        .foreign_keys(true);
    
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await?;
    
    ensure_migrations_table(&pool).await?;
    run_migrations(&pool).await?;
    Ok(pool)
}

async fn ensure_migrations_table(pool: &SqlitePool) -> Result<()> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS migrations (
            version     INTEGER PRIMARY KEY,
            name        TEXT NOT NULL,
            applied_at  TEXT DEFAULT CURRENT_TIMESTAMP,
            checksum    TEXT
        )"
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_current_version(pool: &SqlitePool) -> Result<i32> {
    let row: Option<(i32,)> = sqlx::query_as("SELECT MAX(version) FROM migrations")
        .fetch_optional(pool)
        .await?;
    
    Ok(row.and_then(|r| Some(r.0)).unwrap_or(0))
}

pub fn get_all_migrations() -> Result<Vec<Migration>> {
    let migrations_dir = Path::new("migrations");
    if !migrations_dir.exists() {
        return Ok(vec![]);
    }

    let mut migrations = Vec::new();
    for entry in fs::read_dir(migrations_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("sql") {
            let filename = path.file_name().and_then(|s| s.to_str()).unwrap();
            let parts: Vec<&str> = filename.split('_').collect();
            if let Ok(version) = parts[0].parse::<i32>() {
                let sql = fs::read_to_string(&path)?;
                migrations.push(Migration {
                    version,
                    name: filename.to_string(),
                    sql,
                });
            }
        }
    }

    migrations.sort_by_key(|m| m.version);
    Ok(migrations)
}

pub async fn run_migrations(pool: &SqlitePool) -> Result<Vec<Migration>> {
    let current_version = get_current_version(pool).await?;
    let all_migrations = get_all_migrations()?;
    let mut applied = Vec::new();

    for migration in all_migrations {
        if migration.version > current_version {
            let mut tx = pool.begin().await?;
            
            // Execute migration SQL
            // sqlx doesn't support executing multiple statements in one query easily with parameters,
            // but for migrations we can use raw execute on the connection.
            // Actually, SqlitePool::execute works fine for multiple statements.
            sqlx::query(&migration.sql).execute(&mut *tx).await?;

            // Record migration
            sqlx::query("INSERT INTO migrations (version, name) VALUES (?, ?)")
                .bind(migration.version)
                .bind(&migration.name)
                .execute(&mut *tx)
                .await?;

            tx.commit().await?;
            applied.push(migration);
        }
    }

    Ok(applied)
}

pub async fn check_schema_version(pool: &SqlitePool) -> Result<()> {
    let current_version = get_current_version(pool).await?;
    let all_migrations = get_all_migrations()?;
    let latest_version = all_migrations.last().map(|m| m.version).unwrap_or(0);

    if current_version < latest_version {
        return Err(RepslogError::MigrationRequired(current_version, latest_version));
    }

    Ok(())
}
