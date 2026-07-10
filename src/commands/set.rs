use crate::app_config::SanityLimits;
use crate::bodyweight;
use crate::cli::SetAction;
use crate::error::{RepslogError, Result};
use crate::models::Lap;
use crate::phase::{self, format_phase_label};
use crate::repository::Repository;
use crate::sanity::{self, ProposedSetMetrics};
use crate::utils::{
    format_datetime_opt, format_dry_run_id, format_duration, format_pace, parse_id, print_id,
    print_json, print_table, read_stdin,
};
use sqlx::types::Json;

pub async fn handle_set(
    action: SetAction,
    repo: &Repository,
    limits: &SanityLimits,
    json: bool,
) -> Result<()> {
    match action {
        SetAction::Add {
            workout_exercise_id,
            reps,
            weight,
            external_load,
            no_weight_recorded,
            duration,
            distance,
            rpe,
            rir,
            effective_reps,
            rest_seconds,
            notes,
            side,
            phase,
            avg_heart_rate,
            max_heart_rate,
            hr_zones,
            pace,
            calories,
            laps,
            dry_run,
        } => {
            let resolved_phase = phase::normalize_phase(&phase)?;
            let id_str = if let Some(id) = workout_exercise_id {
                id
            } else if let Some(stdin_id) = read_stdin() {
                stdin_id
            } else {
                return Err(RepslogError::Cli(
                    "No workout-exercise-id provided. Use --help for examples.".into(),
                ));
            };
            let id = parse_id(&id_str, dry_run)?;

            // Validation: at least one metric must be provided (weight is valid for strength/ progressive overload sets)
            if reps.is_none()
                && weight.is_none()
                && duration.is_none()
                && distance.is_none()
                && avg_heart_rate.is_none()
                && external_load.is_none()
            {
                return Err(RepslogError::Cli("At least one metric (reps, weight, duration, distance, external load, or heart rate) must be provided.".into()));
            }

            if let Some(ref laps_wrapper) = laps {
                validate_laps(&laps_wrapper.0, distance, duration.map(|d| d as u32))?;
            }

            let (resolved_weight, resolved_external_load) = resolve_load_for_workout_exercise(
                repo,
                id,
                &id_str,
                dry_run,
                weight,
                external_load,
                no_weight_recorded,
                reps,
                duration,
            )
            .await?;

            sanity::validate_set_metrics(
                &ProposedSetMetrics {
                    reps,
                    weight_kg: resolved_weight,
                    external_load_kg: resolved_external_load,
                    distance_km: distance,
                    duration_seconds: duration,
                    rpe,
                    rir,
                    effective_reps,
                    rest_seconds,
                    avg_heart_rate_bpm: avg_heart_rate,
                    max_heart_rate_bpm: max_heart_rate,
                    avg_pace_min_per_km: pace,
                    calories_burned: calories,
                    heart_rate_zones: hr_zones.clone(),
                    laps: laps.as_ref().map(|l| l.0.clone()),
                    ..Default::default()
                },
                limits,
            )?;

            let set_number = if dry_run && id_str.starts_with("DRY-RUN-") {
                1 // If it's a dry-run workout exercise, start with set 1
            } else {
                repo.get_next_set_number(id).await?
            };

            let set_id = repo
                .add_set(
                    id,
                    set_number,
                    reps,
                    resolved_weight,
                    resolved_external_load,
                    duration,
                    distance,
                    rpe,
                    rir,
                    effective_reps,
                    None, // cluster_id
                    rest_seconds,
                    notes.as_deref(),
                    side.as_deref().map(|s| s.to_lowercase()).as_deref(),
                    resolved_phase,
                    avg_heart_rate,
                    max_heart_rate,
                    hr_zones.map(Json),
                    pace,
                    calories,
                    laps.map(|l| Json(l.0)),
                    None,
                    None,
                    None,
                    None, // date_of_birth
                    None, // resting_hr_bpm
                    dry_run,
                )
                .await?;
            let formatted_set_id = format_dry_run_id(set_id, dry_run);
            if json {
                print_id(&formatted_set_id, true);
            } else {
                eprintln!(
                    "Added set {} to workout-exercise {} with set ID {}",
                    set_number, id_str, formatted_set_id
                );
                println!("{}", formatted_set_id);
            }
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
            laps,
            notes,
            side,
            phase,
            dry_run,
        } => {
            let resolved_phase = phase::normalize_phase(&phase)?;
            let id_str = if let Some(id) = workout_exercise_id {
                id
            } else if let Some(stdin_id) = read_stdin() {
                stdin_id
            } else {
                return Err(RepslogError::Cli(
                    "No workout-exercise-id provided. Use --help for examples.".into(),
                ));
            };
            let id = parse_id(&id_str, dry_run)?;

            if let Some(ref laps_wrapper) = laps {
                validate_laps(&laps_wrapper.0, Some(distance), Some(duration as u32))?;
            }

            sanity::validate_set_metrics(
                &ProposedSetMetrics {
                    distance_km: Some(distance),
                    duration_seconds: Some(duration),
                    avg_heart_rate_bpm: Some(avg_heart_rate),
                    max_heart_rate_bpm: Some(max_heart_rate),
                    avg_pace_min_per_km: Some(pace),
                    calories_burned: Some(calories),
                    heart_rate_zones: Some(hr_zones.clone()),
                    laps: laps.as_ref().map(|l| l.0.clone()),
                    ..Default::default()
                },
                limits,
            )?;

            let set_number = if dry_run && id_str.starts_with("DRY-RUN-") {
                1
            } else {
                repo.get_next_set_number(id).await?
            };

            let set_id = repo
                .add_set(
                    id,
                    set_number,
                    None, // reps
                    None, // weight
                    None, // external_load
                    Some(duration),
                    Some(distance),
                    None, // rpe
                    None, // rir
                    None, // effective_reps
                    None, // cluster_id
                    None, // rest_seconds
                    notes.as_deref(),
                    side.as_deref().map(|s| s.to_lowercase()).as_deref(),
                    resolved_phase,
                    Some(avg_heart_rate),
                    Some(max_heart_rate),
                    Some(Json(hr_zones)),
                    Some(pace),
                    Some(calories),
                    laps.map(|l| Json(l.0)),
                    None,
                    None,
                    None,
                    None, // date_of_birth
                    None, // resting_hr_bpm
                    dry_run,
                )
                .await?;
            let formatted_set_id = format_dry_run_id(set_id, dry_run);
            if json {
                print_id(&formatted_set_id, true);
            } else {
                eprintln!(
                    "Added cardio set {} to workout-exercise {} with set ID {}",
                    set_number, id_str, formatted_set_id
                );
                println!("{}", formatted_set_id);
            }
        }
        SetAction::AddCluster {
            workout_exercise_id,
            weight,
            external_load,
            no_weight_recorded,
            reps,
            rir,
            effective_reps,
            rest_seconds,
            notes,
            side,
            phase,
            dry_run,
        } => {
            let resolved_phase = phase::normalize_phase(&phase)?;
            let id_str = if let Some(id) = workout_exercise_id {
                id
            } else if let Some(stdin_id) = read_stdin() {
                stdin_id
            } else {
                return Err(RepslogError::Cli(
                    "No workout-exercise-id provided. Use --help for examples.".into(),
                ));
            };
            let id = parse_id(&id_str, dry_run)?;

            let reps_list: Vec<i32> = reps
                .split(',')
                .map(|s| {
                    s.trim()
                        .parse()
                        .map_err(|_| RepslogError::Cli(format!("Invalid reps: {}", s)))
                })
                .collect::<Result<_>>()?;
            let rir_list: Vec<f64> = rir
                .split(',')
                .map(|s| {
                    s.trim()
                        .parse()
                        .map_err(|_| RepslogError::Cli(format!("Invalid rir: {}", s)))
                })
                .collect::<Result<_>>()?;
            let eff_list: Vec<i32> = effective_reps
                .split(',')
                .map(|s| {
                    s.trim()
                        .parse()
                        .map_err(|_| RepslogError::Cli(format!("Invalid effective-reps: {}", s)))
                })
                .collect::<Result<_>>()?;

            if reps_list.len() != rir_list.len() || reps_list.len() != eff_list.len() {
                return Err(RepslogError::Cli(
                    "The number of reps, rir, and effective-reps values must match.".into(),
                ));
            }

            let (resolved_weight, resolved_external_load) = resolve_load_for_workout_exercise(
                repo,
                id,
                &id_str,
                dry_run,
                weight,
                external_load,
                no_weight_recorded,
                Some(1),
                None,
            )
            .await?;

            let cluster_id = repo.get_next_cluster_id().await?;
            let mut set_ids = Vec::new();

            for (i, ((r, ri), eff)) in reps_list
                .into_iter()
                .zip(rir_list)
                .zip(eff_list)
                .enumerate()
            {
                let rest = if i > 0 { Some(rest_seconds) } else { None };
                sanity::validate_set_metrics(
                    &ProposedSetMetrics {
                        reps: Some(r),
                        weight_kg: resolved_weight,
                        external_load_kg: resolved_external_load,
                        rir: Some(ri),
                        effective_reps: Some(eff),
                        rest_seconds: rest,
                        ..Default::default()
                    },
                    limits,
                )?;

                let set_number = if dry_run && id_str.starts_with("DRY-RUN-") {
                    (i + 1) as i32
                } else {
                    repo.get_next_set_number(id).await?
                };

                let set_id = repo
                    .add_set(
                        id,
                        set_number,
                        Some(r),
                        resolved_weight,
                        resolved_external_load,
                        None,
                        None,
                        None,
                        Some(ri),
                        Some(eff),
                        Some(cluster_id),
                        rest,
                        notes.as_deref(),
                        side.as_deref().map(|s| s.to_lowercase()).as_deref(),
                        resolved_phase,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None, // date_of_birth
                        None, // resting_hr_bpm
                        dry_run,
                    )
                    .await?;
                set_ids.push(format_dry_run_id(set_id, dry_run));
            }
            let formatted_cluster_id = format_dry_run_id(cluster_id, dry_run);
            if json {
                print_id(&formatted_cluster_id, true);
            } else {
                eprintln!(
                    "Added cluster {} with {} sets to workout-exercise {}. Set IDs: {:?}",
                    formatted_cluster_id,
                    set_ids.len(),
                    id_str,
                    set_ids
                );
                println!("{}", formatted_cluster_id);
            }
        }
        SetAction::List {
            workout_exercise_id,
        } => {
            let sets = repo.list_sets(workout_exercise_id).await?;
            if json {
                #[derive(serde::Serialize)]
                struct SetOut {
                    id: i64,
                    workout_exercise_id: i64,
                    set_number: i32,
                    reps: Option<i32>,
                    weight_kg: Option<f64>,
                    external_load_kg: Option<f64>,
                    distance_km: Option<f64>,
                    duration_seconds: Option<i32>,
                    rpe: Option<f64>,
                    rir: Option<f64>,
                    effective_reps: Option<i32>,
                    cluster_id: Option<i64>,
                    rest_seconds: Option<i32>,
                    notes: Option<String>,
                    side: Option<String>,
                    phase: String,
                    extra_metrics: Option<String>,
                    avg_heart_rate_bpm: Option<f64>,
                    max_heart_rate_bpm: Option<f64>,
                    heart_rate_zones: Option<sqlx::types::Json<crate::models::HeartRateZones>>,
                    avg_pace_min_per_km: Option<f64>,
                    calories_burned: Option<i32>,
                    laps: Option<sqlx::types::Json<Vec<crate::models::Lap>>>,
                    avg_cadence_spm: Option<f64>,
                    total_ascent_m: Option<f64>,
                    total_descent_m: Option<f64>,
                    created_at: Option<String>,
                }
                let outs: Vec<SetOut> = sets
                    .iter()
                    .map(|s| SetOut {
                        id: s.id,
                        workout_exercise_id: s.workout_exercise_id,
                        set_number: s.set_number,
                        reps: s.reps,
                        weight_kg: s.weight_kg,
                        external_load_kg: s.external_load_kg,
                        distance_km: s.distance_km,
                        duration_seconds: s.duration_seconds,
                        rpe: s.rpe,
                        rir: s.rir,
                        effective_reps: s.effective_reps,
                        cluster_id: s.cluster_id,
                        rest_seconds: s.rest_seconds,
                        notes: s.notes.clone(),
                        side: s.side.clone(),
                        phase: s.phase.clone(),
                        extra_metrics: s.extra_metrics.clone(),
                        avg_heart_rate_bpm: s.avg_heart_rate_bpm,
                        max_heart_rate_bpm: s.max_heart_rate_bpm,
                        heart_rate_zones: s.heart_rate_zones.clone(),
                        avg_pace_min_per_km: s.avg_pace_min_per_km,
                        calories_burned: s.calories_burned,
                        laps: s.laps.clone(),
                        avg_cadence_spm: s.avg_cadence_spm,
                        total_ascent_m: s.total_ascent_m,
                        total_descent_m: s.total_descent_m,
                        created_at: format_datetime_opt(&s.created_at),
                    })
                    .collect();
                print_json(&outs)?;
            } else {
                let exercise = repo
                    .get_exercise_for_workout_exercise(workout_exercise_id)
                    .await?;
                let mut rows = Vec::new();
                for s in sets.iter() {
                    let cluster_label = if let Some(cid) = s.cluster_id {
                        format!(" [C{}]", cid)
                    } else {
                        "".to_string()
                    };

                    let cardio_info = if s.avg_heart_rate_bpm.is_some() {
                        format!(
                            "HR: {}/{} bpm | Pace: {} | Cal: {}",
                            s.avg_heart_rate_bpm
                                .map(|v| v.to_string())
                                .unwrap_or_default(),
                            s.max_heart_rate_bpm
                                .map(|v| v.to_string())
                                .unwrap_or_default(),
                            s.avg_pace_min_per_km
                                .map(|v| v.to_string())
                                .unwrap_or_default(),
                            s.calories_burned.map(|v| v.to_string()).unwrap_or_default()
                        )
                    } else {
                        "".to_string()
                    };

                    let side_label = s
                        .side
                        .as_ref()
                        .map(|sd| sd.to_uppercase())
                        .unwrap_or_else(|| "-".to_string());
                    let phase_label = format_phase_label(&s.phase);
                    rows.push(vec![
                        s.id.to_string(),
                        format!("{}{}", s.set_number, cluster_label),
                        side_label,
                        if phase_label.is_empty() {
                            "full".to_string()
                        } else {
                            phase_label
                        },
                        s.reps.map(|r| r.to_string()).unwrap_or_default(),
                        bodyweight::format_load_display(
                            exercise
                                .as_ref()
                                .map(|e| e.load_type.as_str())
                                .unwrap_or(crate::load_type::EXTERNAL),
                            s.weight_kg,
                            s.external_load_kg,
                        ),
                        s.distance_km
                            .map(|d| format!("{:.2} km", d))
                            .unwrap_or_default(),
                        s.duration_seconds
                            .map(|d| format!("{}s", d))
                            .unwrap_or_default(),
                        cardio_info,
                        s.notes.as_ref().cloned().unwrap_or_default(),
                    ]);
                }
                // Basic context header (full exercise + workout date available via `workout view` or richer queries)
                eprintln!(
                    "Sets for workout-exercise {} ({} sets)",
                    workout_exercise_id,
                    sets.len()
                );
                print_table(
                    vec![
                        "ID", "Set #", "Side", "Phase", "Reps", "Weight", "Dist", "Dur", "Cardio",
                        "Notes",
                    ],
                    rows,
                );

                // Show Laps if available
                for s in sets {
                    if let Some(laps_json) = s.laps {
                        let laps = laps_json.0;
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
        }
        SetAction::Update {
            set_id,
            reps,
            weight,
            external_load,
            no_weight_recorded,
            duration,
            distance,
            rpe,
            rir,
            effective_reps,
            rest_seconds,
            notes,
            side,
            phase,
            dry_run,
        } => {
            let id = parse_id(&set_id, dry_run)?;
            // Verify exists for better error (and for dry-run to still validate)
            let existing = repo.get_set(id).await?;
            let existing =
                existing.ok_or_else(|| RepslogError::Cli(format!("Set {} not found", set_id)))?;
            let (resolved_weight, resolved_external_load, clear_weight) =
                resolve_load_for_set_update(
                    repo,
                    &existing,
                    weight,
                    external_load,
                    no_weight_recorded,
                    reps,
                    duration,
                )
                .await?;
            let side_norm = side.as_deref().map(|s| s.to_lowercase());
            let phase_norm = phase.as_deref().map(phase::normalize_phase).transpose()?;

            // Validate the post-update view of numeric fields (patch merges with existing).
            let merged = ProposedSetMetrics {
                reps: reps.or(existing.reps),
                weight_kg: if clear_weight {
                    None
                } else {
                    resolved_weight.or(existing.weight_kg)
                },
                external_load_kg: resolved_external_load.or(existing.external_load_kg),
                distance_km: distance.or(existing.distance_km),
                duration_seconds: duration.or(existing.duration_seconds),
                rpe: rpe.or(existing.rpe),
                rir: rir.or(existing.rir),
                effective_reps: effective_reps.or(existing.effective_reps),
                rest_seconds: rest_seconds.or(existing.rest_seconds),
                avg_heart_rate_bpm: existing.avg_heart_rate_bpm,
                max_heart_rate_bpm: existing.max_heart_rate_bpm,
                avg_pace_min_per_km: existing.avg_pace_min_per_km,
                calories_burned: existing.calories_burned,
                avg_cadence_spm: existing.avg_cadence_spm,
                total_ascent_m: existing.total_ascent_m,
                total_descent_m: existing.total_descent_m,
                heart_rate_zones: existing.heart_rate_zones.as_ref().map(|j| j.0.clone()),
                laps: existing.laps.as_ref().map(|j| j.0.clone()),
            };
            sanity::validate_set_metrics(&merged, limits)?;

            repo.update_set(
                id,
                reps,
                resolved_weight,
                clear_weight,
                resolved_external_load,
                duration,
                distance,
                rpe,
                rir,
                effective_reps,
                rest_seconds,
                notes.as_deref(),
                side_norm.as_deref(),
                phase_norm,
                dry_run,
            )
            .await?;
            if json {
                println!(r#"{{"success": true, "id": "{}"}}"#, set_id);
            } else {
                println!("Updated set {}", set_id);
            }
        }
        SetAction::Delete {
            set_id,
            force,
            dry_run,
        } => {
            let id = parse_id(&set_id, dry_run)?;
            let existing = repo.get_set(id).await?;
            if existing.is_none() {
                return Err(RepslogError::Cli(format!("Set {} not found", set_id)));
            }
            if !force {
                // Interactive confirmation unless non-tty or forced
                if !atty::is(atty::Stream::Stdin) {
                    return Err(RepslogError::Cli(
                        "Refusing to delete without --force in non-interactive mode (pipe/redirect detected).".into(),
                    ));
                }
                eprint!("Delete set {}? This cannot be undone. [y/N] ", set_id);
                let mut input = String::new();
                if std::io::stdin().read_line(&mut input).is_ok() {
                    if !input.trim().eq_ignore_ascii_case("y") {
                        println!("Aborted.");
                        return Ok(());
                    }
                } else {
                    return Err(RepslogError::Cli("Failed to read confirmation".into()));
                }
            }
            repo.delete_set(id, dry_run).await?;
            if json {
                println!(r#"{{"success": true, "id": "{}"}}"#, set_id);
            } else {
                println!("Deleted set {}", set_id);
            }
        }
        SetAction::Move {
            set_id,
            to,
            dry_run,
        } => {
            let id = parse_id(&set_id, dry_run)?;
            if to < 1 {
                return Err(RepslogError::Cli("Target position must be >= 1".into()));
            }
            let existing = repo.get_set(id).await?;
            if existing.is_none() {
                return Err(RepslogError::Cli(format!("Set {} not found", set_id)));
            }
            repo.reorder_set(id, to, dry_run).await?;
            if json {
                println!(
                    r#"{{"success": true, "id": "{}", "new_position": {}}}"#,
                    set_id, to
                );
            } else {
                println!("Moved set {} to position {}", set_id, to);
            }
        }
        SetAction::AddUnilateral {
            workout_exercise_id,
            reps,
            weight,
            external_load,
            no_weight_recorded,
            rir,
            effective_reps,
            rest_seconds,
            notes,
            side,
            phase,
            dry_run,
        } => {
            let resolved_phase = phase::normalize_phase(&phase)?;
            let id_str = if let Some(id) = workout_exercise_id {
                id
            } else if let Some(stdin_id) = read_stdin() {
                stdin_id
            } else {
                return Err(RepslogError::Cli(
                    "No workout-exercise-id provided. Use --help for examples.".into(),
                ));
            };
            let id = parse_id(&id_str, dry_run)?;

            let reps_list: Vec<i32> = reps
                .split(',')
                .map(|s| {
                    s.trim()
                        .parse()
                        .map_err(|_| RepslogError::Cli(format!("Invalid reps: {}", s)))
                })
                .collect::<Result<_>>()?;

            let rir_list: Vec<Option<f64>> = if let Some(r) = &rir {
                r.split(',')
                    .map(|s| {
                        let t = s.trim();
                        if t.is_empty() {
                            Ok(None)
                        } else {
                            t.parse()
                                .map(Some)
                                .map_err(|_| RepslogError::Cli(format!("Invalid rir: {}", s)))
                        }
                    })
                    .collect::<Result<_>>()?
            } else {
                vec![None; reps_list.len()]
            };

            let eff_list: Vec<Option<i32>> = if let Some(e) = &effective_reps {
                e.split(',')
                    .map(|s| {
                        let t = s.trim();
                        if t.is_empty() {
                            Ok(None)
                        } else {
                            t.parse().map(Some).map_err(|_| {
                                RepslogError::Cli(format!("Invalid effective-reps: {}", s))
                            })
                        }
                    })
                    .collect::<Result<_>>()?
            } else {
                vec![None; reps_list.len()]
            };

            if reps_list.len() != rir_list.len() || reps_list.len() != eff_list.len() {
                return Err(RepslogError::Cli("reps, rir, and effective-reps lists must have matching lengths (or omit rir/eff)".into()));
            }

            let side_norm = side.to_lowercase();
            let sides: Vec<&str> = match side_norm.as_str() {
                "both" => vec!["left", "right"],
                "left" | "right" => vec![side_norm.as_str()],
                _ => {
                    return Err(RepslogError::Cli(
                        "side must be left, right, or both".into(),
                    ))
                }
            };

            let (resolved_weight, resolved_external_load) = resolve_load_for_workout_exercise(
                repo,
                id,
                &id_str,
                dry_run,
                weight,
                external_load,
                no_weight_recorded,
                Some(1),
                None,
            )
            .await?;

            let mut created = Vec::new();
            for &sd in &sides {
                for (i, &r) in reps_list.iter().enumerate() {
                    let ri = rir_list.get(i).and_then(|v| *v);
                    let eff = eff_list.get(i).and_then(|v| *v);
                    let rest = if i > 0 { rest_seconds } else { None };
                    sanity::validate_set_metrics(
                        &ProposedSetMetrics {
                            reps: Some(r),
                            weight_kg: resolved_weight,
                            external_load_kg: resolved_external_load,
                            rir: ri,
                            effective_reps: eff,
                            rest_seconds: rest,
                            ..Default::default()
                        },
                        limits,
                    )?;
                    let set_number = if dry_run && id_str.starts_with("DRY-RUN-") {
                        // approximate; real sequencing happens in non-dry
                        (i + 1) as i32
                    } else {
                        repo.get_next_set_number(id).await?
                    };

                    let set_id = repo
                        .add_set(
                            id,
                            set_number,
                            Some(r),
                            resolved_weight,
                            resolved_external_load,
                            None,
                            None,
                            None,
                            ri,
                            eff,
                            None,
                            rest,
                            notes.as_deref(),
                            Some(sd),
                            resolved_phase,
                            None,
                            None,
                            None,
                            None,
                            None,
                            None,
                            None,
                            None,
                            None,
                            None, // date_of_birth
                            None, // resting_hr_bpm
                            dry_run,
                        )
                        .await?;
                    created.push(format_dry_run_id(set_id, dry_run));
                }
            }
            if json {
                print_json(&created)?;
            } else {
                eprintln!(
                    "Added {} unilateral set(s) to workout-exercise {}. IDs: {:?}",
                    created.len(),
                    id_str,
                    created
                );
                for c in &created {
                    println!("{}", c);
                }
            }
        }
        SetAction::Quick {
            workout_id,
            exercise_name_or_id,
            reps,
            weight,
            external_load,
            no_weight_recorded,
            duration,
            notes,
            phase,
            dry_run,
        } => {
            let w_id = parse_id(&workout_id, dry_run)?;
            let exercise = repo
                .find_exercise_by_id_or_name(&exercise_name_or_id)
                .await?;
            if let Some(ex) = exercise {
                let order = if dry_run && workout_id.starts_with("DRY-RUN-") {
                    1
                } else {
                    repo.get_max_order_for_workout(w_id).await? + 1
                };
                let we_id = repo
                    .add_workout_exercise(w_id, ex.id, order, None, None, dry_run)
                    .await?;
                let logging_set =
                    bodyweight::is_strength_metric_set(reps, weight, duration, external_load);
                if logging_set {
                    let resolved_phase = phase
                        .as_deref()
                        .ok_or_else(|| {
                            RepslogError::Cli(
                                "--phase is required when logging a set with set quick.".into(),
                            )
                        })
                        .and_then(|p| phase::normalize_phase(p))?;
                    let _id_str = format_dry_run_id(we_id, dry_run);
                    let (resolved_weight, resolved_external_load) =
                        if dry_run && workout_id.starts_with("DRY-RUN-") {
                            (weight, external_load)
                        } else {
                            bodyweight::resolve_bodyweight_load(
                                &ex,
                                weight,
                                external_load,
                                no_weight_recorded,
                                bodyweight::uses_body_mass(&ex),
                            )?
                        };
                    sanity::validate_set_metrics(
                        &ProposedSetMetrics {
                            reps,
                            weight_kg: resolved_weight,
                            external_load_kg: resolved_external_load,
                            duration_seconds: duration,
                            ..Default::default()
                        },
                        limits,
                    )?;
                    let set_id = repo
                        .add_set(
                            we_id,
                            1,
                            reps,
                            resolved_weight,
                            resolved_external_load,
                            duration,
                            None,
                            None,
                            None,
                            None,
                            None,
                            None,
                            notes.as_deref(),
                            None,
                            resolved_phase,
                            None,
                            None,
                            None,
                            None,
                            None,
                            None,
                            None,
                            None,
                            None,
                            None, // date_of_birth
                            None, // resting_hr_bpm
                            dry_run,
                        )
                        .await?;
                    let formatted_set_id = format_dry_run_id(set_id, dry_run);
                    if json {
                        print_id(&formatted_set_id, true);
                    } else {
                        println!(
                            "Added exercise {} to workout {} and created first set with ID {}",
                            ex.name, workout_id, formatted_set_id
                        );
                    }
                } else if json {
                    print_id(&format_dry_run_id(we_id, dry_run), true);
                } else {
                    println!(
                        "Added exercise {} to workout {} (WE ID {}). Log sets with repslog set add.",
                        ex.name, workout_id, format_dry_run_id(we_id, dry_run)
                    );
                    if bodyweight::uses_body_mass(&ex) {
                        eprintln!(
                            "Tip: body-mass exercises (load_type=body_mass) require --weight <body-mass-kg> on each set."
                        );
                    }
                }
            } else {
                println!("Exercise not found: {}", exercise_name_or_id);
            }
        }
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn resolve_load_for_workout_exercise(
    repo: &Repository,
    workout_exercise_id: i64,
    id_str: &str,
    dry_run: bool,
    weight: Option<f64>,
    external_load: Option<f64>,
    no_weight_recorded: bool,
    reps: Option<i32>,
    duration: Option<i32>,
) -> Result<(Option<f64>, Option<f64>)> {
    if dry_run && id_str.starts_with("DRY-RUN-") {
        return Ok((weight, external_load));
    }
    let exercise = repo
        .get_exercise_for_workout_exercise(workout_exercise_id)
        .await?
        .ok_or_else(|| {
            RepslogError::Cli(format!(
                "Workout-exercise {} not found",
                workout_exercise_id
            ))
        })?;
    let requires_body_weight =
        bodyweight::is_strength_metric_set(reps, weight, duration, external_load)
            && bodyweight::uses_body_mass(&exercise);
    bodyweight::resolve_bodyweight_load(
        &exercise,
        weight,
        external_load,
        no_weight_recorded,
        requires_body_weight,
    )
}

async fn resolve_load_for_set_update(
    repo: &Repository,
    existing: &crate::models::ExerciseSet,
    weight: Option<f64>,
    external_load: Option<f64>,
    no_weight_recorded: bool,
    reps: Option<i32>,
    duration: Option<i32>,
) -> Result<(Option<f64>, Option<f64>, bool)> {
    let exercise = repo
        .get_exercise_for_workout_exercise(existing.workout_exercise_id)
        .await?
        .ok_or_else(|| {
            RepslogError::Cli(format!(
                "Workout-exercise {} not found",
                existing.workout_exercise_id
            ))
        })?;

    if !bodyweight::uses_body_mass(&exercise) {
        bodyweight::validate_external_load(&exercise.load_type, external_load)?;
        if no_weight_recorded {
            return Err(RepslogError::Cli(
                "--no-weight-recorded is only valid for body-mass exercises (load_type=body_mass)."
                    .into(),
            ));
        }
        return Ok((weight, external_load, false));
    }

    let final_reps = reps.or(existing.reps);
    let final_duration = duration.or(existing.duration_seconds);
    let touches_strength = reps.is_some()
        || duration.is_some()
        || weight.is_some()
        || external_load.is_some()
        || no_weight_recorded
        || final_reps.is_some()
        || final_duration.is_some()
        || existing.weight_kg.is_some()
        || existing.external_load_kg.is_some();

    if !touches_strength {
        return Ok((weight, external_load, false));
    }

    bodyweight::validate_external_load(&exercise.load_type, external_load)?;

    if no_weight_recorded {
        if weight.is_some() {
            return Err(RepslogError::Cli(
                "Cannot use --weight together with --no-weight-recorded.".into(),
            ));
        }
        eprintln!("{}", bodyweight::NO_WEIGHT_WARNING);
        return Ok((None, external_load, true));
    }

    if let Some(w) = weight {
        if w <= 0.0 {
            return Err(RepslogError::Cli(
                "Body weight must be a positive value in kg.".into(),
            ));
        }
        return Ok((Some(w), external_load, false));
    }

    if existing.weight_kg.is_some() {
        return Ok((None, external_load, false));
    }

    if final_reps.is_some() || final_duration.is_some() {
        return Err(RepslogError::Cli(format!(
            "Exercise '{}' (load_type=body_mass) requires --weight <kg> (your body mass) \
             or --no-weight-recorded (not recommended).",
            exercise.name
        )));
    }

    Ok((None, external_load, false))
}

#[allow(clippy::explicit_counter_loop)]
fn validate_laps(
    laps: &[Lap],
    total_distance: Option<f64>,
    total_duration_seconds: Option<u32>,
) -> Result<()> {
    let mut sum_dist = 0.0;
    let mut sum_dur = 0;
    let mut expected_lap = 1;

    for lap in laps {
        if lap.lap_number != expected_lap {
            return Err(RepslogError::Cli(format!(
                "Laps must be sequential. Expected lap {}, got {}",
                expected_lap, lap.lap_number
            )));
        }
        if lap.distance_km <= 0.0 {
            return Err(RepslogError::Cli(format!(
                "Lap {} distance must be greater than 0",
                lap.lap_number
            )));
        }
        if lap.duration_seconds == 0 {
            return Err(RepslogError::Cli(format!(
                "Lap {} duration must be greater than 0",
                lap.lap_number
            )));
        }
        sum_dist += lap.distance_km;
        sum_dur += lap.duration_seconds;
        expected_lap += 1;
    }

    if let Some(total_d) = total_distance {
        if (sum_dist - total_d).abs() > total_d * 0.011 {
            // ~1% allowance
            eprintln!("Warning: Sum of lap distances ({:.2} km) differs from total distance ({:.2} km) by more than 1%", sum_dist, total_d);
        }
    }

    if let Some(total_dur) = total_duration_seconds {
        if sum_dur != total_dur && (sum_dur as i32 - total_dur as i32).abs() > 2 {
            eprintln!(
                "Warning: Sum of lap durations ({}s) differs from total duration ({}s)",
                sum_dur, total_dur
            );
        }
    }

    Ok(())
}
