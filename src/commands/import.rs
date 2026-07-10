use crate::app_config::SanityLimits;
use crate::cli::ImportAction;
use crate::error::{RepslogError, Result};
use crate::fit::{parse_fit_bytes, ImportPlan};
use crate::load_type;
use crate::repository::Repository;
use crate::sanity::{self, ProposedSetMetrics, ProposedWorkoutMetrics};
use crate::utils::{
    format_datetime, format_dry_run_id, format_duration, format_pace, print_id, print_json,
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
            store_track,
            hr_zone_bounds,
            dry_run,
        } => {
            import_fit(
                repo,
                &path,
                &exercise,
                workout_type.as_deref(),
                notes.as_deref(),
                force,
                store_track,
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
    exercise_name: &str,
    workout_type: Option<&str>,
    notes: Option<&str>,
    force: bool,
    store_track: bool,
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
        store_track,
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
    if store_track && !plan.trackpoints.is_empty() {
        sanity::validate_trackpoints(&plan.trackpoints, limits, 20)?;
    }

    // Resolve exercise (required name; create as cardio if missing)
    let exercise_name = exercise_name.trim();
    if exercise_name.is_empty() {
        return Err(RepslogError::Cli(
            "--exercise is required and must not be empty".into(),
        ));
    }

    let exercise_id = resolve_or_create_exercise(repo, exercise_name, dry_run).await?;

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

    if store_track && !plan.trackpoints.is_empty() {
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
            exercise: exercise_name.to_string(),
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
            trackpoints_stored: if store_track {
                plan.trackpoints.len()
            } else {
                0
            },
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

async fn resolve_or_create_exercise(repo: &Repository, name: &str, dry_run: bool) -> Result<i64> {
    let existing = repo.list_exercises(Some(name.to_string()), None).await?;
    if let Some(ex) = existing.iter().find(|e| e.name.eq_ignore_ascii_case(name)) {
        return Ok(ex.id);
    }
    // Exact-ish: prefer exact match after normalization
    if let Some(ex) = existing.iter().find(|e| e.name == name) {
        return Ok(ex.id);
    }

    // Create cardio exercise matching init seed defaults for Running-like work
    repo.add_exercise(
        name,
        "cardio",
        Some("[\"legs\", \"cardiovascular\"]"),
        Some("none"),
        load_type::NONE,
        Some("Imported / outdoor or treadmill run"),
        true,
        dry_run,
    )
    .await
}
