use repslog::db::setup_test_db;
use repslog::repository::Repository;

#[tokio::test]
async fn test_update_workout() {
    let pool = setup_test_db().await.unwrap();
    let repo = Repository::new(pool);

    // Create a workout
    let w_id = repo
        .create_workout(Some("Push"), None, None, false)
        .await
        .unwrap();

    // Update it
    repo.update_workout(
        w_id,
        Some("Pull"),
        Some("Feeling good"),
        Some(45),
        Some(4),
        Some("2023-01-01 00:00:00"),
        false,
    )
    .await
    .unwrap();

    let workout = repo.get_workout(w_id).await.unwrap().unwrap();
    assert_eq!(workout.workout_type, Some("Pull".to_string()));
    assert_eq!(workout.notes, Some("Feeling good".to_string()));
    assert_eq!(workout.duration_minutes, Some(45));
    assert_eq!(workout.overall_feeling, Some(4));
    assert_eq!(workout.started_at, "2023-01-01 00:00:00");
}

#[tokio::test]
async fn test_workout_with_custom_date() {
    let pool = setup_test_db().await.unwrap();
    let repo = Repository::new(pool);

    let custom_date = "2023-01-01 10:00:00";
    let w_id = repo
        .create_workout(Some("Holiday"), Some("New Year"), Some(custom_date), false)
        .await
        .unwrap();

    let workout = repo.get_workout(w_id).await.unwrap().unwrap();
    assert_eq!(workout.started_at, custom_date);
    assert_eq!(workout.workout_type, Some("Holiday".to_string()));
}

#[tokio::test]
async fn test_workout_exercise_order() {
    let pool = setup_test_db().await.unwrap();
    let repo = Repository::new(pool);

    let ex1 = repo
        .add_exercise("Ex1", "cat", None, None, "external", None, false, false)
        .await
        .unwrap();
    let ex2 = repo
        .add_exercise("Ex2", "cat", None, None, "external", None, false, false)
        .await
        .unwrap();
    let w_id = repo.create_workout(None, None, None, false).await.unwrap();

    let order1 = repo.get_max_order_for_workout(w_id).await.unwrap() + 1;
    repo.add_workout_exercise(w_id, ex1, order1, None, None, false)
        .await
        .unwrap();
    assert_eq!(order1, 1);

    let order2 = repo.get_max_order_for_workout(w_id).await.unwrap() + 1;
    repo.add_workout_exercise(w_id, ex2, order2, None, None, false)
        .await
        .unwrap();
    assert_eq!(order2, 2);
}
