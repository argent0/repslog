use repslog::app_config::SanityLimits;
use repslog::cli::ImportAction;
use repslog::commands::import;
use repslog::db::setup_test_db;
use repslog::fit::{parse_fit_path, ImportPlan};
use repslog::repository::Repository;
use repslog::track_metrics::{compute, compute_with_zones, ZoneRecomputeContext};
use std::path::PathBuf;

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/Zepp20260710164935.fit")
}

#[tokio::test]
async fn import_fit_creates_running_workout() {
    let path = fixture_path();
    if !path.exists() {
        eprintln!("fixture missing: {:?}", path);
        return;
    }

    let pool = setup_test_db().await.unwrap();
    let repo = Repository::new(pool);

    // Seed running exercise like init does
    let _ = repo
        .add_exercise(
            "running",
            "cardio",
            Some("[\"legs\", \"cardiovascular\"]"),
            Some("none"),
            "none",
            Some("Outdoor or treadmill run"),
            false,
            false,
        )
        .await
        .unwrap();

    let limits = SanityLimits::default();
    import::handle_import(
        ImportAction::Fit {
            path: path.to_string_lossy().to_string(),
            exercise: None,
            workout_type: Some("Run".into()),
            notes: Some("test import".into()),
            force: false,
            hr_zone_bounds: None,
            no_bodylog: true,
            dry_run: false,
        },
        &repo,
        &limits,
        true,
    )
    .await
    .unwrap();

    let workouts = repo.list_workouts(10, None).await.unwrap();
    assert_eq!(workouts.len(), 1);
    let w = &workouts[0];
    assert_eq!(w.workout_type.as_deref(), Some("Run"));
    assert!(w.notes.as_ref().unwrap().contains("test import"));
    assert!(w.started_at.starts_with("2026-07-10"));

    let wes = repo.list_workout_exercises(w.id).await.unwrap();
    assert_eq!(wes.len(), 1);
    assert_eq!(wes[0].1, "running");

    let sets = repo.list_sets(wes[0].0.id).await.unwrap();
    assert_eq!(sets.len(), 1);
    let s = &sets[0];
    assert!((s.distance_km.unwrap() - 8.027).abs() < 0.02);
    assert_eq!(s.duration_seconds, Some(2808));
    assert_eq!(s.avg_heart_rate_bpm, Some(156.0));
    assert_eq!(s.max_heart_rate_bpm, Some(175.0));
    assert_eq!(s.calories_burned, Some(597));
    assert_eq!(s.avg_cadence_spm, Some(77.0));
    assert_eq!(s.total_ascent_m, Some(12.0));
    assert_eq!(s.total_descent_m, Some(11.0));
    assert!(s.avg_pace_min_per_km.unwrap() > 5.0 && s.avg_pace_min_per_km.unwrap() < 6.5);
    // Single-lap FIT → no laps JSON
    assert!(s.laps.is_none() || s.laps.as_ref().map(|j| j.0.is_empty()).unwrap_or(true));
    // Record stream is always stored when present
    let n = repo.count_trackpoints(s.id).await.unwrap();
    assert!(n > 1000, "expected many trackpoints, got {}", n);

    // Duration minutes on workout
    assert_eq!(w.duration_minutes, Some(47)); // 2808/60 ≈ 46.8 → 47

    // Duplicate import fails
    let err = import::handle_import(
        ImportAction::Fit {
            path: path.to_string_lossy().to_string(),
            exercise: None,
            workout_type: None,
            notes: None,
            force: false,
            hr_zone_bounds: None,
            no_bodylog: true,
            dry_run: false,
        },
        &repo,
        &limits,
        true,
    )
    .await;
    assert!(err.is_err());
    assert!(format!("{}", err.unwrap_err()).contains("already imported"));
}

