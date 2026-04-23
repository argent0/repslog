use clap::Parser;
use repslog::cli::{Cli, Commands};
use repslog::repository::Repository;
use repslog::db::{setup_db, check_schema_version};
use repslog::error::Result;
use repslog::commands;
use colored::*;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    if let Err(e) = run(cli).await {
        eprintln!("{} {}", "Error:".red().bold(), e);
        std::process::exit(1);
    }
}

async fn run(cli: Cli) -> Result<()> {
    let pool = setup_db().await?;

    // Handle commands that don't require an up-to-date schema
    match &cli.command {
        Commands::Init => {
            commands::init::handle_init(&pool).await?;
            return Ok(());
        }
        Commands::Migrate { status, dry_run } => {
            commands::migrate::handle_migrate(&pool, *status, *dry_run).await?;
            return Ok(());
        }
        _ => {
            // All other commands require the schema to be up-to-date
            check_schema_version(&pool).await?;
        }
    }

    let repo = Repository::new(pool.clone());

    match cli.command {
        Commands::Exercise { action } => {
            commands::exercise::handle_exercise(action, &repo).await?;
        }
        Commands::Workout { action } => {
            commands::workout::handle_workout(action, &repo).await?;
        }
        Commands::Session { action } => {
            commands::workout::handle_workout(action, &repo).await?;
        }
        Commands::WorkoutExercise { action } => {
            commands::workout::handle_workout_exercise(action, &repo).await?;
        }
        Commands::Set { action } => {
            commands::set::handle_set(action, &repo).await?;
        }
        Commands::Stats { action } => {
            commands::stats::handle_stats(action, &repo).await?;
        }
        _ => unreachable!(),
    }

    Ok(())
}
