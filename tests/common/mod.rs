use repslog::repository::Repository;

/// Minimal strength set for integration tests (phase defaults to full).
#[allow(clippy::too_many_arguments)]
pub async fn add_strength_set(
    repo: &Repository,
    workout_exercise_id: i64,
    set_number: i32,
    reps: Option<i32>,
    weight: Option<f64>,
    rpe: Option<f64>,
    rir: Option<f64>,
    effective_reps: Option<i32>,
    cluster_id: Option<i64>,
    rest_seconds: Option<i32>,
    notes: Option<&str>,
    side: Option<&str>,
    phase: &str,
) -> i64 {
    repo.add_set(
        workout_exercise_id,
        set_number,
        reps,
        weight,
        None,
        None,
        None,
        rpe,
        rir,
        effective_reps,
        cluster_id,
        rest_seconds,
        notes,
        side,
        phase,
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
    .unwrap()
}
