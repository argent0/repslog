-- Add laps column to exercise_sets for detailed cardio tracking
ALTER TABLE exercise_sets ADD COLUMN laps TEXT; -- JSON array of laps
