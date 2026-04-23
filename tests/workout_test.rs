use repslog::repository::Repository;
use repslog::db::setup_test_db;

#[tokio::test]
async fn test_current_workout() {
    let pool = setup_test_db().await.unwrap();
    let repo = Repository::new(pool);

    // No workout yet
    let current = repo.get_current_workout().await.unwrap();
    assert!(current.is_none());

    // Create a workout
    repo.create_workout(Some("Push"), None, None).await.unwrap();
    let current = repo.get_current_workout().await.unwrap();
    assert!(current.is_some());
    assert_eq!(current.unwrap().workout_type, Some("Push".to_string()));

    // Finish it
    repo.finish_workout(1, Some(60), Some(5)).await.unwrap();
    let current = repo.get_current_workout().await.unwrap();
    assert!(current.is_none());
}

#[tokio::test]
async fn test_workout_with_custom_date() {
    let pool = setup_test_db().await.unwrap();
    let repo = Repository::new(pool);

    let custom_date = "2023-01-01 10:00:00";
    let w_id = repo.create_workout(Some("Holiday"), Some("New Year"), Some(custom_date)).await.unwrap();
    
    let workout = repo.get_workout(w_id).await.unwrap().unwrap();
    assert_eq!(workout.started_at, custom_date);
    assert_eq!(workout.workout_type, Some("Holiday".to_string()));
}

#[tokio::test]
async fn test_workout_exercise_order() {
    let pool = setup_test_db().await.unwrap();
    let repo = Repository::new(pool);

    let ex1 = repo.add_exercise("Ex1", "cat", None, None, None, false).await.unwrap();
    let ex2 = repo.add_exercise("Ex2", "cat", None, None, None, false).await.unwrap();
    let w_id = repo.create_workout(None, None, None).await.unwrap();

    let order1 = repo.get_max_order_for_workout(w_id).await.unwrap() + 1;
    repo.add_workout_exercise(w_id, ex1, order1, None).await.unwrap();
    assert_eq!(order1, 1);

    let order2 = repo.get_max_order_for_workout(w_id).await.unwrap() + 1;
    repo.add_workout_exercise(w_id, ex2, order2, None).await.unwrap();
    assert_eq!(order2, 2);
}
