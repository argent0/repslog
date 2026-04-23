use crate::cli::SetAction;
use crate::repository::Repository;
use crate::error::Result;
use crate::utils::{print_table, read_stdin};

pub async fn handle_set(action: SetAction, repo: &Repository) -> Result<()> {
    match action {
        SetAction::Add { workout_exercise_id, reps, weight, duration, distance, rpe, notes } => {
            let id = if let Some(id) = workout_exercise_id {
                id
            } else if let Some(stdin_id) = read_stdin() {
                stdin_id.parse::<i64>().unwrap_or_else(|_| 0)
            } else {
                return Err(crate::error::RepslogError::Cli("No workout-exercise-id provided".into()));
            };
            
            let set_number = repo.get_next_set_number(id).await?;
            let set_id = repo.add_set(id, set_number, reps, weight, duration, distance, rpe, notes.as_deref()).await?;
            println!("Added set {} to workout-exercise {} with set ID {}", set_number, id, set_id);
        }
        SetAction::List { workout_exercise_id } => {
            let sets = repo.list_sets(workout_exercise_id).await?;
            let mut rows = Vec::new();
            for s in sets {
                rows.push(vec![
                    s.id.to_string(),
                    s.set_number.to_string(),
                    s.reps.map(|r| r.to_string()).unwrap_or_default(),
                    s.weight_kg.map(|w| format!("{:.2} kg", w)).unwrap_or_default(),
                    s.duration_seconds.map(|d| d.to_string()).unwrap_or_default(),
                    s.distance_km.map(|d| format!("{:.2} km", d)).unwrap_or_default(),
                    s.rpe.map(|r| r.to_string()).unwrap_or_default(),
                    s.notes.unwrap_or_default(),
                ]);
            }
            print_table(vec!["ID", "Set #", "Reps", "Weight", "Duration (s)", "Distance", "RPE", "Notes"], rows);
        }
        SetAction::Quick { workout_id, exercise_name_or_id } => {
            let exercise = repo.find_exercise_by_id_or_name(&exercise_name_or_id).await?;
            if let Some(ex) = exercise {
                let order = repo.get_max_order_for_workout(workout_id).await? + 1;
                let we_id = repo.add_workout_exercise(workout_id, ex.id, order, None).await?;
                let set_id = repo.add_set(we_id, 1, None, None, None, None, None, None).await?;
                println!("Added exercise {} to workout {} and created first set with ID {}", ex.name, workout_id, set_id);
            } else {
                println!("Exercise not found: {}", exercise_name_or_id);
            }
        }
    }
    Ok(())
}
