use crate::db::{run_migrations, MigrationOptions};
use crate::error::Result;
use crate::load_type::{BODY_MASS, EXTERNAL, NONE};
use crate::repository::Repository;
use sqlx::sqlite::SqlitePool;

pub async fn handle_init(pool: &SqlitePool, dry_run: bool, json: bool) -> Result<()> {
    if !json {
        println!("Initializing database...");
    }
    if !dry_run {
        run_migrations(
            pool,
            MigrationOptions {
                force: false,
                interactive: false,
                quiet: json,
            },
        )
        .await?;
    }

    let repo = Repository::new(pool.clone());

    #[allow(clippy::type_complexity)]
    let default_exercises: Vec<(&str, &str, Option<&str>, Option<&str>, &str, Option<&str>)> = vec![
        (
            "pushups",
            "calisthenics",
            Some("[\"chest\", \"triceps\", \"shoulders\"]"),
            None,
            BODY_MASS,
            Some("Basic pushup"),
        ),
        (
            "pullups",
            "calisthenics",
            Some("[\"back\", \"biceps\"]"),
            None,
            BODY_MASS,
            Some("Basic pullup"),
        ),
        (
            "dips",
            "calisthenics",
            Some("[\"chest\", \"triceps\", \"shoulders\"]"),
            None,
            BODY_MASS,
            Some("Basic dip"),
        ),
        (
            "squats",
            "calisthenics",
            Some("[\"legs\"]"),
            None,
            BODY_MASS,
            Some("Basic bodyweight squat"),
        ),
        (
            "lunges",
            "calisthenics",
            Some("[\"legs\"]"),
            None,
            BODY_MASS,
            Some("Basic lunges"),
        ),
        (
            "plank",
            "flexibility",
            Some("[\"core\"]"),
            None,
            BODY_MASS,
            Some("Timed plank"),
        ),
        (
            "muscle up",
            "calisthenics",
            Some("[\"back\", \"chest\", \"triceps\", \"biceps\"]"),
            None,
            BODY_MASS,
            Some("Advanced calisthenics move"),
        ),
        (
            "bench press",
            "strength",
            Some("[\"chest\", \"triceps\"]"),
            Some("barbell"),
            EXTERNAL,
            Some("Standard bench press"),
        ),
        (
            "deadlift",
            "strength",
            Some("[\"back\", \"legs\"]"),
            Some("barbell"),
            EXTERNAL,
            Some("Standard deadlift"),
        ),
        (
            "squat (barbell)",
            "strength",
            Some("[\"legs\"]"),
            Some("barbell"),
            EXTERNAL,
            Some("Standard back squat"),
        ),
        (
            "running",
            "cardio",
            Some("[\"legs\", \"cardiovascular\"]"),
            Some("none"),
            NONE,
            Some("Outdoor or treadmill run"),
        ),
    ];

    let mut added = Vec::new();
    for (name, category, muscles, equipment, load_type, desc) in default_exercises {
        let existing = repo.list_exercises(Some(name.to_string()), None).await?;
        if existing.is_empty() {
            repo.add_exercise(
                name, category, muscles, equipment, load_type, desc, false, dry_run,
            )
            .await?;
            if !json {
                println!("Added exercise: {}", name);
            }
            added.push(name.to_string());
        }
    }

    if json {
        #[derive(serde::Serialize)]
        struct InitResult {
            success: bool,
            dry_run: bool,
            added_exercises: Vec<String>,
        }
        let s = serde_json::to_string_pretty(&InitResult {
            success: true,
            dry_run,
            added_exercises: added,
        })?;
        println!("{}", s);
    } else {
        println!("Initialization complete!");
    }
    Ok(())
}
