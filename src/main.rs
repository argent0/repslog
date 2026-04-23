mod cli;
mod config;
mod db;
mod error;
mod models;
mod repository;
mod utils;
mod commands;

use clap::Parser;
use crate::cli::{Cli, Commands};
use crate::repository::Repository;
use crate::db::setup_db;
use crate::error::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let pool = setup_db().await?;
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
        Commands::Init => {
            commands::init::handle_init(&pool).await?;
        }
    }

    Ok(())
}
