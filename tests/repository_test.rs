use repslog::repository::Repository;
use repslog::db::setup_test_db;

#[tokio::test]
async fn test_exercise_management() {
    let pool = setup_test_db().await.unwrap();
    let repo = Repository::new(pool);

    let id = repo.add_exercise("Test Pushups", "calisthenics", None, None, None, true).await.unwrap();
    assert!(id > 0);

    let exercises = repo.list_exercises(Some("Test".to_string()), None).await.unwrap();
    assert_eq!(exercises.len(), 1);
    assert_eq!(exercises[0].name, "Test Pushups");

    let found = repo.find_exercise_by_id_or_name("Test Pushups").await.unwrap();
    assert!(found.is_some());
}

#[tokio::test]
async fn test_workout_flow() {
    let pool = setup_test_db().await.unwrap();
    let repo = Repository::new(pool);

    // 1. Setup Exercise
    let ex_id = repo.add_exercise("Bench Press", "strength", None, Some("barbell"), None, false).await.unwrap();

    // 2. Create Workout
    let w_id = repo.create_workout(Some("Push"), Some("Morning workout"), None).await.unwrap();
    assert!(w_id > 0);

    // 3. Add Exercise to Workout
    let we_id = repo.add_workout_exercise(w_id, ex_id, 1, Some("Warmup")).await.unwrap();
    assert!(we_id > 0);

    // 4. Add Sets
    let s1 = repo.add_set(we_id, 1, Some(10), Some(60.0), None, None, Some(8.0), None).await.unwrap();
    let s2 = repo.add_set(we_id, 2, Some(10), Some(60.0), None, None, Some(8.5), None).await.unwrap();
    assert!(s1 > 0);
    assert!(s2 > 0);

    // 5. Verify Workout View
    let workout = repo.get_workout(w_id).await.unwrap().unwrap();
    assert_eq!(workout.workout_type, Some("Push".to_string()));

    let we_list = repo.list_workout_exercises(w_id).await.unwrap();
    assert_eq!(we_list.len(), 1);
    assert_eq!(we_list[0].1, "Bench Press");

    let sets = repo.list_sets(we_id).await.unwrap();
    assert_eq!(sets.len(), 2);
    assert_eq!(sets[0].weight_kg, Some(60.0));

    // 6. Finish Workout
    repo.finish_workout(w_id, Some(45), Some(4)).await.unwrap();
    let finished_w = repo.get_workout(w_id).await.unwrap().unwrap();
    assert!(finished_w.finished_at.is_some());
    assert_eq!(finished_w.duration_minutes, Some(45));
}
