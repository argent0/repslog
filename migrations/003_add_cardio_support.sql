-- Add cardio-specific fields to exercise_sets
ALTER TABLE exercise_sets ADD COLUMN avg_heart_rate_bpm REAL;
ALTER TABLE exercise_sets ADD COLUMN max_heart_rate_bpm REAL;
ALTER TABLE exercise_sets ADD COLUMN heart_rate_zones TEXT; -- JSON string
ALTER TABLE exercise_sets ADD COLUMN avg_pace_min_per_km REAL;
ALTER TABLE exercise_sets ADD COLUMN calories_burned INTEGER;
