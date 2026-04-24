use crate::repository::Repository;
use crate::error::Result;
use crate::db::run_migrations;
use sqlx::sqlite::SqlitePool;

pub async fn handle_init(pool: &SqlitePool, dry_run: bool) -> Result<()> {
    println!("Initializing database...");
    if !dry_run {
        run_migrations(pool, false).await?;
    }
    
    let repo = Repository::new(pool.clone());
    
    let default_exercises = vec![
        ("Pushups", "calisthenics", Some("[\"chest\", \"triceps\", \"shoulders\"]"), Some("bodyweight"), Some("Basic pushup")),
        ("Pullups", "calisthenics", Some("[\"back\", \"biceps\"]"), Some("bodyweight"), Some("Basic pullup")),
        ("Dips", "calisthenics", Some("[\"chest\", \"triceps\", \"shoulders\"]"), Some("bodyweight"), Some("Basic dip")),
        ("Squats", "calisthenics", Some("[\"legs\"]"), Some("bodyweight"), Some("Basic bodyweight squat")),
        ("Lunges", "calisthenics", Some("[\"legs\"]"), Some("bodyweight"), Some("Basic lunges")),
        ("Plank", "flexibility", Some("[\"core\"]"), Some("bodyweight"), Some("Timed plank")),
        ("Muscle Up", "calisthenics", Some("[\"back\", \"chest\", \"triceps\", \"biceps\"]"), Some("bodyweight"), Some("Advanced calisthenics move")),
        ("Bench Press", "strength", Some("[\"chest\", \"triceps\"]"), Some("barbell"), Some("Standard bench press")),
        ("Deadlift", "strength", Some("[\"back\", \"legs\"]"), Some("barbell"), Some("Standard deadlift")),
        ("Squat (Barbell)", "strength", Some("[\"legs\"]"), Some("barbell"), Some("Standard back squat")),
        ("Running", "cardio", Some("[\"legs\", \"cardiovascular\"]"), Some("none"), Some("Outdoor or treadmill run")),
    ];

    for (name, category, muscles, equipment, desc) in default_exercises {
        let existing = repo.list_exercises(Some(name.to_string()), None).await?;
        if existing.is_empty() {
            repo.add_exercise(name, category, muscles, equipment, desc, false, dry_run).await?;
            println!("Added exercise: {}", name);
        }
    }

    println!("Initialization complete!");
    Ok(())
}
