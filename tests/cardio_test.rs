use repslog::db::setup_test_db;
use repslog::models::{HeartRateZones, Lap};
use repslog::repository::Repository;
use sqlx::types::Json;

#[tokio::test]
async fn test_cardio_tracking() {
    let pool = setup_test_db().await.unwrap();
    let repo = Repository::new(pool);

    // 1. Setup Cardio Exercise
    let ex_id = repo
        .add_exercise(
            "Outdoor Run",
            "cardio",
            Some("[\"legs\"]"),
            Some("none"),
            "none",
            Some("Evening run"),
            false,
            false,
        )
        .await
        .unwrap();

    // 2. Create Workout
    let w_id = repo
        .create_workout(Some("Run"), None, None, false)
        .await
        .unwrap();

    // 3. Add Exercise to Workout
    let we_id = repo
        .add_workout_exercise(w_id, ex_id, 1, None, None, false)
        .await
        .unwrap();

    // 4. Add Cardio Set
    let avg_hr = Some(155.0);
    let max_hr = Some(182.0);
    let zones = HeartRateZones {
        z1_seconds: 100,
        z2_seconds: 1000,
        ..Default::default()
    };
    let pace = Some(5.2);
    let calories = Some(500);
    let distance = Some(5.0);
    let duration = Some(1560);
    let laps = vec![
        Lap {
            lap_number: 1,
            distance_km: 1.0,
            duration_seconds: 312,
            pace_min_per_km: 5.2,
        },
        Lap {
            lap_number: 2,
            distance_km: 1.0,
            duration_seconds: 312,
            pace_min_per_km: 5.2,
        },
    ];

    let s_id = repo
        .add_set(
            we_id,
            1,
            None,
            None,
            None,
            duration,
            distance,
            None,
            None,
            None,
            None,
            None,
            Some("Nice run"),
            None, // side
            repslog::phase::FULL,
            avg_hr,
            max_hr,
            Some(Json(zones.clone())),
            pace,
            calories,
            Some(Json(laps.clone())),
            false,
        )
        .await
        .unwrap();
    assert!(s_id > 0);

    // 5. Verify Cardio Data
    let sets = repo.list_sets(we_id).await.unwrap();
    assert_eq!(sets.len(), 1);
    let s = &sets[0];
    assert_eq!(s.avg_heart_rate_bpm, avg_hr);
    assert_eq!(s.max_heart_rate_bpm, max_hr);
    assert_eq!(
        s.heart_rate_zones.as_ref().map(|j| j.0.clone()),
        Some(zones)
    );
    assert_eq!(s.avg_pace_min_per_km, pace);
    assert_eq!(s.calories_burned, calories);
    assert_eq!(s.distance_km, distance);
    assert_eq!(s.duration_seconds, duration);
    assert_eq!(s.laps.as_ref().map(|j| j.0.clone()), Some(laps));
}
