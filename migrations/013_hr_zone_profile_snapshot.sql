-- Snapshot inputs for HR zone calculation (from bodylog at import time).
-- Age is derived from date_of_birth plus activity date. resting_hr_bpm is median sleep HR.
ALTER TABLE exercise_sets ADD COLUMN date_of_birth TEXT;
ALTER TABLE exercise_sets ADD COLUMN resting_hr_bpm REAL;
