use repslog::commands::exercise::handle_exercise;
use repslog::commands::init::handle_init;
use repslog::db::setup_test_db;
use repslog::repository::Repository;
use repslog::{cli::ExerciseAction, error::RepslogError};

#[tokio::test]
async fn test_exercise_add_rejects_uppercase() {
    let pool = setup_test_db().await.unwrap();
    handle_init(&pool, false, false).await.unwrap();
    let repo = Repository::new(pool);

    let err = handle_exercise(
        ExerciseAction::Add {
            name: "Pull Ups".to_string(),
            category: "calisthenics".to_string(),
            equipment: None,
            muscles: None,
            description: None,
            dry_run: false,
        },
        &repo,
        false,
    )
    .await
    .unwrap_err();

    assert!(matches!(err, RepslogError::Cli(_)));
    assert!(err.to_string().contains("lowercase"));
}

#[tokio::test]
async fn test_exercise_add_rejects_near_duplicate_of_seeded() {
    let pool = setup_test_db().await.unwrap();
    handle_init(&pool, false, false).await.unwrap();
    let repo = Repository::new(pool);

    let err = handle_exercise(
        ExerciseAction::Add {
            name: "pull ups".to_string(),
            category: "calisthenics".to_string(),
            equipment: None,
            muscles: None,
            description: None,
            dry_run: false,
        },
        &repo,
        false,
    )
    .await
    .unwrap_err();

    assert!(err.to_string().contains("already exists"));
    assert!(err.to_string().contains("Pullups"));
}

#[tokio::test]
async fn test_exercise_add_warns_on_similar_name() {
    let pool = setup_test_db().await.unwrap();
    handle_init(&pool, false, false).await.unwrap();
    let repo = Repository::new(pool.clone());

    repo.add_exercise(
        "nordic hamstring curl",
        "strength",
        None,
        None,
        None,
        true,
        false,
    )
    .await
    .unwrap();

    handle_exercise(
        ExerciseAction::Add {
            name: "nordic curl".to_string(),
            category: "strength".to_string(),
            equipment: None,
            muscles: None,
            description: None,
            dry_run: false,
        },
        &repo,
        false,
    )
    .await
    .unwrap();

    let exercises = repo.list_exercises(None, None).await.unwrap();
    assert!(exercises.iter().any(|e| e.name == "nordic curl"));
}

#[tokio::test]
async fn test_exercise_add_warns_on_plural_name() {
    let pool = setup_test_db().await.unwrap();
    handle_init(&pool, false, false).await.unwrap();
    let repo = Repository::new(pool);

    handle_exercise(
        ExerciseAction::Add {
            name: "ring dips".to_string(),
            category: "calisthenics".to_string(),
            equipment: None,
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
        .find_exercise_by_id_or_name("ring dips")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(ex.name, "ring dips");
}

#[tokio::test]
async fn test_exercise_add_stores_normalized_lowercase() {
    let pool = setup_test_db().await.unwrap();
    handle_init(&pool, false, false).await.unwrap();
    let repo = Repository::new(pool);

    handle_exercise(
        ExerciseAction::Add {
            name: "  wall   walk  ".to_string(),
            category: "calisthenics".to_string(),
            equipment: None,
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
        .find_exercise_by_id_or_name("wall walk")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(ex.name, "wall walk");
}
