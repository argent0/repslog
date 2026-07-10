use crate::app_config::SanityLimits;
use crate::cli::ImportAction;
use crate::error::{RepslogError, Result};
use crate::fit::{parse_fit_bytes, FitActivity, ImportPlan};
use crate::repository::Repository;
use crate::sanity::{self, ProposedSetMetrics, ProposedWorkoutMetrics};
use crate::utils::{
    find_exercise_name_conflicts, format_datetime, format_dry_run_id, format_duration, format_pace,
    normalize_exercise_name, normalize_exercise_name_lenient, print_id, print_json,
};
use sha2::{Digest, Sha256};
use sqlx::types::Json;
use std::fs;
use std::path::Path;

pub async fn handle_import(
    action: ImportAction,
    repo: &Repository,
    limits: &SanityLimits,
    json: bool,
) -> Result<()> {
    match action {
        ImportAction::Fit {
            path,
            exercise,
            workout_type,
            notes,
            force,
            hr_zone_bounds,
            dry_run,
        } => {
            import_fit(
                repo,
                &path,
                exercise.as_deref(),
                workout_type.as_deref(),
                notes.as_deref(),
                force,
                hr_zone_bounds.as_ref(),
                dry_run,
                limits,
                json,
            )
            .await
        }
    }
}

#[allow(clippy::too_many_arguments)]
async fn import_fit(
    repo: &Repository,
    path: &str,
    exercise_override: Option<&str>,
    workout_type: Option<&str>,
    notes: Option<&str>,
    force: bool,
    hr_zone_bounds: Option<&[f64; 5]>,
    dry_run: bool,
    limits: &SanityLimits,
    json: bool,
) -> Result<()> {
    let path = Path::new(path);
    if !path.exists() {
        return Err(RepslogError::Cli(format!(
            "FIT file not found: {}",
            path.display()
        )));
    }

    let bytes = fs::read(path)
        .map_err(|e| RepslogError::Cli(format!("Failed to read '{}': {}", path.display(), e)))?;
    let file_sha256 = hex::encode(Sha256::digest(&bytes));
    let source_filename = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("activity.fit")
        .to_string();

    if let Some(existing) = repo.get_import_by_hash(&file_sha256).await? {
        if !force {
            return Err(RepslogError::Cli(format!(
                "This FIT file was already imported as workout {} (sha256 {}). Use --force to import again (previous workout is kept).",
                existing.workout_id, file_sha256
            )));
        }
        if !dry_run {
            repo.delete_activity_import_by_hash(&file_sha256).await?;
        }
    }

    let activity = parse_fit_bytes(&bytes)?;
    let plan = ImportPlan::from_activity(
        &activity,
        workout_type,
        notes,
        &source_filename,
        hr_zone_bounds,
    )?;

    // Absolute sanity checks before any DB writes
    sanity::validate_workout_metrics(
        &ProposedWorkoutMetrics {
            duration_minutes: plan.duration_minutes,
            overall_feeling: None,
        },
        limits,
    )?;
    sanity::validate_set_metrics(
        &ProposedSetMetrics {
            distance_km: Some(plan.distance_km),
            duration_seconds: Some(plan.duration_seconds),
            avg_heart_rate_bpm: plan.avg_heart_rate_bpm,
            max_heart_rate_bpm: plan.max_heart_rate_bpm,
            avg_pace_min_per_km: plan.avg_pace_min_per_km,
            calories_burned: plan.calories_burned,
            avg_cadence_spm: plan.avg_cadence_spm,
            total_ascent_m: plan.total_ascent_m,
            total_descent_m: plan.total_descent_m,
            heart_rate_zones: plan.heart_rate_zones.clone(),
            laps: plan.laps.clone(),
            ..Default::default()
        },
        limits,
    )?;
    if !plan.trackpoints.is_empty() {
        sanity::validate_trackpoints(&plan.trackpoints, limits, 20)?;
    }

    let exercise_name = resolve_exercise_name(exercise_override, &activity)?;
    let exercise_id = require_catalog_exercise(repo, &exercise_name, path).await?;

    let workout_id = repo
        .create_workout(
            Some(&plan.workout_type),
            plan.notes.as_deref(),
            Some(&plan.started_at),
            dry_run,
        )
        .await?;

    if !dry_run {
        repo.update_workout(
            workout_id,
            None,
            None,
            plan.duration_minutes,
            None,
            None,
            false,
        )
        .await?;
    }

    let we_id = repo
        .add_workout_exercise(workout_id, exercise_id, 1, None, None, dry_run)
        .await?;

    let set_id = repo
        .add_set(
            we_id,
            1,
            None,
            None,
            None,
            Some(plan.duration_seconds),
            Some(plan.distance_km),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            crate::phase::FULL,
            plan.avg_heart_rate_bpm,
            plan.max_heart_rate_bpm,
            plan.heart_rate_zones.clone().map(Json),
            plan.avg_pace_min_per_km,
            plan.calories_burned,
            plan.laps.clone().map(Json),
            plan.avg_cadence_spm,
            plan.total_ascent_m,
            plan.total_descent_m,
            dry_run,
        )
        .await?;

    if !plan.trackpoints.is_empty() {
        repo.insert_trackpoints_batch(set_id, &plan.trackpoints, dry_run)
            .await?;
    }

    repo.insert_activity_import(
        workout_id,
        "fit",
        Some(&source_filename),
        &file_sha256,
        plan.device_name.as_deref(),
        plan.manufacturer_id,
        plan.product_id,
        plan.fit_sport,
        plan.fit_sub_sport,
        dry_run,
    )
    .await?;

    let formatted_id = format_dry_run_id(workout_id, dry_run);

    if json {
        #[derive(serde::Serialize)]
        struct ImportOut {
            workout_id: String,
            exercise: String,
            started_at: String,
            distance_km: f64,
            duration_seconds: i32,
            avg_pace_min_per_km: Option<f64>,
            avg_heart_rate_bpm: Option<f64>,
            max_heart_rate_bpm: Option<f64>,
            calories_burned: Option<i32>,
            avg_cadence_spm: Option<f64>,
            total_ascent_m: Option<f64>,
            total_descent_m: Option<f64>,
            trackpoints_stored: usize,
            dry_run: bool,
            file_sha256: String,
        }
        print_json(&ImportOut {
            workout_id: formatted_id,
            exercise: exercise_name,
            started_at: format_datetime(&plan.started_at),
            distance_km: plan.distance_km,
            duration_seconds: plan.duration_seconds,
            avg_pace_min_per_km: plan.avg_pace_min_per_km,
            avg_heart_rate_bpm: plan.avg_heart_rate_bpm,
            max_heart_rate_bpm: plan.max_heart_rate_bpm,
            calories_burned: plan.calories_burned,
            avg_cadence_spm: plan.avg_cadence_spm,
            total_ascent_m: plan.total_ascent_m,
            total_descent_m: plan.total_descent_m,
            trackpoints_stored: plan.trackpoints.len(),
            dry_run,
            file_sha256,
        })?;
    } else {
        eprintln!(
            "Imported {} · {:.2} km · {} · {} · HR {}/{} · {} kcal{}",
            source_filename,
            plan.distance_km,
            format_duration(plan.duration_seconds as u32),
            plan.avg_pace_min_per_km
                .map(format_pace)
                .unwrap_or_else(|| "—".into()),
            plan.avg_heart_rate_bpm
                .map(|h| format!("{:.0}", h))
                .unwrap_or_else(|| "—".into()),
            plan.max_heart_rate_bpm
                .map(|h| format!("{:.0}", h))
                .unwrap_or_else(|| "—".into()),
            plan.calories_burned
                .map(|c| c.to_string())
                .unwrap_or_else(|| "—".into()),
            if dry_run { " (dry-run)" } else { "" },
        );
        eprintln!("Created workout with ID {}", formatted_id);
        print_id(&formatted_id, false);
    }

    Ok(())
}

