use crate::error::Result;
use crate::utils::parse_datetime;
use crate::models::{Exercise, ExerciseSet, HeartRateZones, Lap, Workout, WorkoutExercise};
use sqlx::sqlite::SqlitePool;
use sqlx::types::Json;
use sqlx::Row;

pub struct Repository {
    pub pool: SqlitePool,
}

impl Repository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    // --- Exercises ---
    #[allow(clippy::too_many_arguments)]
    pub async fn add_exercise(
        &self,
        name: &str,
        category: &str,
        muscle_groups: Option<&str>,
        equipment: Option<&str>,
        description: Option<&str>,
        is_custom: bool,
        dry_run: bool,
    ) -> Result<i64> {
        if dry_run {
            return self.get_next_id("exercises").await;
        }
        let res = sqlx::query("INSERT INTO exercises (name, category, muscle_groups, equipment, description, is_custom) VALUES (?, ?, ?, ?, ?, ?)")
            .bind(name)
            .bind(category)
            .bind(muscle_groups)
            .bind(equipment)
            .bind(description)
            .bind(is_custom as i32)
            .execute(&self.pool)
            .await?;
        Ok(res.last_insert_rowid())
    }

    pub async fn list_exercises(
        &self,
        search: Option<String>,
        category: Option<String>,
    ) -> Result<Vec<Exercise>> {
        let mut query = "SELECT * FROM exercises WHERE 1=1".to_string();
        if let Some(s) = &search {
            query.push_str(&format!(" AND name LIKE '%{}%'", s));
        }
        if let Some(c) = &category {
            query.push_str(&format!(" AND category = '{}'", c));
        }
        let exercises = sqlx::query_as::<_, Exercise>(&query)
            .fetch_all(&self.pool)
            .await?;
        Ok(exercises)
    }

    pub async fn find_exercise_by_id_or_name(&self, id_or_name: &str) -> Result<Option<Exercise>> {
        if let Ok(id) = id_or_name.parse::<i64>() {
            Ok(
                sqlx::query_as::<_, Exercise>("SELECT * FROM exercises WHERE id = ?")
                    .bind(id)
                    .fetch_optional(&self.pool)
                    .await?,
            )
        } else {
            Ok(
                sqlx::query_as::<_, Exercise>("SELECT * FROM exercises WHERE name = ?")
                    .bind(id_or_name)
                    .fetch_optional(&self.pool)
                    .await?,
            )
        }
    }

    // --- Workouts ---
    pub async fn create_workout(
        &self,
        workout_type: Option<&str>,
        notes: Option<&str>,
        started_at: Option<&str>,
        dry_run: bool,
    ) -> Result<i64> {
        if dry_run {
            return self.get_next_id("workouts").await;
        }
        let started_at = started_at.map(parse_datetime).transpose()?;
        let res = sqlx::query("INSERT INTO workouts (workout_type, notes, started_at) VALUES (?, ?, COALESCE(?, strftime('%Y-%m-%d %H:%M:%S', 'now')))")
            .bind(workout_type)
            .bind(notes)
            .bind(started_at.as_deref())
            .execute(&self.pool)
            .await?;
        Ok(res.last_insert_rowid())
    }

    pub async fn list_workouts(&self, limit: i64, days: Option<i64>) -> Result<Vec<Workout>> {
        let mut query = "SELECT * FROM workouts".to_string();
        if let Some(d) = days {
            query.push_str(&format!(" WHERE started_at >= date('now', '-{} days')", d));
        }
        query.push_str(&format!(" ORDER BY started_at DESC LIMIT {}", limit));
        let workouts = sqlx::query_as::<_, Workout>(&query)
            .fetch_all(&self.pool)
            .await?;
        Ok(workouts)
    }

    pub async fn get_workout(&self, id: i64) -> Result<Option<Workout>> {
        Ok(
            sqlx::query_as::<_, Workout>("SELECT * FROM workouts WHERE id = ?")
                .bind(id)
                .fetch_optional(&self.pool)
                .await?,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn update_workout(
        &self,
        id: i64,
        workout_type: Option<&str>,
        notes: Option<&str>,
        duration: Option<i32>,
        feeling: Option<i32>,
        started_at: Option<&str>,
        dry_run: bool,
    ) -> Result<()> {
        if dry_run {
            return Ok(());
        }
        let started_at = started_at.map(parse_datetime).transpose()?;
        sqlx::query("UPDATE workouts SET workout_type = COALESCE(?, workout_type), notes = COALESCE(?, notes), duration_minutes = COALESCE(?, duration_minutes), overall_feeling = COALESCE(?, overall_feeling), started_at = COALESCE(?, started_at) WHERE id = ?")
            .bind(workout_type)
            .bind(notes)
            .bind(duration)
            .bind(feeling)
            .bind(started_at.as_deref())
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn delete_workout(&self, id: i64, dry_run: bool) -> Result<()> {
        if dry_run {
            return Ok(());
        }
        sqlx::query("DELETE FROM workouts WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    // --- Workout Exercises ---
    pub async fn add_workout_exercise(
        &self,
        workout_id: i64,
        exercise_id: i64,
        order: i32,
        notes: Option<&str>,
        dry_run: bool,
    ) -> Result<i64> {
        if dry_run {
            return self.get_next_id("workout_exercises").await;
        }
        let res = sqlx::query("INSERT INTO workout_exercises (workout_id, exercise_id, \"order\", notes) VALUES (?, ?, ?, ?)")
            .bind(workout_id)
            .bind(exercise_id)
            .bind(order)
            .bind(notes)
            .execute(&self.pool)
            .await?;
        Ok(res.last_insert_rowid())
    }

    pub async fn list_workout_exercises(
        &self,
        workout_id: i64,
    ) -> Result<Vec<(WorkoutExercise, String)>> {
        let res = sqlx::query("SELECT we.*, e.name as exercise_name FROM workout_exercises we JOIN exercises e ON we.exercise_id = e.id WHERE we.workout_id = ? ORDER BY we.\"order\"")
            .bind(workout_id)
            .fetch_all(&self.pool)
            .await?;

        let exercises = res
            .into_iter()
            .map(|r| {
                (
                    WorkoutExercise {
                        id: r.get("id"),
                        workout_id: r.get("workout_id"),
                        exercise_id: r.get("exercise_id"),
                        order: r.get("order"),
                        notes: r.get("notes"),
                        goal_reps: r.get("goal_reps"),
                    },
                    r.get("exercise_name"),
                )
            })
            .collect();
        Ok(exercises)
    }

    pub async fn get_max_order_for_workout(&self, workout_id: i64) -> Result<i32> {
        let res = sqlx::query(
            "SELECT MAX(\"order\") as max_order FROM workout_exercises WHERE workout_id = ?",
        )
        .bind(workout_id)
        .fetch_one(&self.pool)
        .await?;
        Ok(res.get::<Option<i32>, _>("max_order").unwrap_or(0))
    }

    // --- Sets ---
    #[allow(clippy::too_many_arguments)]
    pub async fn add_set(
        &self,
        workout_exercise_id: i64,
        set_number: i32,
        reps: Option<i32>,
        weight: Option<f64>,
        duration: Option<i32>,
        distance: Option<f64>,
        rpe: Option<f64>,
        rir: Option<f64>,
        effective_reps: Option<i32>,
        cluster_id: Option<i64>,
        rest_seconds: Option<i32>,
        notes: Option<&str>,
        side: Option<&str>,
        avg_heart_rate: Option<f64>,
        max_heart_rate: Option<f64>,
        hr_zones: Option<Json<HeartRateZones>>,
        pace: Option<f64>,
        calories: Option<i32>,
        laps: Option<Json<Vec<Lap>>>,
        dry_run: bool,
    ) -> Result<i64> {
        if dry_run {
            return self.get_next_id("exercise_sets").await;
        }
        let res = sqlx::query("INSERT INTO exercise_sets (
            workout_exercise_id, set_number, reps, weight_kg, duration_seconds, distance_km, rpe, rir, 
            effective_reps, cluster_id, rest_seconds, notes, side, avg_heart_rate_bpm, max_heart_rate_bpm, 
            heart_rate_zones, avg_pace_min_per_km, calories_burned, laps
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)")
            .bind(workout_exercise_id)
            .bind(set_number)
            .bind(reps)
            .bind(weight)
            .bind(duration)
            .bind(distance)
            .bind(rpe)
            .bind(rir)
            .bind(effective_reps)
            .bind(cluster_id)
            .bind(rest_seconds)
            .bind(notes)
            .bind(side)
            .bind(avg_heart_rate)
            .bind(max_heart_rate)
            .bind(hr_zones)
            .bind(pace)
            .bind(calories)
            .bind(laps)
            .execute(&self.pool)
            .await?;
        Ok(res.last_insert_rowid())
    }

    pub async fn list_sets(&self, workout_exercise_id: i64) -> Result<Vec<ExerciseSet>> {
        // Logical order: left sets first, then right, then both/unspecified, then by set_number.
        // This supports clean unilateral display without changing the meaning of set_number.
        let sets = sqlx::query_as::<_, ExerciseSet>(
            "SELECT * FROM exercise_sets WHERE workout_exercise_id = ? ORDER BY CASE COALESCE(side, '') WHEN 'left' THEN 0 WHEN 'right' THEN 1 WHEN 'both' THEN 2 ELSE 99 END, set_number",
        )
        .bind(workout_exercise_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(sets)
    }

    pub async fn get_next_set_number(&self, workout_exercise_id: i64) -> Result<i32> {
        let res = sqlx::query(
            "SELECT MAX(set_number) as max_set FROM exercise_sets WHERE workout_exercise_id = ?",
        )
        .bind(workout_exercise_id)
        .fetch_one(&self.pool)
        .await?;
        Ok(res.get::<Option<i32>, _>("max_set").unwrap_or(0) + 1)
    }

    pub async fn get_next_cluster_id(&self) -> Result<i64> {
        let res = sqlx::query("SELECT MAX(cluster_id) as max_cluster FROM exercise_sets")
            .fetch_one(&self.pool)
            .await?;
        Ok(res.get::<Option<i64>, _>("max_cluster").unwrap_or(0) + 1)
    }

    pub async fn get_next_id(&self, table_name: &str) -> Result<i64> {
        let res = sqlx::query("SELECT seq FROM sqlite_sequence WHERE name = ?")
            .bind(table_name)
            .fetch_optional(&self.pool)
            .await?;

        match res {
            Some(row) => Ok(row.get::<i64, _>("seq") + 1),
            None => {
                // If not in sqlite_sequence, check if table has any rows
                let query = format!("SELECT MAX(id) as max_id FROM {}", table_name);
                let res = sqlx::query(&query).fetch_one(&self.pool).await?;
                Ok(res.get::<Option<i64>, _>("max_id").unwrap_or(0) + 1)
            }
        }
    }

    // --- Set management (update/delete/move for corrections and unilateral workflows) ---

    pub async fn get_set(&self, id: i64) -> Result<Option<ExerciseSet>> {
        Ok(
            sqlx::query_as::<_, ExerciseSet>("SELECT * FROM exercise_sets WHERE id = ?")
                .bind(id)
                .fetch_optional(&self.pool)
                .await?,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn update_set(
        &self,
        id: i64,
        reps: Option<i32>,
        weight: Option<f64>,
        duration: Option<i32>,
        distance: Option<f64>,
        rpe: Option<f64>,
        rir: Option<f64>,
        effective_reps: Option<i32>,
        rest_seconds: Option<i32>,
        notes: Option<&str>,
        side: Option<&str>,
        dry_run: bool,
    ) -> Result<()> {
        if dry_run {
            return Ok(());
        }
        sqlx::query(
            "UPDATE exercise_sets SET \
             reps = COALESCE(?, reps), \
             weight_kg = COALESCE(?, weight_kg), \
             duration_seconds = COALESCE(?, duration_seconds), \
             distance_km = COALESCE(?, distance_km), \
             rpe = COALESCE(?, rpe), \
             rir = COALESCE(?, rir), \
             effective_reps = COALESCE(?, effective_reps), \
             rest_seconds = COALESCE(?, rest_seconds), \
             notes = COALESCE(?, notes), \
             side = COALESCE(?, side) \
             WHERE id = ?",
        )
        .bind(reps)
        .bind(weight)
        .bind(duration)
        .bind(distance)
        .bind(rpe)
        .bind(rir)
        .bind(effective_reps)
        .bind(rest_seconds)
        .bind(notes)
        .bind(side)
        .bind(id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn delete_set(&self, id: i64, dry_run: bool) -> Result<()> {
        if dry_run {
            return Ok(());
        }
        sqlx::query("DELETE FROM exercise_sets WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Reorder a set within its workout-exercise by cleanly renumbering 1..N.
    /// new_position is 1-based (clamped to valid range).
    pub async fn reorder_set(&self, set_id: i64, new_position: i32, dry_run: bool) -> Result<()> {
        if dry_run {
            return Ok(());
        }

        // Find the workout_exercise this set belongs to
        let set_row = sqlx::query("SELECT workout_exercise_id FROM exercise_sets WHERE id = ?")
            .bind(set_id)
            .fetch_optional(&self.pool)
            .await?;
        let we_id: i64 = match set_row {
            Some(r) => r.get("workout_exercise_id"),
            None => return Ok(()), // nothing to do
        };

        // Load current sets in logical order (will respect side-aware ORDER BY in list_sets, but we use set_number primarily for relative intent)
        let mut sets = self.list_sets(we_id).await?;
        if sets.is_empty() {
            return Ok(());
        }

        // Find index of the set to move
        let idx = sets.iter().position(|s| s.id == set_id);
        if idx.is_none() {
            return Ok(());
        }
        let moving = sets.remove(idx.unwrap());

        // Compute target index (0-based), clamp
        let mut target_idx = (new_position - 1).max(0) as usize;
        if target_idx > sets.len() {
            target_idx = sets.len();
        }
        sets.insert(target_idx, moving);

        // Renumber sequentially 1..N and persist
        let mut tx = self.pool.begin().await?;
        for (i, s) in sets.iter().enumerate() {
            let new_num = (i as i32) + 1;
            if new_num != s.set_number {
                sqlx::query("UPDATE exercise_sets SET set_number = ? WHERE id = ?")
                    .bind(new_num)
                    .bind(s.id)
                    .execute(&mut *tx)
                    .await?;
            }
        }
        tx.commit().await?;
        Ok(())
    }
}
