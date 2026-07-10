use repslog::app_config::SanityLimits;
use repslog::cli::ImportAction;
use repslog::commands::import;
use repslog::db::setup_test_db;
use repslog::fit::{parse_fit_path, ImportPlan};
use repslog::repository::Repository;
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

    // Seed Running exercise like init does
    let _ = repo
        .add_exercise(
            "Running",
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
            exercise: "Running".into(),
            workout_type: Some("Run".into()),
            notes: Some("test import".into()),
            force: false,
            store_track: false,
            hr_zone_bounds: None,
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
    assert_eq!(wes[0].1, "Running");

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

    // Duration minutes on workout
    assert_eq!(w.duration_minutes, Some(47)); // 2808/60 ≈ 46.8 → 47

    // Duplicate import fails
    let err = import::handle_import(
        ImportAction::Fit {
            path: path.to_string_lossy().to_string(),
            exercise: "Running".into(),
            workout_type: None,
            notes: None,
            force: false,
            store_track: false,
            hr_zone_bounds: None,
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
async fn import_fit_dry_run_writes_nothing() {
    let path = fixture_path();
    if !path.exists() {
        return;
    }
    let pool = setup_test_db().await.unwrap();
    let repo = Repository::new(pool);

    let limits = SanityLimits::default();
    import::handle_import(
        ImportAction::Fit {
            path: path.to_string_lossy().to_string(),
            exercise: "Running".into(),
            workout_type: None,
            notes: None,
            force: false,
            store_track: false,
            hr_zone_bounds: None,
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
async fn import_fit_store_track_and_hr_zones() {
    let path = fixture_path();
    if !path.exists() {
        return;
    }
    let pool = setup_test_db().await.unwrap();
    let repo = Repository::new(pool);

    let limits = SanityLimits::default();
    import::handle_import(
        ImportAction::Fit {
            path: path.to_string_lossy().to_string(),
            exercise: "Running".into(),
            workout_type: None,
            notes: None,
            force: false,
            store_track: true,
            hr_zone_bounds: Some([120.0, 140.0, 160.0, 175.0, 200.0]),
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
    let err = ImportPlan::from_activity(&act, None, None, "x.fit", None, false);
    assert!(err.is_err());
}
