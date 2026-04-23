use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct Exercise {
    pub id: i64,
    pub name: String,
    pub category: String,
    pub muscle_groups: Option<String>,
    pub equipment: Option<String>,
    pub description: Option<String>,
    pub is_custom: i32,
    pub created_at: Option<String>,
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct Workout {
    pub id: i64,
    pub started_at: String,
    pub finished_at: Option<String>,
    pub workout_type: Option<String>,
    pub notes: Option<String>,
    pub overall_feeling: Option<i32>,
    pub duration_minutes: Option<i32>,
    pub created_at: Option<String>,
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct WorkoutExercise {
    pub id: i64,
    pub workout_id: i64,
    pub exercise_id: i64,
    pub order: i32,
    pub notes: Option<String>,
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct ExerciseSet {
    pub id: i64,
    pub workout_exercise_id: i64,
    pub set_number: i32,
    pub reps: Option<i32>,
    pub weight_kg: Option<f64>,
    pub distance_km: Option<f64>,
    pub duration_seconds: Option<i32>,
    pub rpe: Option<f64>,
    pub rest_seconds: Option<i32>,
    pub notes: Option<String>,
    pub extra_metrics: Option<String>,
    pub created_at: Option<String>,
}
