use repslog::db::setup_test_db;
use repslog::repository::Repository;

#[tokio::test]
async fn test_set_numbering() {
    let pool = setup_test_db().await.unwrap();
    let repo = Repository::new(pool);

    let ex_id = repo
        .add_exercise("Ex1", "cat", None, None, None, false, false)
        .await
        .unwrap();
    let w_id = repo.create_workout(None, None, None, false).await.unwrap();
    let we_id = repo
        .add_workout_exercise(w_id, ex_id, 1, None, None, false)
        .await
        .unwrap();

    let set1_num = repo.get_next_set_number(we_id).await.unwrap();
    assert_eq!(set1_num, 1);
    repo.add_set(
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
        None,
        None,
        None,
        None,
        None, // side
        None,
        None,
        None,
        false,
    )
    .await
    .unwrap();

    let set2_num = repo.get_next_set_number(we_id).await.unwrap();
    assert_eq!(set2_num, 2);
    repo.add_set(
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
        None,
        None,
        None,
        None, // side
        None,
        None,
        None,
        None,
        false,
    )
    .await
    .unwrap();
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
            Some("bodyweight"),
            None,
            false,
            false,
        )
        .await
        .unwrap();
    let w_id = repo.create_workout(None, None, None, false).await.unwrap();

    // Replicating Quick logic from handle_set
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
    repo.add_set(
        we_id, 1, None, None, None, None, None, None, None, None, None, None, None,
        None, // side
        None, None, None, None, None, false,
    )
    .await
    .unwrap();

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

    // Add right first, then left — list_sets should return left before right due to side-aware ORDER BY
    let _r1 = repo
        .add_set(
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
            None,
            None,
            Some("right"),
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
    let l1 = repo
        .add_set(
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
            None,
            None,
            Some("left"),
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
        .add_exercise("Split Squat", "strength", None, None, None, false, false)
        .await
        .unwrap();
    let w_id = repo.create_workout(None, None, None, false).await.unwrap();
    let we_id = repo
        .add_workout_exercise(w_id, ex_id, 1, None, None, false)
        .await
        .unwrap();

    let sid = repo
        .add_set(
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
            None,
            None,
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

    // Weight-only should now be accepted at handler level; here we test repo + update
    repo.update_set(
        sid,
        None,
        Some(20.0),
        None,
        None,
        None,
        None,
        None,
        None,
        Some("Left leg only"),
        Some("left"),
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
async fn test_set_delete_and_reorder() {
    let pool = setup_test_db().await.unwrap();
    let repo = Repository::new(pool);

    let ex_id = repo
        .add_exercise("Lunge", "strength", None, None, None, false, false)
        .await
        .unwrap();
    let w_id = repo.create_workout(None, None, None, false).await.unwrap();
    let we_id = repo
        .add_workout_exercise(w_id, ex_id, 1, None, None, false)
        .await
        .unwrap();

    let s1 = repo
        .add_set(
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
            None,
            None,
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
    let s2 = repo
        .add_set(
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
            None,
            None,
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
    let s3 = repo
        .add_set(
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
            None,
            None,
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

    // Delete middle
    repo.delete_set(s2, false).await.unwrap();
    let remaining = repo.list_sets(we_id).await.unwrap();
    assert_eq!(remaining.len(), 2);

    // Reorder: move last (s3) to position 1 -> should become set_number 1, old s1 becomes 2
    repo.reorder_set(s3, 1, false).await.unwrap();
    let after = repo.list_sets(we_id).await.unwrap();
    assert_eq!(after[0].id, s3);
    assert_eq!(after[0].set_number, 1);
    assert_eq!(after[1].id, s1);
    assert_eq!(after[1].set_number, 2);
}
