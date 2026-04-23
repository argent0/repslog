use crate::cli::SetAction;
use crate::repository::Repository;
use crate::error::{Result, RepslogError};
use crate::utils::{print_table, read_stdin};

pub async fn handle_set(action: SetAction, repo: &Repository) -> Result<()> {
    match action {
        SetAction::Add { 
            workout_exercise_id, 
            reps, 
            weight, 
            duration, 
            distance, 
            rpe, 
            rir, 
            effective_reps, 
            rest_seconds,
            notes,
            avg_heart_rate,
            max_heart_rate,
            hr_zones,
            pace,
            calories,
        } => {
            let id = if let Some(id) = workout_exercise_id {
                id
            } else if let Some(stdin_id) = read_stdin() {
                stdin_id.parse::<i64>().unwrap_or_else(|_| 0)
            } else {
                return Err(RepslogError::Cli("No workout-exercise-id provided. Use --help for examples.".into()));
            };
            
            // Validation: at least one metric must be provided
            if reps.is_none() && duration.is_none() && distance.is_none() && avg_heart_rate.is_none() {
                return Err(RepslogError::Cli("At least one metric (reps, duration, distance, or heart rate) must be provided.".into()));
            }

            let set_number = repo.get_next_set_number(id).await?;
            let set_id = repo.add_set(
                id, 
                set_number, 
                reps, 
                weight, 
                duration, 
                distance, 
                rpe, 
                rir, 
                effective_reps, 
                None, // cluster_id
                rest_seconds,
                notes.as_deref(),
                avg_heart_rate,
                max_heart_rate,
                hr_zones,
                pace,
                calories,
            ).await?;
            println!("Added set {} to workout-exercise {} with set ID {}", set_number, id, set_id);
        }
        SetAction::AddCardio {
            workout_exercise_id,
            distance,
            duration,
            avg_heart_rate,
            max_heart_rate,
            hr_zones,
            pace,
            calories,
            notes,
        } => {
            let id = if let Some(id) = workout_exercise_id {
                id
            } else if let Some(stdin_id) = read_stdin() {
                stdin_id.parse::<i64>().unwrap_or_else(|_| 0)
            } else {
                return Err(RepslogError::Cli("No workout-exercise-id provided. Use --help for examples.".into()));
            };

            let set_number = repo.get_next_set_number(id).await?;
            let set_id = repo.add_set(
                id,
                set_number,
                None, // reps
                None, // weight
                Some(duration),
                Some(distance),
                None, // rpe
                None, // rir
                None, // effective_reps
                None, // cluster_id
                None, // rest_seconds
                notes.as_deref(),
                Some(avg_heart_rate),
                Some(max_heart_rate),
                Some(hr_zones),
                Some(pace),
                Some(calories),
            ).await?;
            println!("Added cardio set {} to workout-exercise {} with set ID {}", set_number, id, set_id);
        }
        SetAction::AddCluster {
            workout_exercise_id,
            weight,
            reps,
            rir,
            effective_reps,
            rest_seconds,
            notes,
        } => {
            let id = if let Some(id) = workout_exercise_id {
                id
            } else if let Some(stdin_id) = read_stdin() {
                stdin_id.parse::<i64>().unwrap_or_else(|_| 0)
            } else {
                return Err(RepslogError::Cli("No workout-exercise-id provided. Use --help for examples.".into()));
            };

            let reps_list: Vec<i32> = reps.split(',').map(|s| s.trim().parse().map_err(|_| RepslogError::Cli(format!("Invalid reps: {}", s)))).collect::<Result<_>>()?;
            let rir_list: Vec<f64> = rir.split(',').map(|s| s.trim().parse().map_err(|_| RepslogError::Cli(format!("Invalid rir: {}", s)))).collect::<Result<_>>()?;
            let eff_list: Vec<i32> = effective_reps.split(',').map(|s| s.trim().parse().map_err(|_| RepslogError::Cli(format!("Invalid effective-reps: {}", s)))).collect::<Result<_>>()?;

            if reps_list.len() != rir_list.len() || reps_list.len() != eff_list.len() {
                return Err(RepslogError::Cli("The number of reps, rir, and effective-reps values must match.".into()));
            }

            let cluster_id = repo.get_next_cluster_id().await?;
            let mut set_ids = Vec::new();

            for (i, ((r, ri), eff)) in reps_list.into_iter().zip(rir_list.into_iter()).zip(eff_list.into_iter()).enumerate() {
                let set_number = repo.get_next_set_number(id).await?;
                // First set in cluster has no rest from previous? 
                // Or maybe we record rest AFTER each set. 
                // The spec says: 10 dips (rir 0) -> rest 15s -> 5 more.
                // So the second set has rest_seconds from the first.
                let rest = if i > 0 { Some(rest_seconds) } else { None };
                
                let set_id = repo.add_set(
                    id,
                    set_number,
                    Some(r),
                    weight,
                    None,
                    None,
                    None,
                    Some(ri),
                    Some(eff),
                    Some(cluster_id),
                    rest,
                    notes.as_deref(),
                    None,
                    None,
                    None,
                    None,
                    None,
                ).await?;
                set_ids.push(set_id);
            }
            println!("Added cluster {} with {} sets to workout-exercise {}. Set IDs: {:?}", cluster_id, set_ids.len(), id, set_ids);
        }
        SetAction::List { workout_exercise_id } => {
            let sets = repo.list_sets(workout_exercise_id).await?;
            let mut rows = Vec::new();
            for s in sets {
                let cluster_label = if let Some(cid) = s.cluster_id {
                    format!(" [C{}]", cid)
                } else {
                    "".to_string()
                };
                
                let cardio_info = if s.avg_heart_rate_bpm.is_some() {
                    format!("HR: {}/{} bpm | Pace: {} | Cal: {}", 
                        s.avg_heart_rate_bpm.map(|v| v.to_string()).unwrap_or_default(),
                        s.max_heart_rate_bpm.map(|v| v.to_string()).unwrap_or_default(),
                        s.avg_pace_min_per_km.map(|v| v.to_string()).unwrap_or_default(),
                        s.calories_burned.map(|v| v.to_string()).unwrap_or_default()
                    )
                } else {
                    "".to_string()
                };

                rows.push(vec![
                    s.id.to_string(),
                    format!("{}{}", s.set_number, cluster_label),
                    s.reps.map(|r| r.to_string()).unwrap_or_default(),
                    s.weight_kg.map(|w| format!("{:.2} kg", w)).unwrap_or_default(),
                    s.distance_km.map(|d| format!("{:.2} km", d)).unwrap_or_default(),
                    s.duration_seconds.map(|d| format!("{}s", d)).unwrap_or_default(),
                    cardio_info,
                    s.notes.unwrap_or_default(),
                ]);
            }
            print_table(vec!["ID", "Set #", "Reps", "Weight", "Dist", "Dur", "Cardio", "Notes"], rows);
        }
        SetAction::Quick { workout_id, exercise_name_or_id } => {
            let exercise = repo.find_exercise_by_id_or_name(&exercise_name_or_id).await?;
            if let Some(ex) = exercise {
                let order = repo.get_max_order_for_workout(workout_id).await? + 1;
                let we_id = repo.add_workout_exercise(workout_id, ex.id, order, None).await?;
                let set_id = repo.add_set(we_id, 1, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None).await?;
                println!("Added exercise {} to workout {} and created first set with ID {}", ex.name, workout_id, set_id);
            } else {
                println!("Exercise not found: {}", exercise_name_or_id);
            }
        }
    }
    Ok(())
}
