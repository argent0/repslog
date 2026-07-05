-- Separate body weight (weight_kg) from added external load (vest, belt, etc.)
ALTER TABLE exercise_sets ADD COLUMN external_load_kg REAL;