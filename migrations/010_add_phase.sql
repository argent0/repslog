-- Rep phase: full, eccentric, or concentric.
ALTER TABLE exercise_sets ADD COLUMN phase TEXT;

UPDATE exercise_sets SET phase = 'full' WHERE phase IS NULL;

-- New sets must always specify phase (existing rows backfilled above).
CREATE TABLE exercise_sets_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    workout_exercise_id INTEGER NOT NULL REFERENCES workout_exercises(id) ON DELETE CASCADE,
    set_number INTEGER NOT NULL,
    reps INTEGER,
    weight_kg REAL,
    external_load_kg REAL,
    distance_km REAL,
    duration_seconds INTEGER,
    rpe REAL,
    rir REAL,
    effective_reps INTEGER,
    cluster_id INTEGER,
    rest_seconds INTEGER,
    notes TEXT,
    side TEXT,
    phase TEXT NOT NULL CHECK (phase IN ('full', 'eccentric', 'concentric')),
    extra_metrics TEXT,
    avg_heart_rate_bpm REAL,
    max_heart_rate_bpm REAL,
    heart_rate_zones TEXT,
    avg_pace_min_per_km REAL,
    calories_burned INTEGER,
    laps TEXT,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP
);

INSERT INTO exercise_sets_new (
    id, workout_exercise_id, set_number, reps, weight_kg, external_load_kg,
    distance_km, duration_seconds, rpe, rir, effective_reps, cluster_id,
    rest_seconds, notes, side, phase, extra_metrics, avg_heart_rate_bpm,
    max_heart_rate_bpm, heart_rate_zones, avg_pace_min_per_km, calories_burned,
    laps, created_at
)
SELECT
    id, workout_exercise_id, set_number, reps, weight_kg, external_load_kg,
    distance_km, duration_seconds, rpe, rir, effective_reps, cluster_id,
    rest_seconds, notes, side, phase, extra_metrics, avg_heart_rate_bpm,
    max_heart_rate_bpm, heart_rate_zones, avg_pace_min_per_km, calories_burned,
    laps, created_at
FROM exercise_sets;

DROP TABLE exercise_sets;
ALTER TABLE exercise_sets_new RENAME TO exercise_sets;