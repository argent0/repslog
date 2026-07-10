use clap::Parser;
use colored::*;
use repslog::app_config;
use repslog::cli::{Cli, Commands};
use repslog::commands;
use repslog::db::{check_schema_version, setup_db};
use repslog::error::Result;
use repslog::repository::Repository;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    if let Err(e) = run(cli).await {
        eprintln!("{} {}", "Error:".red().bold(), e);
        std::process::exit(1);
    }
}

async fn run(cli: Cli) -> Result<()> {
    let json = cli.json;
    let config_override = cli.config.clone();

    // Config commands do not need the database
    if matches!(&cli.command, Commands::Config { .. }) {
        if let Commands::Config { action } = cli.command {
            return commands::config_cmd::handle_config(action, json);
        }
        unreachable!();
    }

    let loaded = app_config::load(config_override.as_deref())?;
    let limits = &loaded.config.sanity;

    let pool = setup_db(cli.db.as_deref()).await?;

    // Handle commands that don't require an up-to-date schema
    match &cli.command {
        Commands::Init { dry_run } => {
            commands::init::handle_init(&pool, *dry_run, json).await?;
            return Ok(());
        }
        Commands::Migrate {
            status,
            dry_run,
            force,
        } => {
            commands::migrate::handle_migrate(&pool, *status, *dry_run, *force, json).await?;
            return Ok(());
        }
        _ => {
            check_schema_version(&pool).await?;
        }
    }

    let repo = Repository::new(pool.clone());

    match cli.command {
        Commands::Exercise { action } => {
            commands::exercise::handle_exercise(action, &repo, json).await?;
        }
        Commands::Workout { action } => {
            commands::workout::handle_workout(action, &repo, limits, json).await?;
        }
        Commands::Session { action } => {
            commands::workout::handle_workout(action, &repo, limits, json).await?;
        }
        Commands::WorkoutExercise { action } => {
            commands::workout::handle_workout_exercise(action, &repo, json).await?;
        }
        Commands::Set { action } => {
            commands::set::handle_set(action, &repo, limits, json).await?;
        }
        Commands::Stats { action } => {
            commands::stats::handle_stats(action, &repo, json).await?;
        }
        Commands::Import { action } => {
            commands::import::handle_import(action, &repo, limits, json).await?;
        }
        Commands::Config { .. } | Commands::Init { .. } | Commands::Migrate { .. } => {
            unreachable!()
        }
    }

    Ok(())
}
