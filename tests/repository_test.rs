use repslog::db::setup_test_db;
use repslog::repository::Repository;

#[tokio::test]
async fn test_exercise_management() {
    let pool = setup_test_db().await.unwrap();
    let repo = Repository::new(pool);

    let id = repo
        .add_exercise(
            "Test Pushups",
            "calisthenics",
            None,
            None,
            "body_mass",
            None,
            true,
            false,
        )
        .await
        .unwrap();
    assert!(id > 0);

    let exercises = repo
        .list_exercises(Some("Test".to_string()), None)
        .await
        .unwrap();
    assert_eq!(exercises.len(), 1);
    assert_eq!(exercises[0].name, "Test Pushups");

    let found = repo
        .find_exercise_by_id_or_name("Test Pushups")
        .await
        .unwrap();
    assert!(found.is_some());
}

#[tokio::test]
async fn test_workout_flow() {
    let pool = setup_test_db().await.unwrap();
    let repo = Repository::new(pool);

    // 1. Setup Exercise
    let ex_id = repo
        .add_exercise(
            "Bench Press",
            "strength",
            None,
            Some("barbell"),
            "external",
            None,
            false,
            false,
        )
        .await
        .unwrap();

    // 2. Create Workout
    let w_id = repo
        .create_workout(Some("Push"), Some("Morning workout"), None, false)
        .await
        .unwrap();
    assert!(w_id > 0);

    // 3. Add Exercise to Workout
    let we_id = repo
        .add_workout_exercise(w_id, ex_id, 1, Some("Warmup"), None, false)
        .await
        .unwrap();
    assert!(we_id > 0);

    // 4. Add Sets
    let s1 = repo
        .add_set(
            we_id,
            1,
            Some(10),
            Some(60.0),
            None,
            None,
            None,
            Some(8.0),
            None,
            None,
            None,
            None,
            None,
            None,
            None, // side
            None,
            None,
            None,
            None,
            None,
            false,
        )
        .await
        .unwrap();
    let s2 = repo
        .add_set(
            we_id,
            2,
            Some(10),
            Some(60.0),
            None,
            None,
            None,
            Some(8.5),
            None,
            None,
            None,
            None,
            None,
            None,
            None, // side
            None,
            None,
            None,
            None,
            None,
            false,
        )
        .await
        .unwrap();
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

    // 6. Update Workout
    repo.update_workout(w_id, None, None, Some(45), Some(4), None, false)
        .await
        .unwrap();
    let updated_w = repo.get_workout(w_id).await.unwrap().unwrap();
    assert_eq!(updated_w.duration_minutes, Some(45));
    assert_eq!(updated_w.overall_feeling, Some(4));
}

#[tokio::test]
async fn test_new_set_fields() {
    let pool = setup_test_db().await.unwrap();
    let repo = Repository::new(pool);

    let ex_id = repo
        .add_exercise(
            "New Fields Exercise",
            "strength",
            None,
            None,
            "external",
            None,
            false,
            false,
        )
        .await
        .unwrap();
    let w_id = repo.create_workout(None, None, None, false).await.unwrap();
    let we_id = repo
        .add_workout_exercise(w_id, ex_id, 1, None, None, false)
        .await
        .unwrap();

    let rir = Some(1.0);
    let effective_reps = Some(4);
    let cluster_id = Some(101i64);
    let rest_seconds = Some(90);

    let s_id = repo
        .add_set(
            we_id,
            1,
            Some(5),
            Some(100.0),
            None,
            None,
            None,
            Some(9.0),
            rir,
            effective_reps,
            cluster_id,
            rest_seconds,
            Some("Test notes"),
            None, // side
            None,
            None,
            None,
            None,
            None,
            None,
            false,
        )
        .await
        .unwrap();
    assert!(s_id > 0);

    let sets = repo.list_sets(we_id).await.unwrap();
    assert_eq!(sets.len(), 1);
    let s = &sets[0];
    assert_eq!(s.rir, rir);
    assert_eq!(s.effective_reps, effective_reps);
    assert_eq!(s.cluster_id, cluster_id);
    assert_eq!(s.rest_seconds, rest_seconds);
    assert_eq!(s.notes, Some("Test notes".to_string()));
}
