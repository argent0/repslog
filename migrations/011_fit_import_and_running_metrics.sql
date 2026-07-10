-- FIT import provenance and extra running metrics

CREATE TABLE IF NOT EXISTS activity_imports (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    workout_id INTEGER NOT NULL REFERENCES workouts(id) ON DELETE CASCADE,
    source_format TEXT NOT NULL,
    source_filename TEXT,
    file_sha256 TEXT NOT NULL UNIQUE,
    device_name TEXT,
    manufacturer_id INTEGER,
    product_id INTEGER,
    fit_sport INTEGER,
    fit_sub_sport INTEGER,
    imported_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%d %H:%M:%S', 'now'))
);

CREATE TABLE IF NOT EXISTS activity_trackpoints (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    exercise_set_id INTEGER NOT NULL REFERENCES exercise_sets(id) ON DELETE CASCADE,
    recorded_at TEXT NOT NULL,
    latitude REAL,
    longitude REAL,
    altitude_m REAL,
    heart_rate_bpm REAL,
    cadence_spm REAL,
    distance_km REAL,
    speed_m_s REAL
);

CREATE INDEX IF NOT EXISTS idx_trackpoints_set ON activity_trackpoints(exercise_set_id);

ALTER TABLE exercise_sets ADD COLUMN avg_cadence_spm REAL;
ALTER TABLE exercise_sets ADD COLUMN total_ascent_m REAL;
ALTER TABLE exercise_sets ADD COLUMN total_descent_m REAL;
