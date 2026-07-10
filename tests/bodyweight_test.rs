use repslog::cli::SetAction;
use repslog::commands::set::handle_set;
use repslog::db::setup_test_db;
use repslog::repository::Repository;

#[tokio::test]
async fn test_bodyweight_set_requires_weight() {
    let pool = setup_test_db().await.unwrap();
    let repo = Repository::new(pool);

    let ex_id = repo
        .add_exercise(
            "pull up",
            "calisthenics",
            None,
            None,
            "body_mass",
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

    let err = handle_set(
        SetAction::Add {
            workout_exercise_id: Some(we_id.to_string()),
            reps: Some(8),
            weight: None,
            external_load: None,
            no_weight_recorded: false,
            duration: None,
            distance: None,
            rpe: None,
            rir: None,
            effective_reps: None,
            rest_seconds: None,
            notes: None,
            side: None,
            phase: "full".to_string(),
            avg_heart_rate: None,
            max_heart_rate: None,
            hr_zones: None,
            pace: None,
            calories: None,
            laps: None,
            dry_run: false,
        },
        &repo,
        &repslog::app_config::SanityLimits::default(),
        false,
    )
    .await
    .unwrap_err()
    .to_string();
    assert!(err.contains("requires --weight"));
}

#[tokio::test]
async fn test_bodyweight_set_with_external_load() {
    let pool = setup_test_db().await.unwrap();
    let repo = Repository::new(pool);

    let ex_id = repo
        .add_exercise(
            "dip",
            "calisthenics",
            None,
            None,
            "body_mass",
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

    handle_set(
        SetAction::Add {
            workout_exercise_id: Some(we_id.to_string()),
            reps: Some(6),
            weight: Some(82.0),
            external_load: Some(5.0),
            no_weight_recorded: false,
            duration: None,
            distance: None,
            rpe: None,
            rir: None,
            effective_reps: None,
            rest_seconds: None,
            notes: None,
            side: None,
            phase: "full".to_string(),
            avg_heart_rate: None,
            max_heart_rate: None,
            hr_zones: None,
            pace: None,
            calories: None,
            laps: None,
            dry_run: false,
        },
        &repo,
        &repslog::app_config::SanityLimits::default(),
        false,
    )
    .await
    .unwrap();

    let sets = repo.list_sets(we_id).await.unwrap();
    assert_eq!(sets[0].weight_kg, Some(82.0));
    assert_eq!(sets[0].external_load_kg, Some(5.0));
}
