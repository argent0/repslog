use crate::db::{get_all_migrations, get_current_version, run_migrations};
use crate::error::Result;
use colored::*;
use serde::Serialize;
use sqlx::SqlitePool;

pub async fn handle_migrate(
    pool: &SqlitePool,
    status: bool,
    dry_run: bool,
    force: bool,
    json: bool,
) -> Result<()> {
    let current_version = get_current_version(pool).await?;
    let all_migrations = get_all_migrations()?;
    let latest_version = all_migrations.last().map(|m| m.version).unwrap_or(0);

    if status {
        if json {
            #[derive(Serialize)]
            struct MigrateStatus {
                current_version: i32,
                latest_version: i32,
                up_to_date: bool,
            }
            let s = serde_json::to_string_pretty(&MigrateStatus {
                current_version,
                latest_version,
                up_to_date: current_version >= latest_version,
            })?;
            println!("{}", s);
        } else {
            println!("{} {}", "Current schema version:".bold(), current_version);
            println!("{} {}", "Latest available version:".bold(), latest_version);
            if current_version < latest_version {
                println!(
                    "{}",
                    "Updates available. Run `repslog migrate` to apply.".yellow()
                );
            } else {
                println!("{}", "Database is up-to-date.".green());
            }
        }
        return Ok(());
    }

    if current_version >= latest_version && !force {
        if json {
            println!(
                r#"{{"success": true, "current_version": {}, "latest_version": {}, "applied": 0}}"#,
                current_version, latest_version
            );
        } else {
            println!(
                "{} ({}). No changes needed.",
                "Database is already at the latest version".green(),
                current_version,
            );
        }
        return Ok(());
    }

    let pending: Vec<_> = all_migrations
        .clone()
        .into_iter()
        .filter(|m| force || m.version > current_version)
        .collect();

    if dry_run {
        if json {
            #[derive(Serialize)]
            struct Dry {
                dry_run: bool,
                from_version: i32,
                to_version: i32,
                force: bool,
                pending_migrations: Vec<String>,
            }
            let names: Vec<String> = pending.iter().map(|m| m.name.clone()).collect();
            let s = serde_json::to_string_pretty(&Dry {
                dry_run: true,
                from_version: current_version,
                to_version: latest_version,
                force,
                pending_migrations: names,
            })?;
            println!("{}", s);
        } else {
            if force {
                println!(
                    "{} forces re-execution of ALL migrations.",
                    "Dry run:".cyan()
                );
            } else {
                println!(
                    "{} {} to {}",
                    "Dry run: migrating from version".cyan(),
                    current_version,
                    latest_version
                );
            }
            println!("{}", "The following migrations would be applied:".bold());
            for m in &pending {
                println!("  - {}", m.name);
            }
            println!("{}", "No changes were made.".yellow());
        }
        return Ok(());
    }

    if json {
        // still perform, but output json at end; suppress human during? keep simple, do prints only if !json
    }
    if !json {
        if force {
            println!(
                "Force-migrating all {} available migrations...",
                pending.len()
            );
        } else {
            println!(
                "Migrating from version {} to {}...",
                current_version, latest_version
            );
        }
    }

    let applied = run_migrations(pool, force).await?;

    if json {
        println!(
            r#"{{"success": true, "from_version": {}, "to_version": {}, "migrations_applied": {}}}"#,
            current_version,
            latest_version,
            applied.len()
        );
    } else {
        println!(
            "{} Successfully migrated from version {} to version {} ({} migrations applied).",
            "✔".green(),
            current_version,
            latest_version,
            applied.len()
        );
    }

    Ok(())
}
