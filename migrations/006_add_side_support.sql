-- Add side column for unilateral (left/right/both) tracking on sets.
-- Also add goal_reps on workout_exercises to support goal vs actual tracking (spec §4).
ALTER TABLE exercise_sets ADD COLUMN side TEXT;
ALTER TABLE workout_exercises ADD COLUMN goal_reps INTEGER;