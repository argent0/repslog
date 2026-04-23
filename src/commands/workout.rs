use crate::cli::{WorkoutAction, WorkoutExerciseAction};
use crate::repository::Repository;
use crate::error::Result;
use crate::utils::{print_table, format_duration, format_pace};

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
                for (we, name) in exercises {
                    println!("\n{} (WE ID: {})", name, we.id);
                    if let Some(notes) = we.notes {
                        println!("Notes: {}", notes);
                    }
                    let sets = repo.list_sets(we.id).await?;
                    let mut set_rows = Vec::new();
                    for s in &sets {
                        let cluster_label = if let Some(cid) = s.cluster_id {
                            format!(" [C{}]", cid)
                        } else {
                            "".to_string()
                        };

                        let cardio_info = if s.avg_heart_rate_bpm.is_some() {
                            format!("HR: {}/{} | Pace: {} | Cal: {}", 
                                s.avg_heart_rate_bpm.map(|v| v.to_string()).unwrap_or_default(),
                                s.max_heart_rate_bpm.map(|v| v.to_string()).unwrap_or_default(),
                                s.avg_pace_min_per_km.map(|v| v.to_string()).unwrap_or_default(),
                                s.calories_burned.map(|v| v.to_string()).unwrap_or_default()
                            )
                        } else {
                            "".to_string()
                        };

                        set_rows.push(vec![
                            s.set_number.to_string() + &cluster_label,
                            s.reps.map(|r| r.to_string()).unwrap_or_default(),
                            s.weight_kg.map(|w| format!("{:.2} kg", w)).unwrap_or_default(),
                            s.distance_km.map(|d| format!("{:.2} km", d)).unwrap_or_default(),
                            s.duration_seconds.map(|d| format!("{}s", d)).unwrap_or_default(),
                            cardio_info,
                            s.notes.as_ref().cloned().unwrap_or_default(),
                        ]);
                    }
                    print_table(vec!["Set #", "Reps", "Weight", "Dist", "Dur", "Cardio", "Notes"], set_rows);

                    // Show Laps if available
                    for s in &sets {
                        if let Some(ref laps_json) = s.laps {
                            let laps = &laps_json.0;
                            if !laps.is_empty() {
                                println!("\nLap Breakdown (Set {}):", s.set_number);
                                let mut lap_rows = Vec::new();
                                for lap in laps {
                                    lap_rows.push(vec![
                                        format!("Lap {}", lap.lap_number),
                                        format!("{:.2} km", lap.distance_km),
                                        format_duration(lap.duration_seconds),
                                        format_pace(lap.pace_min_per_km),
                                    ]);
                                }
                                print_table(vec!["Lap", "Distance", "Time", "Pace"], lap_rows);
                            }
                        }
                    }
                }
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
