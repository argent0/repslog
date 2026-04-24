use repslog::repository::Repository;
use repslog::db::setup_test_db;
use sqlx::Row;

#[tokio::test]
async fn test_stats_prs() {
    let pool = setup_test_db().await.unwrap();
    let repo = Repository::new(pool);

    let ex_id = repo.add_exercise("Squat", "strength", None, Some("barbell"), None, false, false).await.unwrap();
    let w_id = repo.create_workout(Some("Legs"), None, None, false).await.unwrap();
    let we_id = repo.add_workout_exercise(w_id, ex_id, 1, None, false).await.unwrap();

    // Max weight set
    repo.add_set(we_id, 1, Some(10), Some(100.0), None, None, Some(9.0), None, None, None, None, None, None, None, None, None, None, None, false).await.unwrap();
    // Max reps set (different weight)
    repo.add_set(we_id, 2, Some(15), Some(80.0), None, None, Some(8.0), None, None, None, None, None, None, None, None, None, None, None, false).await.unwrap();

    let query = "SELECT e.name, MAX(es.weight_kg) as max_weight, MAX(es.reps) as max_reps FROM exercise_sets es JOIN workout_exercises we ON es.workout_exercise_id = we.id JOIN exercises e ON we.exercise_id = e.id GROUP BY e.name";
    let res = sqlx::query(&query).fetch_one(&repo.pool).await.unwrap();

    assert_eq!(res.get::<String, _>("name"), "Squat");
    assert_eq!(res.get::<f64, _>("max_weight"), 100.0);
    assert_eq!(res.get::<i32, _>("max_reps"), 15);
}

#[tokio::test]
async fn test_stats_volume() {
    let pool = setup_test_db().await.unwrap();
    let repo = Repository::new(pool);

    let ex_id = repo.add_exercise("Curl", "strength", None, Some("dumbbell"), None, false, false).await.unwrap();
    let w_id = repo.create_workout(Some("Arms"), None, None, false).await.unwrap();
    let we_id = repo.add_workout_exercise(w_id, ex_id, 1, None, false).await.unwrap();

    // Volume: 10 * 10.0 = 100.0
    repo.add_set(we_id, 1, Some(10), Some(10.0), None, None, None, None, None, None, None, None, None, None, None, None, None, None, false).await.unwrap();
    // Volume: 12 * 10.0 = 120.0
    repo.add_set(we_id, 2, Some(12), Some(10.0), None, None, None, None, None, None, None, None, None, None, None, None, None, None, false).await.unwrap();

    let query = "SELECT e.name, SUM(es.weight_kg * es.reps) as total_volume FROM exercise_sets es JOIN workout_exercises we ON es.workout_exercise_id = we.id JOIN exercises e ON we.exercise_id = e.id GROUP BY e.name";
    let res = sqlx::query(&query).fetch_one(&repo.pool).await.unwrap();

    assert_eq!(res.get::<String, _>("name"), "Curl");
    assert_eq!(res.get::<f64, _>("total_volume"), 220.0);
}
