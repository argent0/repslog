use crate::config::get_db_url_with_override;
use crate::error::{RepslogError, Result};
use crate::utils::normalize_exercise_name_lenient;
use serde::Serialize;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use sqlx::{Executor, Row};
use std::collections::BTreeMap;
use std::io::{self, BufRead, IsTerminal, Write};
use std::str::FromStr;

#[derive(Debug, Serialize, Clone)]
pub struct Migration {
    pub version: i32,
    pub name: String,
    pub sql: String,
}

/// Options controlling how migrations run (especially data migrations).
#[derive(Debug, Clone, Copy)]
pub struct MigrationOptions {
    pub force: bool,
    /// When true and stdin is a TTY, prompt on exercise case-collision merges.
    pub interactive: bool,
    /// Suppress human-readable merge logs (e.g. JSON migrate mode).
    pub quiet: bool,
}

impl Default for MigrationOptions {
    fn default() -> Self {
        Self {
            force: false,
            interactive: false,
            quiet: true,
        }
    }
}

pub async fn setup_db(db_path: Option<&str>) -> Result<SqlitePool> {
    let db_url = get_db_url_with_override(db_path)?;
    let options = SqliteConnectOptions::from_str(&db_url)?
        .create_if_missing(true)
        .foreign_keys(true);

    let pool = SqlitePool::connect_with(options).await?;

    // Ensure migrations table exists
    ensure_migrations_table(&pool).await?;

    Ok(pool)
}

