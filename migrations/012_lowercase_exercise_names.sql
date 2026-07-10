-- Lowercase all exercise names and merge case-only collisions.
-- Merge/rename logic is applied in Rust (db::lowercase_exercise_names) when this
-- migration version is recorded, so that UNIQUE conflicts can be resolved safely
-- (and interactively when a TTY is available).
--
-- After this migration, every exercises.name is lowercase (no uppercase letters).
SELECT 1;
