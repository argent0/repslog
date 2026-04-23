-- Add RIR, effective_reps, and cluster_id support to exercise_sets
ALTER TABLE exercise_sets ADD COLUMN rir REAL;
ALTER TABLE exercise_sets ADD COLUMN effective_reps INTEGER;
ALTER TABLE exercise_sets ADD COLUMN cluster_id INTEGER;
