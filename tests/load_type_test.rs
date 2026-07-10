use repslog::cli::{ExerciseAction, SetAction};
use repslog::commands::exercise::handle_exercise;
use repslog::commands::set::handle_set;
use repslog::db::setup_test_db;
use repslog::repository::Repository;

#[tokio::test]
async fn test_ring_dip_with_rings_equipment_requires_body_mass() {
    let pool = setup_test_db().await.unwrap();
    let repo = Repository::new(pool);

    handle_exercise(
        ExerciseAction::Add {
            name: "ring dip".to_string(),
            category: "calisthenics".to_string(),
            equipment: Some("rings".to_string()),
            load_type: Some("body_mass".to_string()),
            muscles: None,
            description: None,
            allow_phase_in_name: false,
            dry_run: false,
        },
        &repo,
        false,
    )
    .await
    .unwrap();

    let ex = repo
        .find_exercise_by_id_or_name("ring dip")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(ex.equipment.as_deref(), Some("rings"));
    assert_eq!(ex.load_type, "body_mass");

    let w_id = repo.create_workout(None, None, None, false).await.unwrap();
    let we_id = repo
        .add_workout_exercise(w_id, ex.id, 1, None, None, false)
        .await
        .unwrap();

    let err = handle_set(
        SetAction::Add {
            workout_exercise_id: Some(we_id.to_string()),
            reps: Some(5),
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
async fn test_exercise_update_changes_load_type() {
    let pool = setup_test_db().await.unwrap();
    let repo = Repository::new(pool);

    let ex_id = repo
        .add_exercise(
            "dip",
            "calisthenics",
            None,
            Some("rings"),
            "external",
            None,
            false,
            false,
        )
        .await
        .unwrap();

    handle_exercise(
        ExerciseAction::Update {
            exercise_id_or_name: ex_id.to_string(),
            category: None,
            equipment: None,
            clear_equipment: false,
            load_type: Some("body_mass".to_string()),
            muscles: None,
            description: None,
            dry_run: false,
        },
        &repo,
        false,
    )
    .await
    .unwrap();

    let ex = repo
        .find_exercise_by_id_or_name("dip")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(ex.load_type, "body_mass");
}
