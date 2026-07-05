use repslog::db::setup_test_db;
use repslog::repository::Repository;
use sqlx::Row;

#[tokio::test]
async fn test_stats_prs() {
    let pool = setup_test_db().await.unwrap();
    let repo = Repository::new(pool);

    let ex_id = repo
        .add_exercise(
            "Squat",
            "strength",
            None,
            Some("barbell"),
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

    // Max weight set
    repo.add_set(
        we_id,
        1,
        Some(10),
        Some(100.0),
        None,
        None,
        None,
        Some(9.0),
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
        None,
        false,
    )
    .await
    .unwrap();
    // Max reps set (different weight)
    repo.add_set(
        we_id,
        2,
        Some(15),
        Some(80.0),
        None,
        None,
        None,
        Some(8.0),
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
        None,
        false,
    )
    .await
    .unwrap();

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

    let ex_id = repo
        .add_exercise(
            "Curl",
            "strength",
            None,
            Some("dumbbell"),
            None,
            false,
            false,
        )
        .await
        .unwrap();
    let w_id = repo
        .create_workout(Some("Arms"), None, None, false)
        .await
        .unwrap();
    let we_id = repo
        .add_workout_exercise(w_id, ex_id, 1, None, None, false)
        .await
        .unwrap();

    // Volume: 10 * 10.0 = 100.0
    repo.add_set(
        we_id,
        1,
        Some(10),
        Some(10.0),
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
        None,
        None,
        None,
        false,
    )
    .await
    .unwrap();
    // Volume: 12 * 10.0 = 120.0
    repo.add_set(
        we_id,
        2,
        Some(12),
        Some(10.0),
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
        None,
        None,
        None,
        false,
    )
    .await
    .unwrap();

    let query = "SELECT e.name, SUM(es.weight_kg * es.reps) as total_volume FROM exercise_sets es JOIN workout_exercises we ON es.workout_exercise_id = we.id JOIN exercises e ON we.exercise_id = e.id GROUP BY e.name";
    let res = sqlx::query(&query).fetch_one(&repo.pool).await.unwrap();

    assert_eq!(res.get::<String, _>("name"), "Curl");
    assert_eq!(res.get::<f64, _>("total_volume"), 220.0);
}

#[tokio::test]
async fn test_stats_volume_with_null_weight_returns_real() {
    let pool = setup_test_db().await.unwrap();
    let repo = Repository::new(pool);

    let ex_id = repo
        .add_exercise("push up", "calisthenics", None, None, None, false, false)
        .await
        .unwrap();
    let w_id = repo
        .create_workout(Some("Calisthenics"), None, None, false)
        .await
        .unwrap();
    let we_id = repo
        .add_workout_exercise(w_id, ex_id, 1, None, None, false)
        .await
        .unwrap();

    repo.add_set(
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
        None,
        false,
    )
    .await
    .unwrap();

    let query = "SELECT e.name, \
        SUM(CASE \
            WHEN es.weight_kg IS NULL THEN 0.0 \
            WHEN e.equipment = 'bodyweight' THEN (es.weight_kg + COALESCE(es.external_load_kg, 0)) * es.reps \
            ELSE es.weight_kg * es.reps \
        END) as total_volume \
        FROM exercise_sets es \
        JOIN workout_exercises we ON es.workout_exercise_id = we.id \
        JOIN exercises e ON we.exercise_id = e.id \
        GROUP BY e.name";
    let res = sqlx::query(&query).fetch_one(&repo.pool).await.unwrap();

    assert_eq!(res.get::<String, _>("name"), "push up");
    assert_eq!(res.get::<f64, _>("total_volume"), 0.0);
}

#[tokio::test]
async fn test_stats_history_lists_each_set_in_date_range() {
    let pool = setup_test_db().await.unwrap();
    let repo = Repository::new(pool);

    let ex_id = repo
        .add_exercise("push up", "calisthenics", None, None, None, false, false)
        .await
        .unwrap();

    let recent_w = repo
        .create_workout(
            Some("Calisthenics"),
            None,
            Some("2026-07-01 10:00:00"),
            false,
        )
        .await
        .unwrap();
    let recent_we = repo
        .add_workout_exercise(recent_w, ex_id, 1, None, None, false)
        .await
        .unwrap();

    repo.add_set(
        recent_we,
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
        None,
        false,
    )
    .await
    .unwrap();
    repo.add_set(
        recent_we,
        2,
        Some(8),
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
        None,
        false,
    )
    .await
    .unwrap();

    let old_w = repo
        .create_workout(
            Some("Calisthenics"),
            None,
            Some("2025-01-01 10:00:00"),
            false,
        )
        .await
        .unwrap();
    let old_we = repo
        .add_workout_exercise(old_w, ex_id, 1, None, None, false)
        .await
        .unwrap();
    repo.add_set(
        old_we,
        1,
        Some(20),
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
        None,
        false,
    )
    .await
    .unwrap();

    let query = "SELECT w.id AS workout_id, es.set_number, es.reps \
                  FROM exercise_sets es \
                  JOIN workout_exercises we ON es.workout_exercise_id = we.id \
                  JOIN exercises e ON we.exercise_id = e.id \
                  JOIN workouts w ON we.workout_id = w.id \
                  WHERE e.name LIKE ? AND w.started_at >= date('now', ?) \
                  ORDER BY w.started_at ASC, es.set_number ASC";
    let like = "%push up%";
    let days_ago = "-30 days";
    let res = sqlx::query(query)
        .bind(like)
        .bind(days_ago)
        .fetch_all(&repo.pool)
        .await
        .unwrap();

    assert_eq!(res.len(), 2);
    assert_eq!(res[0].get::<i64, _>("workout_id"), recent_w);
    assert_eq!(res[0].get::<i32, _>("set_number"), 1);
    assert_eq!(res[0].get::<i32, _>("reps"), 10);
    assert_eq!(res[1].get::<i32, _>("set_number"), 2);
    assert_eq!(res[1].get::<i32, _>("reps"), 8);
}
