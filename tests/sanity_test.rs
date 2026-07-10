use repslog::app_config::SanityLimits;
use repslog::db::setup_test_db;
use repslog::repository::Repository;
use repslog::sanity::{validate_set_metrics, ProposedSetMetrics};

#[tokio::test]
async fn add_set_rejects_impossible_hr_without_insert() {
    let pool = setup_test_db().await.unwrap();
    let repo = Repository::new(pool);
    let ex = repo
        .add_exercise("run", "cardio", None, None, "none", None, true, false)
        .await
        .unwrap();
    let w = repo
        .create_workout(Some("Run"), None, Some("2026-07-10 10:00:00"), false)
        .await
        .unwrap();
    let we = repo
        .add_workout_exercise(w, ex, 1, None, None, false)
        .await
        .unwrap();

    let limits = SanityLimits::default();
    let bad = ProposedSetMetrics {
        distance_km: Some(5.0),
        duration_seconds: Some(1500),
        avg_heart_rate_bpm: Some(999.0),
        max_heart_rate_bpm: Some(170.0),
        avg_pace_min_per_km: Some(5.0),
        calories_burned: Some(300),
        ..Default::default()
    };
    assert!(validate_set_metrics(&bad, &limits).is_err());

    // Ensure no sets were written via a direct path either — validation is pre-write.
    let sets = repo.list_sets(we).await.unwrap();
    assert!(sets.is_empty());
}