/// Resolve catalog exercise name from optional CLI override or FIT session.sport.
fn resolve_exercise_name(
    exercise_override: Option<&str>,
    activity: &FitActivity,
) -> Result<String> {
    if let Some(name) = exercise_override {
        return normalize_exercise_name(name);
    }

    if let Some(sport) = activity.sport.as_deref() {
        let name = normalize_exercise_name_lenient(sport);
        if !name.is_empty() {
            return Ok(name);
        }
    }

    // Fallback: known FIT sport ids (Garmin profile: running = 1)
    if activity.sport_id == Some(1) {
        return Ok("running".to_string());
    }

    Err(RepslogError::Cli(
        "FIT file has no session.sport; cannot determine exercise. \
         Pass --exercise <name> to override, or fix the export."
            .into(),
    ))
}

/// Look up exercise; abort with similar-name hints and an add recipe if missing.
async fn require_catalog_exercise(
    repo: &Repository,
    exercise_name: &str,
    fit_path: &Path,
) -> Result<i64> {
    if let Some(ex) = repo.find_exercise_by_id_or_name(exercise_name).await? {
        return Ok(ex.id);
    }

    let catalog = repo.list_exercises(None, None).await?;
    let catalog_pairs: Vec<(i64, String)> =
        catalog.iter().map(|e| (e.id, e.name.clone())).collect();
    let similar = find_exercise_name_conflicts(exercise_name, &catalog_pairs);

    let mut msg = format!(
        "No catalog exercise matching '{}' (from FIT sport or --exercise).\n",
        exercise_name
    );

    if similar.is_empty() {
        msg.push_str("\nNo similarly named exercises found in the catalog.\n");
    } else {
        msg.push_str("\nSimilar exercises:\n");
        for c in &similar {
            msg.push_str(&format!("  - {} (id {})\n", c.existing_name, c.existing_id));
        }
        msg.push_str(
            "\nIf one of these is correct, re-import with --exercise <name>, or rename/add to match the FIT sport.\n",
        );
    }

    let path_display = fit_path.display();
    msg.push_str(&format!(
        "\nAdd a new exercise, then re-import:\n  \
         repslog exercise add \"{name}\" --category cardio --equipment none --load-type none\n  \
         repslog import fit {path}\n\n\
         Search catalog: repslog exercise search {name}",
        name = exercise_name,
        path = path_display,
    ));

    Err(RepslogError::Cli(msg))
}
