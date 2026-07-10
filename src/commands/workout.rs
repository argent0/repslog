use crate::app_config::SanityLimits;
use crate::cli::{WorkoutAction, WorkoutExerciseAction};
use crate::error::Result;
use crate::models::{ExerciseSet, HeartRateZones};
use crate::repository::Repository;
use crate::sanity::{self, ProposedWorkoutMetrics};
use crate::track_metrics::{compute_with_zones, RouteKind, TrackMetrics, ZoneRecomputeContext};
use crate::utils::{
    format_datetime, format_datetime_opt, format_dry_run_id, format_duration, format_hr_zones_bar,
    format_pace, parse_datetime, parse_id, print_id, print_json, print_table,
};
use colored::*;

pub async fn handle_workout(
    action: WorkoutAction,
    repo: &Repository,
    limits: &SanityLimits,
    json: bool,
) -> Result<()> {
    match action {
        WorkoutAction::Create {
            workout_type,
            notes,
            date,
            dry_run,
        } => {
            let date = parse_datetime(&date)?;
            let id = repo
                .create_workout(
                    workout_type.as_deref(),
                    notes.as_deref(),
                    Some(&date),
                    dry_run,
                )
                .await?;
            let formatted_id = format_dry_run_id(id, dry_run);
            if json {
                print_id(&formatted_id, true);
            } else {
                eprintln!("Created workout with ID {}", formatted_id);
                println!("{}", formatted_id);
            }
        }
        WorkoutAction::List { limit, days } => {
            let workouts = repo.list_workouts(limit, days).await?;
            if json {
                #[derive(serde::Serialize)]
                struct WorkoutOut {
                    id: i64,
                    started_at: String,
                    workout_type: Option<String>,
                    notes: Option<String>,
                    overall_feeling: Option<i32>,
                    duration_minutes: Option<i32>,
                    created_at: Option<String>,
                }
                let outs: Vec<WorkoutOut> = workouts
                    .iter()
                    .map(|w| WorkoutOut {
                        id: w.id,
                        started_at: format_datetime(&w.started_at),
                        workout_type: w.workout_type.clone(),
                        notes: w.notes.clone(),
                        overall_feeling: w.overall_feeling,
                        duration_minutes: w.duration_minutes,
                        created_at: format_datetime_opt(&w.created_at),
                    })
                    .collect();
                print_json(&outs)?;
            } else {
                let mut rows = Vec::new();
                for w in workouts {
                    let summary = get_workout_summary(repo, &w).await?;
                    rows.push(vec![
                        w.id.to_string().cyan().to_string(),
                        format_datetime(&w.started_at).dimmed().to_string(),
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
        }
        WorkoutAction::View { workout_id } => {
            let workout = repo.get_workout(workout_id).await?;
            if let Some(w) = workout {
                if json {
                    let exercises = repo.list_workout_exercises(workout_id).await?;
                    // Collect cardio for summary (dupe of below logic for json path)
                    let mut cardio_sets: Vec<(String, ExerciseSet)> = Vec::new();
                    for (we, name, _equipment) in &exercises {
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

                    let activity_date = activity_date_prefix(&w.started_at);
                    let mut primary_track: Option<TrackMetrics> = None;

                    let mut ex_list = Vec::new();
                    for (we, name, _equipment) in &exercises {
                        let sets = repo.list_sets(we.id).await?;
                        let mut sets_json = Vec::new();
                        for s in &sets {
                            let mut v = serde_json::to_value(s).expect("set serializes");
                            if let Some(obj) = v.as_object_mut() {
                                obj.insert(
                                    "created_at".to_string(),
                                    serde_json::json!(format_datetime_opt(&s.created_at)),
                                );
                                if let Some(tm) =
                                    track_metrics_for_set(repo, s, &activity_date).await?
                                {
                                    if primary_track.is_none() {
                                        primary_track = Some(tm.clone());
                                    }
                                    obj.insert(
                                        "track_metrics".to_string(),
                                        serde_json::to_value(&tm).expect("track metrics serialize"),
                                    );
                                }
                            }
                            sets_json.push(v);
                        }
                        ex_list.push(serde_json::json!({
                            "id": we.id,
                            "exercise_name": name,
                            "order": we.order,
                            "notes": we.notes,
                            "sets": sets_json
                        }));
                    }

                    let mut data = serde_json::json!({
                        "id": w.id,
                        "started_at": format_datetime(&w.started_at),
                        "workout_type": w.workout_type,
                        "notes": w.notes,
                        "overall_feeling": w.overall_feeling,
                        "duration_minutes": w.duration_minutes,
                        "created_at": format_datetime_opt(&w.created_at),
                        "exercises": ex_list,
                    });

                    if !cardio_sets.is_empty() {
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

                        let mut laps_all = Vec::new();
                        for (_, s) in &cardio_sets {
                            if let Some(ref laps_json) = s.laps {
                                laps_all.extend(laps_json.0.clone());
                            }
                        }

                        let mut cadence_samples = Vec::new();
                        let mut ascent = 0.0f64;
                        let mut descent = 0.0f64;
                        for (_, s) in &cardio_sets {
                            if let Some(c) = s.avg_cadence_spm {
                                cadence_samples.push(c);
                            }
                            ascent += s.total_ascent_m.unwrap_or(0.0);
                            descent += s.total_descent_m.unwrap_or(0.0);
                        }
                        let avg_cadence = if cadence_samples.is_empty() {
                            None
                        } else {
                            Some(cadence_samples.iter().sum::<f64>() / cadence_samples.len() as f64)
                        };

                        data["cardio_summary"] = serde_json::json!({
                            "total_distance_km": total_dist,
                            "total_duration_seconds": total_dur,
                            "avg_pace_min_per_km": if avg_pace > 0.0 { Some(avg_pace) } else { None },
                            "avg_heart_rate_bpm": if avg_hr > 0.0 { Some(avg_hr.round()) } else { None },
                            "max_heart_rate_bpm": if max_hr > 0.0 { Some(max_hr.round()) } else { None },
                            "total_calories": total_cals,
                            "avg_cadence_spm": avg_cadence,
                            "total_ascent_m": if ascent > 0.0 { Some(ascent) } else { None },
                            "total_descent_m": if descent > 0.0 { Some(descent) } else { None },
                            "hr_zones": if aggregated_zones.z1_seconds + aggregated_zones.z2_seconds + aggregated_zones.z3_seconds + aggregated_zones.z4_seconds + aggregated_zones.z5_seconds > 0 {
                                Some(&aggregated_zones)
                            } else { None },
                            "laps": if laps_all.is_empty() { None } else { Some(laps_all) },
                            "track": primary_track,
                        });
                    }

                    print_json(&data)?;
                } else {
                    println!("{}", format!("Workout ID: {}", w.id).bold().cyan());
                    println!(
                        "Type: {}",
                        w.workout_type.as_deref().unwrap_or("General").green()
                    );
                    println!("Started: {}", format_datetime(&w.started_at).dimmed());
                    if let Some(ref notes) = w.notes {
                        if !notes.is_empty() {
                            println!("Notes: {}", notes);
                        }
                    }

                    let exercises = repo.list_workout_exercises(workout_id).await?;

                    // Collect all cardio data for a high-level summary
                    let mut cardio_sets = Vec::new();
                    for (we, name, _equipment) in &exercises {
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

                        let mut cadence_samples = Vec::new();
                        let mut ascent = 0.0;
                        let mut descent = 0.0;
                        for (_, s) in &cardio_sets {
                            if let Some(c) = s.avg_cadence_spm {
                                cadence_samples.push(c);
                            }
                            ascent += s.total_ascent_m.unwrap_or(0.0);
                            descent += s.total_descent_m.unwrap_or(0.0);
                        }
                        let cadence_display = if cadence_samples.is_empty() {
                            "--".to_string()
                        } else {
                            let avg_c =
                                cadence_samples.iter().sum::<f64>() / cadence_samples.len() as f64;
                            format!("{:.0} spm", avg_c)
                        };
                        let elev_display = if ascent > 0.0 || descent > 0.0 {
                            format!("↑{:.0}m ↓{:.0}m", ascent, descent)
                        } else {
                            "--".to_string()
                        };

                        let mut summary_table = Vec::new();
                        summary_table.push(vec![
                            format!("{:.2} km", total_dist).bold().to_string(),
                            format_duration(total_dur),
                            format_pace(avg_pace).bold().green().to_string(),
                            hr_display,
                            format!("{} kcal", total_cals).yellow().to_string(),
                            cadence_display,
                            elev_display,
                        ]);
                        print_table(
                            vec![
                                "Total Dist",
                                "Total Time",
                                "Avg Pace",
                                "Avg/Max HR",
                                "Calories",
                                "Cadence",
                                "Elev",
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
                            let show_lap_hr =
                                all_laps.iter().any(|l| l.avg_heart_rate_bpm.is_some());
                            for lap in all_laps {
                                let mut row = vec![
                                    lap.lap_number.to_string(),
                                    format!("{:.2} km", lap.distance_km),
                                    format_duration(lap.duration_seconds),
                                    format_pace(lap.pace_min_per_km).green().to_string(),
                                ];
                                if show_lap_hr {
                                    row.push(
                                        lap.avg_heart_rate_bpm
                                            .map(|h| format!("{:.0}", h))
                                            .unwrap_or_else(|| "--".into()),
                                    );
                                }
                                lap_rows.push(row);
                            }
                            if show_lap_hr {
                                print_table(
                                    vec!["Lap", "Distance", "Time", "Pace", "Avg HR"],
                                    lap_rows,
                                );
                            } else {
                                print_table(vec!["Lap", "Distance", "Time", "Pace"], lap_rows);
                            }
                        }

                        // Trackpoint-derived metrics (first cardio set with a record stream)
                        let activity_date = activity_date_prefix(&w.started_at);
                        for (_, s) in &cardio_sets {
                            if let Some(tm) = track_metrics_for_set(repo, s, &activity_date).await?
                            {
                                let has_device_laps =
                                    s.laps.as_ref().map(|j| !j.0.is_empty()).unwrap_or(false);
                                let stored_zones_empty = s
                                    .heart_rate_zones
                                    .as_ref()
                                    .map(|z| {
                                        z.0.z1_seconds
                                            + z.0.z2_seconds
                                            + z.0.z3_seconds
                                            + z.0.z4_seconds
                                            + z.0.z5_seconds
                                            == 0
                                    })
                                    .unwrap_or(true);
                                print_track_metrics(
                                    &tm,
                                    s.distance_km,
                                    stored_zones_empty,
                                    !has_device_laps,
                                );
                                break;
                            }
                        }
                    }

                    println!("\n{}", "EXERCISES".bold().yellow());
                    for (we, name, load_type) in exercises {
                        println!("{} (WE ID: {})", name.bold(), we.id.to_string().dimmed());
                        if let Some(ref notes) = we.notes {
                            println!("Notes: {}", notes);
                        }
                        let sets = repo.list_sets(we.id).await?;
                        let mut set_rows = Vec::new();
                        let mut left_reps = 0i32;
                        let mut right_reps = 0i32;
                        let mut both_or_unspec_reps = 0i32;
                        let mut has_side = false;

                        for s in &sets {
                            if let Some(ref sd) = s.side {
                                has_side = true;
                                match sd.as_str() {
                                    "left" => left_reps += s.reps.unwrap_or(0),
                                    "right" => right_reps += s.reps.unwrap_or(0),
                                    _ => both_or_unspec_reps += s.reps.unwrap_or(0),
                                }
                            } else {
                                both_or_unspec_reps += s.reps.unwrap_or(0);
                            }

                            let cluster_label = if let Some(cid) = s.cluster_id {
                                format!(" [C{}]", cid)
                            } else {
                                "".to_string()
                            };

                            let mut details = Vec::new();
                            if let Some(reps) = s.reps {
                                details.push(crate::phase::format_reps_with_phase(reps, &s.phase));
                            }
                            let load = crate::bodyweight::format_load_display(
                                &load_type,
                                s.weight_kg,
                                s.external_load_kg,
                            );
                            if !load.is_empty() {
                                details.push(load);
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

                            let side_label = s
                                .side
                                .as_ref()
                                .map(|sd| sd.to_uppercase())
                                .unwrap_or_else(|| "-".to_string());
                            let phase_label = {
                                let label = crate::phase::format_phase_label(&s.phase);
                                if label.is_empty() {
                                    "full".to_string()
                                } else {
                                    label
                                }
                            };
                            set_rows.push(vec![
                                s.set_number.to_string() + &cluster_label,
                                side_label,
                                phase_label,
                                details.join(" • "),
                                cardio_info.dimmed().to_string(),
                                s.notes.as_ref().cloned().unwrap_or_default(),
                            ]);
                        }

                        if we.notes.is_some() || has_side || we.goal_reps.is_some() {
                            // already printed notes above; add side totals if relevant
                            if has_side || we.goal_reps.is_some() {
                                let mut summary_parts = Vec::new();
                                if left_reps > 0 || right_reps > 0 {
                                    summary_parts.push(format!(
                                        "Left: {} reps | Right: {} reps",
                                        left_reps, right_reps
                                    ));
                                }
                                if both_or_unspec_reps > 0 && (left_reps > 0 || right_reps > 0) {
                                    summary_parts
                                        .push(format!("Other: {} reps", both_or_unspec_reps));
                                }
                                if let Some(g) = we.goal_reps {
                                    let actual = left_reps + right_reps + both_or_unspec_reps;
                                    summary_parts.push(format!("Goal: {} | Actual: {}", g, actual));
                                }
                                if !summary_parts.is_empty() {
                                    println!("  {}", summary_parts.join("  •  ").dimmed());
                                }
                            }
                        }

                        print_table(
                            vec!["Set #", "Side", "Phase", "Details", "Cardio", "Notes"],
                            set_rows,
                        );
                    }
                }
            } else {
                if json {
                    println!("null");
                } else {
                    println!("Workout not found");
                }
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
            let date = date.as_deref().map(parse_datetime).transpose()?;
            sanity::validate_workout_metrics(
                &ProposedWorkoutMetrics {
                    duration_minutes: duration,
                    overall_feeling: feeling,
                },
                limits,
            )?;
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
            if json {
                println!(r#"{{"success": true, "id": "{}"}}"#, workout_id);
            } else {
                println!("Updated workout {}", workout_id);
            }
        }
        WorkoutAction::Delete {
            workout_id,
            dry_run,
        } => {
            let id = parse_id(&workout_id, dry_run)?;
            repo.delete_workout(id, dry_run).await?;
            if json {
                println!(r#"{{"success": true, "id": "{}"}}"#, workout_id);
            } else {
                println!("Deleted workout {}", workout_id);
            }
        }
    }
    Ok(())
}

fn activity_date_prefix(started_at: &str) -> String {
    started_at.get(..10).unwrap_or(started_at).to_string()
}

async fn track_metrics_for_set(
    repo: &Repository,
    set: &ExerciseSet,
    activity_date: &str,
) -> Result<Option<TrackMetrics>> {
    let points = repo.list_trackpoints(set.id).await?;
    if points.is_empty() {
        return Ok(None);
    }
    let ctx = ZoneRecomputeContext {
        date_of_birth: set.date_of_birth.clone(),
        resting_hr_bpm: set.resting_hr_bpm,
        activity_date: Some(activity_date.to_string()),
    };
    Ok(compute_with_zones(&points, set.distance_km, &ctx))
}

fn print_track_metrics(
    m: &TrackMetrics,
    device_distance_km: Option<f64>,
    stored_zones_empty: bool,
    show_synthetic_splits: bool,
) {
    println!("\n{}", "TRACK METRICS".bold().yellow());
    println!("  Samples  {}", m.sample_count);

    let moving_pace = m
        .moving_pace_min_per_km
        .map(|p| format_pace(p).green().to_string())
        .unwrap_or_else(|| "--".into());
    println!(
        "  Moving   {}  (stopped {})    Moving pace  {}",
        format_duration(m.moving_seconds),
        format_duration(m.stopped_seconds),
        moving_pace
    );

    if let Some(ref pace) = m.pace {
        let cv = m
            .pace_cv
            .map(|c| format!("  · CV {:.0}%", c * 100.0))
            .unwrap_or_default();
        println!(
            "  Pace     med {}  · {}–{}{}",
            format_pace(pace.median).green(),
            format_pace(pace.min),
            format_pace(pace.max),
            cv
        );
    }

    if !m.best_efforts.is_empty() {
        let parts: Vec<String> = m
            .best_efforts
            .iter()
            .take(4)
            .map(|b| {
                if let Some(dur) = b.duration_seconds {
                    if b.label.contains("min") {
                        format!(
                            "{} {}",
                            b.label,
                            b.distance_km
                                .map(|d| format!("{:.2} km", d))
                                .unwrap_or_else(|| "--".into())
                        )
                    } else {
                        format!("{} {}", b.label, format_duration(dur))
                    }
                } else {
                    b.label.clone()
                }
            })
            .collect();
        println!("  Best     {}", parts.join("  ·  "));
    }

    if let Some(ref cad) = m.cadence {
        let cv = m
            .cadence_cv
            .map(|c| format!("  · CV {:.0}%", c * 100.0))
            .unwrap_or_default();
        let stride = m
            .avg_stride_m
            .map(|s| format!("  · stride ~{:.2} m", s))
            .unwrap_or_default();
        println!(
            "  Cadence  med {:.0}  · {:.0}–{:.0}{}{}  {}",
            cad.median,
            cad.min,
            cad.max,
            cv,
            stride,
            "(device units)".dimmed()
        );
    }

    if m.elev_min_m.is_some() || m.elev_max_m.is_some() {
        let mut parts = Vec::new();
        if let (Some(lo), Some(hi)) = (m.elev_min_m, m.elev_max_m) {
            parts.push(format!("{:.0}–{:.0} m", lo, hi));
        }
        if let Some(net) = m.elev_net_m {
            parts.push(format!("net {:+.0} m", net));
        }
        if let (Some(a), Some(d)) = (m.ascent_m, m.descent_m) {
            parts.push(format!("↑{:.0} ↓{:.0} (smoothed)", a, d));
        }
        if let Some(gap) = m.grade_adj_pace_min_per_km {
            parts.push(format!("GAP {}", format_pace(gap)));
        }
        if let Some(vam) = m.vam_m_per_hour {
            parts.push(format!("VAM {:.0} m/h", vam));
        }
        if !parts.is_empty() {
            println!("  Elev     {}", parts.join("  ·  "));
        }
    }

    {
        let mut hr_parts = Vec::new();
        if let Some(min) = m.hr_min {
            hr_parts.push(format!("min {:.0}", min));
        }
        if let Some(drift) = m.hr_drift_pct {
            hr_parts.push(format!("drift {:+.1}%", drift));
        }
        if !hr_parts.is_empty() {
            println!("  HR       {}", hr_parts.join("  ·  ").red());
        }
    }

    if stored_zones_empty {
        if let Some(ref z) = m.hr_zones_recomputed {
            let total = z.z1_seconds + z.z2_seconds + z.z3_seconds + z.z4_seconds + z.z5_seconds;
            if total > 0 {
                println!("  Track zones: {}", format_hr_zones_bar(z));
            }
        }
    }

    if let Some(ref route) = m.route {
        let kind = match route.kind {
            RouteKind::Loop => "loop",
            RouteKind::PointToPoint => "point-to-point",
            RouteKind::Unknown => "unknown",
        };
        let mut parts = vec![kind.to_string()];
        if let Some(gps) = route.gps_distance_km {
            if let Some(dev) = device_distance_km {
                parts.push(format!("GPS {:.2} km (device {:.2})", gps, dev));
            } else {
                parts.push(format!("GPS {:.2} km", gps));
            }
        }
        if let Some(gap) = route.start_end_gap_m {
            parts.push(format!("start–end {:.0} m", gap));
        }
        println!("  Route    {}", parts.join("  ·  "));
    }

    if show_synthetic_splits {
        let full: Vec<_> = m
            .synthetic_km_splits
            .iter()
            .filter(|s| !s.partial || s.distance_km >= 0.2)
            .collect();
        if full.iter().any(|s| !s.partial) {
            println!("\n{}", "COMPUTED KM SPLITS".bold().yellow());
            let show_hr = full.iter().any(|s| s.avg_hr.is_some());
            let mut rows = Vec::new();
            for s in full {
                let label = if s.partial {
                    format!("{:.2}*", s.distance_km)
                } else {
                    s.km_index.to_string()
                };
                let mut row = vec![
                    label,
                    format!("{:.2} km", s.distance_km),
                    format_duration(s.duration_seconds),
                    format_pace(s.pace_min_per_km).green().to_string(),
                ];
                if show_hr {
                    row.push(
                        s.avg_hr
                            .map(|h| format!("{:.0}", h))
                            .unwrap_or_else(|| "--".into()),
                    );
                }
                rows.push(row);
            }
            if show_hr {
                print_table(vec!["Km", "Distance", "Time", "Pace", "Avg HR"], rows);
            } else {
                print_table(vec!["Km", "Distance", "Time", "Pace"], rows);
            }
        }
    }
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

    for (we, _, _) in &exercises {
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
    json: bool,
) -> Result<()> {
    match action {
        WorkoutExerciseAction::Add {
            workout_id,
            exercise_id_or_name,
            order,
            goal_reps,
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
                    .add_workout_exercise(w_id, ex.id, order, None, goal_reps, dry_run)
                    .await?;
                let formatted_id = format_dry_run_id(id, dry_run);
                if json {
                    print_id(&formatted_id, true);
                } else {
                    eprintln!(
                        "Added exercise {} (ID: {}) to workout {} with WE ID {}",
                        ex.name, ex.id, workout_id, formatted_id
                    );
                    println!("{}", formatted_id);
                }
            } else {
                println!("Exercise not found: {}", exercise_id_or_name);
            }
        }
        WorkoutExerciseAction::List { workout_id } => {
            let exercises = repo.list_workout_exercises(workout_id).await?;
            if json {
                #[derive(serde::Serialize)]
                struct WeOut {
                    id: i64,
                    workout_id: i64,
                    exercise_id: i64,
                    order: i32,
                    notes: Option<String>,
                    goal_reps: Option<i32>,
                    exercise_name: String,
                }
                let outs: Vec<WeOut> = exercises
                    .into_iter()
                    .map(|(we, name, _equipment)| WeOut {
                        id: we.id,
                        workout_id: we.workout_id,
                        exercise_id: we.exercise_id,
                        order: we.order,
                        notes: we.notes,
                        goal_reps: we.goal_reps,
                        exercise_name: name,
                    })
                    .collect();
                print_json(&outs)?;
            } else {
                let mut rows = Vec::new();
                for (we, name, _equipment) in exercises {
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
    }
    Ok(())
}
