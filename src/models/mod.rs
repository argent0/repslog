use serde::{Deserialize, Serialize};
use sqlx::types::Json;
use sqlx::FromRow;

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct Exercise {
    pub id: i64,
    pub name: String,
    pub category: String,
    pub muscle_groups: Option<String>,
    pub equipment: Option<String>,
    pub load_type: String,
    pub description: Option<String>,
    pub is_custom: i32,
    pub created_at: Option<String>,
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct Workout {
    pub id: i64,
    pub started_at: String,
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
    pub goal_reps: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct HeartRateZones {
    #[serde(default)]
    pub z1_seconds: u32,
    #[serde(default)]
    pub z2_seconds: u32,
    #[serde(default)]
    pub z3_seconds: u32,
    #[serde(default)]
    pub z4_seconds: u32,
    #[serde(default)]
    pub z5_seconds: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Lap {
    pub lap_number: u16,       // 1-based
    pub distance_km: f64,      // e.g. 1.0 or 0.98 for last lap
    pub duration_seconds: u32, // exact seconds for this lap
    pub pace_min_per_km: f64,  // optional/calculated, stored for convenience
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub avg_heart_rate_bpm: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_heart_rate_bpm: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Laps(pub Vec<Lap>);

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct ExerciseSet {
    pub id: i64,
    pub workout_exercise_id: i64,
    pub set_number: i32,
    pub reps: Option<i32>,
    pub weight_kg: Option<f64>,
    pub external_load_kg: Option<f64>,
    pub distance_km: Option<f64>,
    pub duration_seconds: Option<i32>,
    pub rpe: Option<f64>,
    pub rir: Option<f64>,
    pub effective_reps: Option<i32>,
    pub cluster_id: Option<i64>,
    pub rest_seconds: Option<i32>,
    pub notes: Option<String>,
    pub side: Option<String>,
    pub phase: String,
    pub extra_metrics: Option<String>,
    pub avg_heart_rate_bpm: Option<f64>,
    pub max_heart_rate_bpm: Option<f64>,
    pub heart_rate_zones: Option<Json<HeartRateZones>>,
    pub avg_pace_min_per_km: Option<f64>,
    pub calories_burned: Option<i32>,
    pub laps: Option<Json<Vec<Lap>>>,
    pub avg_cadence_spm: Option<f64>,
    pub total_ascent_m: Option<f64>,
    pub total_descent_m: Option<f64>,
    pub created_at: Option<String>,
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct ActivityImport {
    pub id: i64,
    pub workout_id: i64,
    pub source_format: String,
    pub source_filename: Option<String>,
    pub file_sha256: String,
    pub device_name: Option<String>,
    pub manufacturer_id: Option<i64>,
    pub product_id: Option<i64>,
    pub fit_sport: Option<i64>,
    pub fit_sub_sport: Option<i64>,
    pub imported_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trackpoint {
    pub recorded_at: String,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub altitude_m: Option<f64>,
    pub heart_rate_bpm: Option<f64>,
    pub cadence_spm: Option<f64>,
    pub distance_km: Option<f64>,
    pub speed_m_s: Option<f64>,
}
