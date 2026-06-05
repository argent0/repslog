use crate::cli::{WorkoutAction, WorkoutExerciseAction};
use crate::error::Result;
use crate::models::HeartRateZones;
use crate::repository::Repository;
use crate::utils::{
    format_dry_run_id, format_duration, format_hr_zones_bar, format_pace, parse_id, print_table,
};
use colored::*;

pub async fn handle_workout(action: WorkoutAction, repo: &Repository) -> Result<()> {
    match action {
        WorkoutAction::Create {
            workout_type,
            notes,
            date,
            dry_run,
        } => {
            // Try to parse to ensure it's a valid date (YYYY-MM-DD)
            if chrono::NaiveDate::parse_from_str(&date, "%Y-%m-%d").is_err()
                && chrono::NaiveDateTime::parse_from_str(&date, "%Y-%m-%d %H:%M:%S").is_err()
            {
                return Err(crate::error::RepslogError::Cli(
                    "Invalid date format. Use YYYY-MM-DD or 'YYYY-MM-DD HH:MM:SS'".to_string(),
                ));
            }
            let id = repo
                .create_workout(
                    workout_type.as_deref(),
                    notes.as_deref(),
                    Some(&date),
                    dry_run,
                )
                .await?;
            let formatted_id = format_dry_run_id(id, dry_run);
            eprintln!("Created workout with ID {}", formatted_id);
            println!("{}", formatted_id);
        }
        WorkoutAction::List { limit, days } => {
            let workouts = repo.list_workouts(limit, days).await?;
            let mut rows = Vec::new();
            for w in workouts {
                let summary = get_workout_summary(repo, &w).await?;
                rows.push(vec![
                    w.id.to_string().cyan().to_string(),
                    w.started_at.dimmed().to_string(),
                    w.workout_type
                        .clone()
                        .unwrap_or_default()
                        .green()
                        .to_string(),
                    w.duration_minutes
                        .map(|d| d.to_string())
                        .unwrap_or_default(),
                    summary,
                ]);
            }
            print_table(vec!["ID", "Started At", "Type", "Dur", "Summary"], rows);
        }
        WorkoutAction::View { workout_id } => {
            let workout = repo.get_workout(workout_id).await?;
            if let Some(w) = workout {
                println!("{}", format!("Workout ID: {}", w.id).bold().cyan());
                println!(
                    "Type: {}",
                    w.workout_type.as_deref().unwrap_or("General").green()
                );
                println!("Started: {}", w.started_at.dimmed());
                if let Some(ref notes) = w.notes {
                    if !notes.is_empty() {
                        println!("Notes: {}", notes);
                    }
                }

                let exercises = repo.list_workout_exercises(workout_id).await?;

                // Collect all cardio data for a high-level summary
                let mut cardio_sets = Vec::new();
                for (we, name) in &exercises {
                    let sets = repo.list_sets(we.id).await?;
                    for s in sets {
                        if s.distance_km.is_some()
                            || s.duration_seconds.is_some()
                            || s.avg_heart_rate_bpm.is_some()
                        {
                            cardio_sets.push((name.clone(), s));
                        }
                    }
                }

                if !cardio_sets.is_empty() {
                    println!("\n{}", "CARDIO SUMMARY".bold().yellow());
                    let mut total_dist = 0.0;
                    let mut total_dur = 0;
                    let mut total_cals = 0;
                    let mut hr_samples = Vec::new();
                    let mut max_hr = 0.0;
                    let mut aggregated_zones = HeartRateZones::default();

                    for (_, s) in &cardio_sets {
                        total_dist += s.distance_km.unwrap_or(0.0);
                        total_dur += s.duration_seconds.unwrap_or(0) as u32;
                        total_cals += s.calories_burned.unwrap_or(0);
                        if let Some(hr) = s.avg_heart_rate_bpm {
                            hr_samples.push(hr);
                        }
                        if let Some(hr) = s.max_heart_rate_bpm {
                            if hr > max_hr {
                                max_hr = hr;
                            }
                        }
                        if let Some(ref zones) = s.heart_rate_zones {
                            aggregated_zones.z1_seconds += zones.0.z1_seconds;
                            aggregated_zones.z2_seconds += zones.0.z2_seconds;
                            aggregated_zones.z3_seconds += zones.0.z3_seconds;
                            aggregated_zones.z4_seconds += zones.0.z4_seconds;
                            aggregated_zones.z5_seconds += zones.0.z5_seconds;
                        }
                    }

                    let avg_hr = if hr_samples.is_empty() {
                        0.0
                    } else {
                        hr_samples.iter().sum::<f64>() / hr_samples.len() as f64
                    };
                    let avg_pace = if total_dist > 0.0 {
                        (total_dur as f64 / 60.0) / total_dist
                    } else {
                        0.0
                    };

                    let hr_display = if hr_samples.is_empty() && max_hr == 0.0 {
                        "--".to_string()
                    } else {
                        format!("{} / {} bpm", avg_hr.round(), max_hr.round())
                            .red()
                            .to_string()
                    };

                    let mut summary_table = Vec::new();
                    summary_table.push(vec![
                        format!("{:.2} km", total_dist).bold().to_string(),
                        format_duration(total_dur),
                        format_pace(avg_pace).bold().green().to_string(),
                        hr_display,
                        format!("{} kcal", total_cals).yellow().to_string(),
                    ]);
                    print_table(
                        vec![
                            "Total Dist",
                            "Total Time",
                            "Avg Pace",
                            "Avg/Max HR",
                            "Calories",
                        ],
                        summary_table,
                    );

                    if total_dur > 0
                        && (aggregated_zones.z1_seconds > 0
                            || aggregated_zones.z2_seconds > 0
                            || aggregated_zones.z3_seconds > 0
                            || aggregated_zones.z4_seconds > 0
                            || aggregated_zones.z5_seconds > 0)
                    {
                        println!("HR Zones: {}", format_hr_zones_bar(&aggregated_zones));
                    }

                    // Laps Table if available
                    let mut all_laps = Vec::new();
                    for (_, s) in &cardio_sets {
                        if let Some(ref laps_json) = s.laps {
                            all_laps.extend(laps_json.0.clone());
                        }
                    }

                    if !all_laps.is_empty() {
                        println!("\n{}", "LAPS / SPLITS".bold().yellow());
                        let mut lap_rows = Vec::new();
                        for lap in all_laps {
                            lap_rows.push(vec![
                                lap.lap_number.to_string(),
                                format!("{:.2} km", lap.distance_km),
                                format_duration(lap.duration_seconds),
                                format_pace(lap.pace_min_per_km).green().to_string(),
                            ]);
                        }
                        print_table(vec!["Lap", "Distance", "Time", "Pace"], lap_rows);
                    }
                }

                println!("\n{}", "EXERCISES".bold().yellow());
                for (we, name) in exercises {
                    println!("\n{} (WE ID: {})", name.bold(), we.id.to_string().dimmed());
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

                        let mut details = Vec::new();
                        if let Some(reps) = s.reps {
                            details.push(format!("{} reps", reps));
                        }
                        if let Some(weight) = s.weight_kg {
                            details.push(format!("{:.2} kg", weight));
                        }
                        if let Some(dist) = s.distance_km {
                            details.push(format!("{:.2} km", dist));
                        }
                        if let Some(dur) = s.duration_seconds {
                            details.push(format_duration(dur as u32));
                        }
                        if let Some(rpe) = s.rpe {
                            details.push(format!("RPE {}", rpe));
                        }
                        if let Some(rir) = s.rir {
                            details.push(format!("RIR {}", rir));
                        }

                        let cardio_info = if s.avg_heart_rate_bpm.is_some() {
                            format!(
                                "{} bpm | {} | {} cal",
                                s.avg_heart_rate_bpm
                                    .map(|v| v.round().to_string())
                                    .unwrap_or_else(|| "--".to_string()),
                                s.avg_pace_min_per_km
                                    .map(format_pace)
                                    .unwrap_or_else(|| "--".to_string()),
                                s.calories_burned
                                    .map(|v| v.to_string())
                                    .unwrap_or_else(|| "--".to_string())
                            )
                        } else {
                            "".to_string()
                        };

                        set_rows.push(vec![
                            s.set_number.to_string() + &cluster_label,
                            details.join(" • "),
                            cardio_info.dimmed().to_string(),
                            s.notes.as_ref().cloned().unwrap_or_default(),
                        ]);
                    }
                    print_table(vec!["Set #", "Details", "Cardio", "Notes"], set_rows);
                }
            } else {
                println!("Workout not found");
            }
        }
        WorkoutAction::Update {
            workout_id,
            workout_type,
            notes,
            duration,
            feeling,
            date,
            dry_run,
        } => {
            let id = parse_id(&workout_id, dry_run)?;
            if let Some(ref d) = date {
                if chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").is_err()
                    && chrono::NaiveDateTime::parse_from_str(d, "%Y-%m-%d %H:%M:%S").is_err()
                {
                    return Err(crate::error::RepslogError::Cli(
                        "Invalid date format. Use YYYY-MM-DD or 'YYYY-MM-DD HH:MM:SS'".to_string(),
                    ));
                }
            }
            repo.update_workout(
                id,
                workout_type.as_deref(),
                notes.as_deref(),
                duration,
                feeling,
                date.as_deref(),
                dry_run,
            )
            .await?;
            println!("Updated workout {}", workout_id);
        }
        WorkoutAction::Delete {
            workout_id,
            dry_run,
        } => {
            let id = parse_id(&workout_id, dry_run)?;
            repo.delete_workout(id, dry_run).await?;
            println!("Deleted workout {}", workout_id);
        }
    }
    Ok(())
}

