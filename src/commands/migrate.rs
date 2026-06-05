use crate::db::{get_all_migrations, get_current_version, run_migrations};
use crate::error::Result;
use colored::*;
use sqlx::SqlitePool;

pub async fn handle_migrate(
    pool: &SqlitePool,
    status: bool,
    dry_run: bool,
    force: bool,
) -> Result<()> {
    let current_version = get_current_version(pool).await?;
    let all_migrations = get_all_migrations()?;
    let latest_version = all_migrations.last().map(|m| m.version).unwrap_or(0);

    if status {
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
        return Ok(());
    }

    if current_version >= latest_version && !force {
        println!(
            "{} ({}) {}",
            "Database is already at the latest version".green(),
            current_version,
            "No changes needed."
        );
        return Ok(());
    }

    let pending: Vec<_> = all_migrations
        .clone()
        .into_iter()
        .filter(|m| force || m.version > current_version)
        .collect();

    if dry_run {
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
        return Ok(());
    }

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

    let applied = run_migrations(pool, force).await?;

    println!(
        "{} Successfully migrated from version {} to version {} ({} migrations applied).",
        "✔".green(),
        current_version,
        latest_version,
        applied.len()
    );

    Ok(())
}
