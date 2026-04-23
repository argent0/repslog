use repslog::repository::Repository;
use repslog::db::setup_test_db;

#[tokio::test]
async fn test_set_numbering() {
    let pool = setup_test_db().await.unwrap();
    let repo = Repository::new(pool);

    let ex_id = repo.add_exercise("Ex1", "cat", None, None, None, false).await.unwrap();
    let w_id = repo.create_workout(None, None, None).await.unwrap();
    let we_id = repo.add_workout_exercise(w_id, ex_id, 1, None).await.unwrap();

    let set1_num = repo.get_next_set_number(we_id).await.unwrap();
    assert_eq!(set1_num, 1);
    repo.add_set(we_id, set1_num, Some(10), None, None, None, None, None, None, None, None, None, None, None, None, None, None, None).await.unwrap();

    let set2_num = repo.get_next_set_number(we_id).await.unwrap();
    assert_eq!(set2_num, 2);
    repo.add_set(we_id, set2_num, Some(12), None, None, None, None, None, None, None, None, None, None, None, None, None, None, None).await.unwrap();
}

#[tokio::test]
async fn test_set_quick_logic() {
    let pool = setup_test_db().await.unwrap();
    let repo = Repository::new(pool);

    let ex_id = repo.add_exercise("Pullups", "calisthenics", None, Some("bodyweight"), None, false).await.unwrap();
    let w_id = repo.create_workout(None, None, None).await.unwrap();

    // Replicating Quick logic from handle_set
    let exercise = repo.find_exercise_by_id_or_name("Pullups").await.unwrap().unwrap();
    let order = repo.get_max_order_for_workout(w_id).await.unwrap() + 1;
    let we_id = repo.add_workout_exercise(w_id, exercise.id, order, None).await.unwrap();
    repo.add_set(we_id, 1, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None).await.unwrap();

    let sets = repo.list_sets(we_id).await.unwrap();
    assert_eq!(sets.len(), 1);
    assert_eq!(sets[0].set_number, 1);
}