#[tokio::test]
async fn track_metrics_from_imported_fit() {
    let path = fixture_path();
    if !path.exists() {
        eprintln!("fixture missing: {:?}", path);
        return;
    }

    let pool = setup_test_db().await.unwrap();
    let repo = Repository::new(pool);

    let _ = repo
        .add_exercise(
            "running",
            "cardio",
            Some("[\"legs\", \"cardiovascular\"]"),
            Some("none"),
            "none",
            Some("Outdoor or treadmill run"),
            false,
            false,
        )
        .await
        .unwrap();

    let limits = SanityLimits::default();
    import::handle_import(
        ImportAction::Fit {
            path: path.to_string_lossy().to_string(),
            exercise: None,
            workout_type: Some("Run".into()),
            notes: None,
            force: false,
            hr_zone_bounds: None,
            no_bodylog: true,
            dry_run: false,
        },
        &repo,
        &limits,
        true,
    )
    .await
    .unwrap();

    let workouts = repo.list_workouts(10, None).await.unwrap();
    let w = &workouts[0];
    let wes = repo.list_workout_exercises(w.id).await.unwrap();
    let sets = repo.list_sets(wes[0].0.id).await.unwrap();
    let s = &sets[0];

    let n = repo.count_trackpoints(s.id).await.unwrap();
    let points = repo.list_trackpoints(s.id).await.unwrap();
    assert_eq!(points.len() as i64, n);
    assert!(n > 1000, "expected many trackpoints, got {}", n);

    let m = compute(&points, s.distance_km).expect("track metrics");
    assert_eq!(m.sample_count, points.len());
    assert!(m.elapsed_seconds > 0);
    assert!(m.moving_seconds <= m.elapsed_seconds + 5);
    assert!(m.moving_seconds <= 2808 + 60);
    if let Some(ref route) = m.route {
        if let Some(gps) = route.gps_distance_km {
            assert!(
                (gps - 8.027).abs() < 0.5,
                "gps distance {} not near 8.027",
                gps
            );
        }
    }
    assert!(m.elev_min_m.is_some() || m.elev_max_m.is_some() || m.ascent_m.is_some());
    assert!(
        m.best_efforts.iter().any(|b| b.label == "1 km"),
        "expected 1 km best effort: {:?}",
        m.best_efforts
    );
    assert!(
        m.synthetic_km_splits.iter().filter(|k| !k.partial).count() >= 7,
        "expected ~8 km splits, got {:?}",
        m.synthetic_km_splits.len()
    );

    // Zone recompute only when DOB snapshot present (none with --no-bodylog)
    let ctx = ZoneRecomputeContext {
        date_of_birth: s.date_of_birth.clone(),
        resting_hr_bpm: s.resting_hr_bpm,
        activity_date: Some(w.started_at[..10].to_string()),
    };
    let m2 = compute_with_zones(&points, s.distance_km, &ctx).unwrap();
    if s.date_of_birth.is_none() {
        assert!(m2.hr_zones_recomputed.is_none());
    }
}

#[tokio::test]
async fn import_fit_missing_exercise_aborts_with_suggestion() {
    let path = fixture_path();
    if !path.exists() {
        return;
    }
    let pool = setup_test_db().await.unwrap();
    let repo = Repository::new(pool);

    // Seed a similar name but not exact "running"
    let _ = repo
        .add_exercise(
            "run",
            "cardio",
            Some("[\"legs\", \"cardiovascular\"]"),
            Some("none"),
            "none",
            None,
            true,
            false,
        )
        .await
        .unwrap();

    let limits = SanityLimits::default();
    let err = import::handle_import(
        ImportAction::Fit {
            path: path.to_string_lossy().to_string(),
            exercise: None,
            workout_type: None,
            notes: None,
            force: false,
            hr_zone_bounds: None,
            no_bodylog: true,
            dry_run: false,
        },
        &repo,
        &limits,
        true,
    )
    .await
    .unwrap_err();

    let msg = format!("{}", err);
    assert!(
        msg.contains("No catalog exercise matching 'running'"),
        "msg={}",
        msg
    );
    assert!(msg.contains("exercise add"), "msg={}", msg);
    assert!(msg.contains("run"), "similar hint missing: {}", msg);

    // No workout written
    let workouts = repo.list_workouts(10, None).await.unwrap();
    assert!(workouts.is_empty());
}

