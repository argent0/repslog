use repslog::repository::Repository;
use repslog::db::setup_test_db;

#[tokio::test]
async fn test_cardio_tracking() {
    let pool = setup_test_db().await.unwrap();
    let repo = Repository::new(pool);

    // 1. Setup Cardio Exercise
    let ex_id = repo.add_exercise("Outdoor Run", "cardio", Some("[\"legs\"]"), Some("none"), Some("Evening run"), false).await.unwrap();

    // 2. Create Workout
    let w_id = repo.create_workout(Some("Run"), None, None).await.unwrap();

    // 3. Add Exercise to Workout
    let we_id = repo.add_workout_exercise(w_id, ex_id, 1, None).await.unwrap();

    // 4. Add Cardio Set
    let avg_hr = Some(155.0);
    let max_hr = Some(182.0);
    let zones = Some("{\"z1\": 100, \"z2\": 1000}".to_string());
    let pace = Some(5.2);
    let calories = Some(500);
    let distance = Some(5.0);
    let duration = Some(1560);

    let s_id = repo.add_set(
        we_id, 1, None, None, duration, distance, None, None, None, None, None, Some("Nice run"),
        avg_hr, max_hr, zones.clone(), pace, calories
    ).await.unwrap();
    assert!(s_id > 0);

    // 5. Verify Cardio Data
    let sets = repo.list_sets(we_id).await.unwrap();
    assert_eq!(sets.len(), 1);
    let s = &sets[0];
    assert_eq!(s.avg_heart_rate_bpm, avg_hr);
    assert_eq!(s.max_heart_rate_bpm, max_hr);
    assert_eq!(s.heart_rate_zones, zones);
    assert_eq!(s.avg_pace_min_per_km, pace);
    assert_eq!(s.calories_burned, calories);
    assert_eq!(s.distance_km, distance);
    assert_eq!(s.duration_seconds, duration);
}
