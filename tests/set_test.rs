mod common;

use common::add_strength_set;
use repslog::db::setup_test_db;
use repslog::repository::Repository;

#[tokio::test]
async fn test_set_numbering() {
    let pool = setup_test_db().await.unwrap();
    let repo = Repository::new(pool);

    let ex_id = repo
        .add_exercise("Ex1", "cat", None, None, "external", None, false, false)
        .await
        .unwrap();
    let w_id = repo.create_workout(None, None, None, false).await.unwrap();
    let we_id = repo
        .add_workout_exercise(w_id, ex_id, 1, None, None, false)
        .await
        .unwrap();

    let set1_num = repo.get_next_set_number(we_id).await.unwrap();
    assert_eq!(set1_num, 1);
    add_strength_set(
        &repo,
        we_id,
        set1_num,
        Some(10),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        repslog::phase::FULL,
    )
    .await;

    let set2_num = repo.get_next_set_number(we_id).await.unwrap();
    assert_eq!(set2_num, 2);
    add_strength_set(
        &repo,
        we_id,
        set2_num,
        Some(12),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        repslog::phase::FULL,
    )
    .await;
}

#[tokio::test]
async fn test_set_quick_logic() {
    let pool = setup_test_db().await.unwrap();
    let repo = Repository::new(pool);

    let _ex_id = repo
        .add_exercise(
            "Pullups",
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

    let exercise = repo
        .find_exercise_by_id_or_name("Pullups")
        .await
        .unwrap()
        .unwrap();
    let order = repo.get_max_order_for_workout(w_id).await.unwrap() + 1;
    let we_id = repo
        .add_workout_exercise(w_id, exercise.id, order, None, None, false)
        .await
        .unwrap();
    add_strength_set(
        &repo,
        we_id,
        1,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        repslog::phase::FULL,
    )
    .await;

    let sets = repo.list_sets(we_id).await.unwrap();
    assert_eq!(sets.len(), 1);
    assert_eq!(sets[0].set_number, 1);
}

#[tokio::test]
async fn test_side_and_unilateral_ordering() {
    let pool = setup_test_db().await.unwrap();
    let repo = Repository::new(pool);

    let ex_id = repo
        .add_exercise(
            "Bulgarian Split Squat",
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
    let w_id = repo
        .create_workout(Some("Legs"), None, None, false)
        .await
        .unwrap();
    let we_id = repo
        .add_workout_exercise(w_id, ex_id, 1, None, None, false)
        .await
        .unwrap();

    let _r1 = add_strength_set(
        &repo,
        we_id,
        1,
        Some(8),
        Some(20.0),
        None,
        None,
        None,
        None,
        None,
        None,
        Some("right"),
        repslog::phase::FULL,
    )
    .await;
    let l1 = add_strength_set(
        &repo,
        we_id,
        2,
        Some(8),
        Some(20.0),
        None,
        None,
        None,
        None,
        None,
        None,
        Some("left"),
        repslog::phase::FULL,
    )
    .await;

    let sets = repo.list_sets(we_id).await.unwrap();
    assert_eq!(sets.len(), 2);
    assert_eq!(sets[0].side, Some("left".to_string()));
    assert_eq!(sets[0].id, l1);
    assert_eq!(sets[1].side, Some("right".to_string()));
}

#[tokio::test]
async fn test_set_update_and_weight_only() {
    let pool = setup_test_db().await.unwrap();
    let repo = Repository::new(pool);

    let ex_id = repo
        .add_exercise(
            "Split Squat",
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

    let sid = add_strength_set(
        &repo,
        we_id,
        1,
        Some(6),
        Some(18.0),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        repslog::phase::FULL,
    )
    .await;

    repo.update_set(
        sid,
        None,
        Some(20.0),
        false,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some("Left leg only"),
        Some("left"),
        None,
        false,
    )
    .await
    .unwrap();

    let s = repo.get_set(sid).await.unwrap().unwrap();
    assert_eq!(s.weight_kg, Some(20.0));
    assert_eq!(s.side, Some("left".to_string()));
    assert_eq!(s.notes, Some("Left leg only".to_string()));
}

#[tokio::test]
async fn test_phase_eccentric_set() {
    let pool = setup_test_db().await.unwrap();
    let repo = Repository::new(pool);

    let ex_id = repo
        .add_exercise(
            "Pistol Squat",
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

    let sid = add_strength_set(
        &repo,
        we_id,
        1,
        Some(3),
        Some(82.0),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        repslog::phase::ECCENTRIC,
    )
    .await;

    let s = repo.get_set(sid).await.unwrap().unwrap();
    assert_eq!(s.phase, repslog::phase::ECCENTRIC);
}

#[tokio::test]
async fn test_set_delete_and_reorder() {
    let pool = setup_test_db().await.unwrap();
    let repo = Repository::new(pool);

    let ex_id = repo
        .add_exercise(
            "Lunge", "strength", None, None, "external", None, false, false,
        )
        .await
        .unwrap();
    let w_id = repo.create_workout(None, None, None, false).await.unwrap();
    let we_id = repo
        .add_workout_exercise(w_id, ex_id, 1, None, None, false)
        .await
        .unwrap();

    let s1 = add_strength_set(
        &repo,
        we_id,
        1,
        Some(10),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        repslog::phase::FULL,
    )
    .await;
    let s2 = add_strength_set(
        &repo,
        we_id,
        2,
        Some(10),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        repslog::phase::FULL,
    )
    .await;
    let s3 = add_strength_set(
        &repo,
        we_id,
        3,
        Some(10),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        repslog::phase::FULL,
    )
    .await;

    repo.delete_set(s2, false).await.unwrap();
    let remaining = repo.list_sets(we_id).await.unwrap();
    assert_eq!(remaining.len(), 2);

    repo.reorder_set(s3, 1, false).await.unwrap();
    let after = repo.list_sets(we_id).await.unwrap();
    assert_eq!(after[0].id, s3);
    assert_eq!(after[0].set_number, 1);
    assert_eq!(after[1].id, s1);
    assert_eq!(after[1].set_number, 2);
}
