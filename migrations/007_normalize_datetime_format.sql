-- Normalize date-only timestamps to YYYY-MM-DD HH:MM:SS
UPDATE workouts
SET started_at = started_at || ' 00:00:00'
WHERE length(started_at) = 10;

UPDATE workouts
SET created_at = created_at || ' 00:00:00'
WHERE created_at IS NOT NULL AND length(created_at) = 10;

UPDATE exercises
SET created_at = created_at || ' 00:00:00'
WHERE created_at IS NOT NULL AND length(created_at) = 10;

UPDATE exercise_sets
SET created_at = created_at || ' 00:00:00'
WHERE created_at IS NOT NULL AND length(created_at) = 10;