pub async fn setup_test_db() -> Result<SqlitePool> {
    let options = SqliteConnectOptions::from_str("sqlite::memory:")?.foreign_keys(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await?;

    ensure_migrations_table(&pool).await?;
    run_migrations(&pool, MigrationOptions::default()).await?;
    Ok(pool)
}

async fn ensure_migrations_table(pool: &SqlitePool) -> Result<()> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS migrations (
            version     INTEGER PRIMARY KEY,
            name        TEXT NOT NULL,
            applied_at  TEXT DEFAULT CURRENT_TIMESTAMP,
            checksum    TEXT
        )",
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_current_version(pool: &SqlitePool) -> Result<i32> {
    let row: Option<(i32,)> = sqlx::query_as("SELECT MAX(version) FROM migrations")
        .fetch_optional(pool)
        .await?;

    Ok(row.map(|r| r.0).unwrap_or(0))
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

pub async fn run_migrations(pool: &SqlitePool, opts: MigrationOptions) -> Result<Vec<Migration>> {
    let current_version = if opts.force {
        0
    } else {
        get_current_version(pool).await?
    };
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
                    if opts.force {
                        let msg = e.to_string();
                        if msg.contains("duplicate column name") || msg.contains("already exists") {
                            continue;
                        }
                    }
                    return Err(e.into());
                }
            }

            // Data migration: lowercase every exercise name; merge case collisions.
            if migration.version == 12 {
                lowercase_exercise_names_in_tx(&mut tx, opts).await?;
            }

            // Record migration (upsert if forced)
            sqlx::query(
                "INSERT INTO migrations (version, name) VALUES (?, ?) 
                         ON CONFLICT(version) DO UPDATE SET name=excluded.name, applied_at=CURRENT_TIMESTAMP",
            )
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

/// Convenience for call sites that only need force (non-interactive, quiet).
pub async fn run_migrations_force(pool: &SqlitePool, force: bool) -> Result<Vec<Migration>> {
    run_migrations(
        pool,
        MigrationOptions {
            force,
            interactive: false,
            quiet: true,
        },
    )
    .await
}

#[derive(Debug, Clone)]
struct ExerciseRow {
    id: i64,
    name: String,
    category: String,
    use_count: i64,
}

/// Lowercase all exercise names; merge rows that collide on the normalized key.
async fn lowercase_exercise_names_in_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    opts: MigrationOptions,
) -> Result<()> {
    let rows = sqlx::query(
        "SELECT e.id, e.name, e.category,
                (SELECT COUNT(*) FROM workout_exercises we WHERE we.exercise_id = e.id) AS use_count
         FROM exercises e
         ORDER BY e.id",
    )
    .fetch_all(&mut **tx)
    .await?;

    let mut exercises: Vec<ExerciseRow> = rows
        .into_iter()
        .map(|r| ExerciseRow {
            id: r.get("id"),
            name: r.get("name"),
            category: r.get("category"),
            use_count: r.get("use_count"),
        })
        .collect();

    // Group by normalized lowercase key
    let mut groups: BTreeMap<String, Vec<ExerciseRow>> = BTreeMap::new();
    for ex in exercises.drain(..) {
        let key = normalize_exercise_name_lenient(&ex.name);
        groups.entry(key).or_default().push(ex);
    }

    for (target_name, mut group) in groups {
        if group.len() == 1 {
            let ex = &group[0];
            if ex.name != target_name {
                sqlx::query("UPDATE exercises SET name = ? WHERE id = ?")
                    .bind(&target_name)
                    .bind(ex.id)
                    .execute(&mut **tx)
                    .await?;
                if !opts.quiet {
                    eprintln!(
                        "  renamed exercise id {} '{}' → '{}'",
                        ex.id, ex.name, target_name
                    );
                }
            }
            continue;
        }

        // Collision: multiple rows map to the same lowercase name
        group.sort_by(|a, b| b.use_count.cmp(&a.use_count).then_with(|| a.id.cmp(&b.id)));

        let survivor_id = if opts.interactive && io::stdin().is_terminal() {
            prompt_merge_survivor(&target_name, &group)?
        } else {
            group[0].id
        };

        let survivor = group.iter().find(|e| e.id == survivor_id).ok_or_else(|| {
            RepslogError::Cli(format!(
                "Invalid survivor id {} for exercise '{}'",
                survivor_id, target_name
            ))
        })?;

        if !opts.quiet {
            eprintln!(
                "  merging {} exercises into '{}' (keeping id {})",
                group.len(),
                target_name,
                survivor.id
            );
            for e in &group {
                if e.id != survivor.id {
                    eprintln!("    - drop id {} '{}' ({} uses)", e.id, e.name, e.use_count);
                }
            }
        }

        for e in &group {
            if e.id == survivor.id {
                continue;
            }
            // Re-point workout_exercises, then delete loser
            sqlx::query("UPDATE workout_exercises SET exercise_id = ? WHERE exercise_id = ?")
                .bind(survivor.id)
                .bind(e.id)
                .execute(&mut **tx)
                .await?;
            sqlx::query("DELETE FROM exercises WHERE id = ?")
                .bind(e.id)
                .execute(&mut **tx)
                .await?;
        }

        if survivor.name != target_name {
            sqlx::query("UPDATE exercises SET name = ? WHERE id = ?")
                .bind(&target_name)
                .bind(survivor.id)
                .execute(&mut **tx)
                .await?;
        }
    }

    // Final safety: reject any remaining uppercase names
    let bad: Option<(i64, String)> =
        sqlx::query_as("SELECT id, name FROM exercises WHERE name != lower(name) LIMIT 1")
            .fetch_optional(&mut **tx)
            .await?;
    // SQLite lower() is ASCII; also check for any uppercase Unicode letter
    let all: Vec<(i64, String)> = sqlx::query_as("SELECT id, name FROM exercises")
        .fetch_all(&mut **tx)
        .await?;
    for (id, name) in all {
        if name.chars().any(|c| c.is_uppercase()) {
            return Err(RepslogError::Cli(format!(
                "Migration 012 left exercise id {} with uppercase in name: '{}'",
                id, name
            )));
        }
    }
    let _ = bad;

    Ok(())
}

fn prompt_merge_survivor(target_name: &str, group: &[ExerciseRow]) -> Result<i64> {
    let stdout = io::stdout();
    let mut out = stdout.lock();
    let stdin = io::stdin();
    let mut lines = stdin.lock().lines();

    writeln!(
        out,
        "\nCase-collision for exercise name '{}': multiple catalog rows would become the same name.",
        target_name
    )
    .map_err(RepslogError::Io)?;
    writeln!(
        out,
        "Choose which exercise id to keep (others are merged into it):"
    )
    .map_err(RepslogError::Io)?;
    for (i, e) in group.iter().enumerate() {
        writeln!(
            out,
            "  [{}] id={} name='{}' category={} uses={}",
            i + 1,
            e.id,
            e.name,
            e.category,
            e.use_count
        )
        .map_err(RepslogError::Io)?;
    }
    writeln!(
        out,
        "Enter choice 1-{} or 'auto' (most uses, then lowest id) [auto]: ",
        group.len()
    )
    .map_err(RepslogError::Io)?;
    out.flush().map_err(RepslogError::Io)?;

    let line = match lines.next() {
        Some(Ok(s)) => s,
        Some(Err(e)) => return Err(RepslogError::Io(e)),
        None => String::new(),
    };
    let choice = line.trim();
    if choice.is_empty() || choice.eq_ignore_ascii_case("auto") {
        return Ok(group[0].id);
    }
    if let Ok(n) = choice.parse::<usize>() {
        if n >= 1 && n <= group.len() {
            return Ok(group[n - 1].id);
        }
    }
    // Also allow raw id
    if let Ok(id) = choice.parse::<i64>() {
        if group.iter().any(|e| e.id == id) {
            return Ok(id);
        }
    }
    Err(RepslogError::Cli(format!(
        "Invalid merge choice '{}'. Re-run `repslog migrate` and pick 1-{} or auto.",
        choice,
        group.len()
    )))
}

