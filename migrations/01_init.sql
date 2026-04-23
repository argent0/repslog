PRAGMA foreign_keys = ON;

CREATE TABLE exercises (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    category TEXT NOT NULL,           -- e.g. "strength", "calisthenics", "cardio", "flexibility", "hiit"
    muscle_groups TEXT,               -- JSON array or comma-separated string, e.g. '["chest","triceps"]'
    equipment TEXT,                   -- "barbell", "dumbbell", "bodyweight", "none", etc.
    description TEXT,
    is_custom INTEGER DEFAULT 0,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE workouts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    started_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    finished_at TEXT,
    workout_type TEXT,                -- "push", "pull", "legs", "full_body", "run", "yoga", etc.
    notes TEXT,
    overall_feeling INTEGER CHECK (overall_feeling BETWEEN 1 AND 5 OR overall_feeling IS NULL),
    duration_minutes INTEGER,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE workout_exercises (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    workout_id INTEGER NOT NULL REFERENCES workouts(id) ON DELETE CASCADE,
    exercise_id INTEGER NOT NULL REFERENCES exercises(id),
    "order" INTEGER NOT NULL,
    notes TEXT
);

CREATE TABLE exercise_sets (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    workout_exercise_id INTEGER NOT NULL REFERENCES workout_exercises(id) ON DELETE CASCADE,
    set_number INTEGER NOT NULL,
    reps INTEGER,
    weight_kg REAL,
    distance_km REAL,
    duration_seconds INTEGER,
    rpe REAL,                         -- Rate of Perceived Exertion (e.g. 7.5)
    rest_seconds INTEGER,
    notes TEXT,
    extra_metrics TEXT,               -- JSONB-style for future extensibility
    created_at TEXT DEFAULT CURRENT_TIMESTAMP
);

-- Migration tracking table (handled by sqlx, but following spec)
CREATE TABLE IF NOT EXISTS _migrations (
    version INTEGER PRIMARY KEY,
    applied_at TEXT DEFAULT CURRENT_TIMESTAMP
);
