use crate::cli::{WorkoutAction, WorkoutExerciseAction};
use crate::repository::Repository;
use crate::error::Result;
use crate::utils::print_table;

pub async fn handle_workout(action: WorkoutAction, repo: &Repository) -> Result<()> {
    match action {
        WorkoutAction::Create { workout_type, notes, date } => {
            // Try to parse to ensure it's a valid date (YYYY-MM-DD)
            if chrono::NaiveDate::parse_from_str(&date, "%Y-%m-%d").is_err() && 
               chrono::NaiveDateTime::parse_from_str(&date, "%Y-%m-%d %H:%M:%S").is_err() {
                return Err(crate::error::RepslogError::Cli("Invalid date format. Use YYYY-MM-DD or 'YYYY-MM-DD HH:MM:SS'".to_string()));
            }
            let id = repo.create_workout(workout_type.as_deref(), notes.as_deref(), Some(&date)).await?;
            println!("Created workout with ID {}", id);
        }
        WorkoutAction::List { limit, days } => {
            let workouts = repo.list_workouts(limit, days).await?;
            let mut rows = Vec::new();
            for w in workouts {
                rows.push(vec![
                    w.id.to_string(),
                    w.started_at,
                    w.finished_at.unwrap_or_else(|| "IN PROGRESS".to_string()),
                    w.workout_type.unwrap_or_default(),
                    w.duration_minutes.map(|d| d.to_string()).unwrap_or_default(),
                ]);
            }
            print_table(vec!["ID", "Started At", "Finished At", "Type", "Duration (min)"], rows);
        }
        WorkoutAction::View { workout_id } => {
            let workout = repo.get_workout(workout_id).await?;
            if let Some(w) = workout {
                println!("Workout ID: {}", w.id);
                println!("Type: {}", w.workout_type.unwrap_or_default());
                println!("Started: {}", w.started_at);
                println!("Finished: {}", w.finished_at.unwrap_or_else(|| "IN PROGRESS".to_string()));
                println!("Notes: {}", w.notes.unwrap_or_default());
                
                let exercises = repo.list_workout_exercises(workout_id).await?;
                println!("\nExercises:");
                let mut rows = Vec::new();
                for (we, name) in exercises {
                    let sets = repo.list_sets(we.id).await?;
                    rows.push(vec![
                        we.id.to_string(),
                        name,
                        sets.len().to_string(),
                        we.notes.unwrap_or_default(),
                    ]);
                }
                print_table(vec!["WE ID", "Exercise", "Sets", "Notes"], rows);
            } else {
                println!("Workout not found");
            }
        }
        WorkoutAction::Finish { workout_id, duration, feeling } => {
            repo.finish_workout(workout_id, duration, feeling).await?;
            println!("Finished workout {}", workout_id);
        }
        WorkoutAction::Current => {
            let workout = repo.get_current_workout().await?;
            if let Some(w) = workout {
                println!("Active workout ID: {}", w.id);
                println!("Type: {}", w.workout_type.unwrap_or_default());
                println!("Started: {}", w.started_at);
            } else {
                println!("No active workout found");
            }
        }
        WorkoutAction::Delete { workout_id } => {
            repo.delete_workout(workout_id).await?;
            println!("Deleted workout {}", workout_id);
        }
    }
    Ok(())
}

pub async fn handle_workout_exercise(action: WorkoutExerciseAction, repo: &Repository) -> Result<()> {
    match action {
        WorkoutExerciseAction::Add { workout_id, exercise_id_or_name, order } => {
            let exercise = repo.find_exercise_by_id_or_name(&exercise_id_or_name).await?;
            if let Some(ex) = exercise {
                let order = if let Some(o) = order {
                    o
                } else {
                    repo.get_max_order_for_workout(workout_id).await? + 1
                };
                let id = repo.add_workout_exercise(workout_id, ex.id, order, None).await?;
                println!("Added exercise {} (ID: {}) to workout {} with WE ID {}", ex.name, ex.id, workout_id, id);
            } else {
                println!("Exercise not found: {}", exercise_id_or_name);
            }
        }
        WorkoutExerciseAction::List { workout_id } => {
            let exercises = repo.list_workout_exercises(workout_id).await?;
            let mut rows = Vec::new();
            for (we, name) in exercises {
                rows.push(vec![
                    we.id.to_string(),
                    name,
                    we.order.to_string(),
                    we.notes.unwrap_or_default(),
                ]);
            }
            print_table(vec!["WE ID", "Exercise", "Order", "Notes"], rows);
        }
    }
    Ok(())
}