#[tokio::test]
async fn import_fit_exercise_override_is_case_insensitive() {
    let path = fixture_path();
    if !path.exists() {
        return;
    }
    let pool = setup_test_db().await.unwrap();
    let repo = Repository::new(pool);

    let _ = repo
        .add_exercise(
            "running",
            "cardio",
            Some("[\"legs\", \"cardiovascular\"]"),
            Some("none"),
            "none",
            None,
            false,
            false,
        )
        .await
        .unwrap();

    let limits = SanityLimits::default();
    import::handle_import(
        ImportAction::Fit {
            path: path.to_string_lossy().to_string(),
            exercise: Some("Running".into()),
            workout_type: Some("Run".into()),
            notes: None,
            force: false,
            hr_zone_bounds: None,
            no_bodylog: true,
            dry_run: false,
        },
        &repo,
        &limits,
        true,
    )
    .await
    .unwrap();

    let workouts = repo.list_workouts(10, None).await.unwrap();
    assert_eq!(workouts.len(), 1);
    let wes = repo.list_workout_exercises(workouts[0].id).await.unwrap();
    assert_eq!(wes[0].1, "running");
}

#[tokio::test]
async fn import_fit_dry_run_writes_nothing() {
    let path = fixture_path();
    if !path.exists() {
        return;
    }
    let pool = setup_test_db().await.unwrap();
    let repo = Repository::new(pool);

    let _ = repo
        .add_exercise(
            "running",
            "cardio",
            Some("[\"legs\", \"cardiovascular\"]"),
            Some("none"),
            "none",
            None,
            false,
            false,
        )
        .await
        .unwrap();

    let limits = SanityLimits::default();
    import::handle_import(
        ImportAction::Fit {
            path: path.to_string_lossy().to_string(),
            exercise: None,
            workout_type: None,
            notes: None,
            force: false,
            hr_zone_bounds: None,
            no_bodylog: true,
            dry_run: true,
        },
        &repo,
        &limits,
        true,
    )
    .await
    .unwrap();

    let workouts = repo.list_workouts(10, None).await.unwrap();
    assert!(workouts.is_empty());
}

#[tokio::test]
async fn import_fit_hr_zone_bounds() {
    let path = fixture_path();
    if !path.exists() {
        return;
    }
    let pool = setup_test_db().await.unwrap();
    let repo = Repository::new(pool);

    let _ = repo
        .add_exercise(
            "running",
            "cardio",
            Some("[\"legs\", \"cardiovascular\"]"),
            Some("none"),
            "none",
            None,
            false,
            false,
        )
        .await
        .unwrap();

    let limits = SanityLimits::default();
    import::handle_import(
        ImportAction::Fit {
            path: path.to_string_lossy().to_string(),
            exercise: None,
            workout_type: None,
            notes: None,
            force: false,
            hr_zone_bounds: Some([120.0, 140.0, 160.0, 175.0, 200.0]),
            no_bodylog: true,
            dry_run: false,
        },
        &repo,
        &limits,
        true,
    )
    .await
    .unwrap();

    let workouts = repo.list_workouts(1, None).await.unwrap();
    let wes = repo.list_workout_exercises(workouts[0].id).await.unwrap();
    let sets = repo.list_sets(wes[0].0.id).await.unwrap();
    let s = &sets[0];
    assert!(s.heart_rate_zones.is_some());
    let zones = &s.heart_rate_zones.as_ref().unwrap().0;
    let total = zones.z1_seconds
        + zones.z2_seconds
        + zones.z3_seconds
        + zones.z4_seconds
        + zones.z5_seconds;
    assert!(
        total > 1000,
        "expected substantial zone time, got {}",
        total
    );

    let n = repo.count_trackpoints(s.id).await.unwrap();
    assert!(n > 1000, "expected many trackpoints, got {}", n);
}

#[test]
fn import_plan_rejects_non_running() {
    let path = fixture_path();
    if !path.exists() {
        return;
    }
    let mut act = parse_fit_path(&path).unwrap();
    act.sport = Some("cycling".into());
    act.sport_id = Some(2);
    let err = ImportPlan::from_activity(&act, None, None, "x.fit", None, None);
    assert!(err.is_err());
}