async fn get_workout_summary(
    repo: &Repository,
    workout: &crate::models::Workout,
) -> Result<String> {
    let exercises = repo.list_workout_exercises(workout.id).await?;
    let mut total_distance = 0.0;
    let mut total_duration = 0;
    let mut hr_samples = Vec::new();
    let mut cardio_found = false;

    for (we, _) in &exercises {
        let sets = repo.list_sets(we.id).await?;
        for s in sets {
            if let Some(dist) = s.distance_km {
                total_distance += dist;
                cardio_found = true;
            }
            if let Some(dur) = s.duration_seconds {
                total_duration += dur as u32;
                cardio_found = true;
            }
            if let Some(hr) = s.avg_heart_rate_bpm {
                hr_samples.push(hr);
                cardio_found = true;
            }
        }
    }

    if cardio_found {
        let pace = if total_distance > 0.0 {
            Some((total_duration as f64 / 60.0) / total_distance)
        } else {
            None
        };
        let avg_hr = if hr_samples.is_empty() {
            None
        } else {
            Some(hr_samples.iter().sum::<f64>() / hr_samples.len() as f64)
        };

        Ok(format!(
            "{} • {:.2} km • {} • {} • {} bpm",
            workout.workout_type.as_deref().unwrap_or("Run").bold(),
            total_distance,
            format_duration(total_duration),
            pace.map(format_pace).unwrap_or_else(|| "--".to_string()),
            avg_hr
                .map(|h| h.round().to_string())
                .unwrap_or_else(|| "--".to_string())
        ))
    } else {
        Ok(workout
            .notes
            .clone()
            .unwrap_or_default()
            .dimmed()
            .to_string())
    }
}

pub async fn handle_workout_exercise(
    action: WorkoutExerciseAction,
    repo: &Repository,
) -> Result<()> {
    match action {
        WorkoutExerciseAction::Add {
            workout_id,
            exercise_id_or_name,
            order,
            dry_run,
        } => {
            let w_id = parse_id(&workout_id, dry_run)?;
            let exercise = repo
                .find_exercise_by_id_or_name(&exercise_id_or_name)
                .await?;
            if let Some(ex) = exercise {
                let order = if let Some(o) = order {
                    o
                } else {
                    repo.get_max_order_for_workout(w_id).await? + 1
                };
                let id = repo
                    .add_workout_exercise(w_id, ex.id, order, None, dry_run)
                    .await?;
                let formatted_id = format_dry_run_id(id, dry_run);
                eprintln!(
                    "Added exercise {} (ID: {}) to workout {} with WE ID {}",
                    ex.name, ex.id, workout_id, formatted_id
                );
                println!("{}", formatted_id);
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
