use sqlx::sqlite::{SqlitePool, SqliteConnectOptions, SqlitePoolOptions};
use sqlx::Executor;
use crate::error::{Result, RepslogError};
use crate::config::get_db_url;
use std::str::FromStr;
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
    run_migrations(&pool, false).await?;
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
    let migrator = sqlx::migrate!("./migrations");
    
    let mut migrations = Vec::new();
    for m in migrator.migrations.iter() {
        // sqlx migrations have i64 version, our Migration has i32
        migrations.push(Migration {
            version: m.version as i32,
            name: m.description.to_string(),
            sql: m.sql.to_string(),
        });
    }

    migrations.sort_by_key(|m| m.version);
    Ok(migrations)
}

pub async fn run_migrations(pool: &SqlitePool, force: bool) -> Result<Vec<Migration>> {
    let current_version = if force { 0 } else { get_current_version(pool).await? };
    let all_migrations = get_all_migrations()?;
    let mut applied = Vec::new();

    for migration in all_migrations {
        if migration.version > current_version {
            let mut tx = pool.begin().await?;
            
            // Execute migration SQL statements individually to handle errors more granularly
            let statements: Vec<&str> = migration.sql.split(';').collect();
            for statement in statements {
                let s = statement.trim();
                if s.is_empty() {
                    continue;
                }
                if let Err(e) = tx.execute(s).await {
                    if force {
                        let msg = e.to_string();
                        if msg.contains("duplicate column name") || msg.contains("already exists") {
                            continue;
                        }
                    }
                    return Err(e.into());
                }
            }

            // Record migration (upsert if forced)
            sqlx::query("INSERT INTO migrations (version, name) VALUES (?, ?) 
                         ON CONFLICT(version) DO UPDATE SET name=excluded.name, applied_at=CURRENT_TIMESTAMP")
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