pub async fn check_schema_version(pool: &SqlitePool) -> Result<()> {
    let current_version = get_current_version(pool).await?;
    let all_migrations = get_all_migrations()?;
    let latest_version = all_migrations.last().map(|m| m.version).unwrap_or(0);

    if current_version < latest_version {
        return Err(RepslogError::MigrationRequired(
            current_version,
            latest_version,
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqliteConnectOptions;
    use std::str::FromStr;

    async fn empty_pool() -> SqlitePool {
        let options = SqliteConnectOptions::from_str("sqlite::memory:")
            .unwrap()
            .foreign_keys(true);
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await
            .unwrap();
        ensure_migrations_table(&pool).await.unwrap();
        pool
    }

    #[tokio::test]
    async fn migration_12_lowercases_and_merges() {
        let pool = empty_pool().await;
        // Apply migrations up through 11 only by running all then... actually run all
        // First seed a pre-12 state: run migrations 1-11 by temporarily...
        // Simpler: run all migrations on empty, then insert title-case via raw SQL
        // after rolling? Better: apply all migrations, then the names from init aren't there.
        // Build schema only via run_migrations, insert colliding names with raw SQL
        // BEFORE migration 12 by applying migrations manually.

        let all = get_all_migrations().unwrap();
        for m in &all {
            if m.version >= 12 {
                break;
            }
            let mut tx = pool.begin().await.unwrap();
            for statement in m.sql.split(';') {
                let s = statement.trim();
                if s.is_empty() {
                    continue;
                }
                tx.execute(s).await.unwrap();
            }
            sqlx::query("INSERT INTO migrations (version, name) VALUES (?, ?)")
                .bind(m.version)
                .bind(&m.name)
                .execute(&mut *tx)
                .await
                .unwrap();
            tx.commit().await.unwrap();
        }

        // Insert Title Case + lowercase collision
        sqlx::query(
            "INSERT INTO exercises (id, name, category, load_type, is_custom) VALUES
             (1, 'Running', 'cardio', 'none', 0),
             (2, 'running', 'cardio', 'none', 1),
             (3, 'Pushups', 'calisthenics', 'body_mass', 0)",
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query("INSERT INTO workouts (id, started_at) VALUES (1, '2026-01-01 10:00:00')")
            .execute(&pool)
            .await
            .unwrap();
        // Two workout_exercises on different exercise ids that will merge
        sqlx::query(
            "INSERT INTO workout_exercises (workout_id, exercise_id, \"order\") VALUES
             (1, 1, 1),
             (1, 2, 2)",
        )
        .execute(&pool)
        .await
        .unwrap();

        run_migrations(
            &pool,
            MigrationOptions {
                force: false,
                interactive: false,
                quiet: true,
            },
        )
        .await
        .unwrap();

        let names: Vec<(i64, String)> =
            sqlx::query_as("SELECT id, name FROM exercises ORDER BY id")
                .fetch_all(&pool)
                .await
                .unwrap();
        // One running (survivor id 1 has more or equal uses - both have 1 use, lower id wins)
        // Actually both have use_count 1, sort by use_count desc then id asc → id 1 wins
        assert!(names.iter().any(|(id, n)| *id == 1 && n == "running"));
        assert!(!names.iter().any(|(_, n)| n == "Running"));
        assert!(!names.iter().any(|(id, _)| *id == 2));
        assert!(names.iter().any(|(_, n)| n == "pushups"));

        let we_count: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM workout_exercises WHERE exercise_id = 1")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(we_count.0, 2);

        assert_eq!(get_current_version(&pool).await.unwrap(), 12);
    }
}
