You are an expert Rust developer working on the repslog project: https://github.com/argent0/repslog

Project overview:
- Linux-first CLI workout tracker written in Rust (clap v4 for CLI, SQLite via sqlx or rusqlite for storage, comfy-table for beautiful output).
- Single-file XDG-compliant DB at ~/.local/share/repslog/repslog.db
- Recent feature (added today, April 23 2026): full cardio/running support via migration `003_add_cardio_support.sql` and `repslog set add-cardio <we_id>` command.
- Cardio data lives in the `exercise_sets` table (ExerciseSet struct in src/models/).
- HR zones are already stored as a JSON column (`heart_rate_zones`) with shape: {"z1_seconds": u32, "z2_seconds": u32, ..., "z5_seconds": u32}.

Task: Add **first-class structured support for lap/split tracking** in running/cardio workouts (exactly like the HR zones feature was added).

### Requirements (be thorough and consistent with existing patterns)

1. **Data Model**
   - Add a new optional column `laps` (TEXT / JSON) to the `exercise_sets` table.
   - Create a **new migration** `004_add_laps_support.sql` (follow the exact style of `003_add_cardio_support.sql`).
   - Define a new Rust struct `Lap` in `src/models/` (or inside the ExerciseSet file) with:
     ```rust
     #[derive(Debug, Clone, Serialize, Deserialize, ...)]
     pub struct Lap {
         pub lap_number: u16,           // 1-based
         pub distance_km: f64,          // e.g. 1.0 or 0.98 for last lap
         pub duration_seconds: u32,     // exact seconds for this lap
         pub pace_min_per_km: f64,      // optional/calculated, stored for convenience
         // Future-proof fields (add even if not used yet):
         // avg_heart_rate_bpm: Option<u16>,
     }
     ```
   - Update `ExerciseSet` struct to include:
     ```rust
     pub laps: Option<Vec<Lap>>,
     ```
   - Add proper Serialize/Deserialize (serde) and any DB conversion traits already used for `heart_rate_zones`.

2. **CLI Changes** (`src/cli.rs`)
   - Extend the `add-cardio` subcommand (and any `update-cardio` if it exists) with a new flag:
     ```bash
     --laps <JSON>     # e.g. --laps '[{"lap_number":1,"distance_km":1.0,"duration_seconds":332,"pace_min_per_km":5.533}, ...]'
     ```
   - Make the flag optional (backward compatible).
   - Use clap's `value_parser` for JSON (follow exactly how `--hr-zones` is parsed).
   - Add help text and example in the command doc comment.

3. **Command Logic** (`src/commands/` – wherever `add-cardio` and `update-cardio` live)
   - Parse the `--laps` JSON into `Vec<Lap>`.
   - Validation (important):
     - All laps must have distance_km > 0 and duration_seconds > 0.
     - Sum of distance_km should be within ~1% of the top-level `--distance` (warn on mismatch).
     - Sum of duration_seconds should equal (or be very close to) top-level `--duration`.
     - Lap numbers must be sequential starting from 1.
     - Reject invalid JSON or missing required fields with clear error message.
   - Store the Vec<Lap> in the DB (serialize to JSON).
   - Also support updating laps on existing cardio sets.

4. **Repository / DB Layer** (`src/repository.rs` and models)
   - Update all queries that load/save `ExerciseSet` to handle the new `laps` column (use existing JSON handling pattern).
   - Add helper methods if needed (e.g. `calculate_total_distance_from_laps`, `format_lap_pace`).

5. **Display / Output**
   - When showing a workout (`repslog show ...` or list commands), automatically display a beautiful "Lap Breakdown" section if laps exist.
   - Use comfy-table to format exactly like the user's example:
     ```
     Lap Breakdown:
     Lap 1   1.00 km   5:32   5'32"/km
     Lap 2   1.00 km   5:45   5'45"/km
     ...
     Lap 8   0.98 km   5:26   5'34"/km
     ```
   - Show total distance/time from laps for verification.
   - Keep existing HR zones display unchanged.

6. **Tests**
   - Add comprehensive tests in `tests/` and `spec/`:
     - Round-trip add → show → update laps.
     - Validation edge cases (bad JSON, distance mismatch, non-sequential laps).
     - Backward compatibility (old cardio workouts without laps still work).
     - Migration test for 004.

7. **Documentation & Polish**
   - Update `README.md` with a full example of `add-cardio` using both `--hr-zones` **and** `--laps`.
   - Add to the "Cardio/Running" section with the exact user example from the conversation:
     ```
     Lap1 1km 5:32 5'32"/km, Lap2 1km 5:45 5'45", ... Lap8 0.98km 5:26 5'34"/km
     ```
   - Update any other docs (GEMINI.md if relevant).
   - Ensure `cargo fmt`, `cargo clippy --fix`, and `cargo test` all pass.
   - Keep the code style 100% consistent with the recent cardio PR (commit 16fa584 "Track running").

### Acceptance Criteria
- User can run: `repslog set add-cardio 2 --distance 7.98 --duration 2701 --avg-heart-rate 154 ... --laps '[{"lap_number":1,"distance_km":1.0,"duration_seconds":332,"pace_min_per_km":5.533}, ...]'`
- `repslog show` displays clean Lap Breakdown table.
- Existing cardio workouts continue to work unchanged.
- All data is queryable and structured (not just free-text in Notes).
- No breaking changes.

Implement this cleanly, following the exact architecture patterns used for HR zones. Commit message should be "Add structured lap/split tracking for cardio/running workouts".

Start by exploring the relevant files (`src/models/`, `migrations/003_add_cardio_support.sql`, `src/cli.rs`, the add-cardio command handler, and how `heart_rate_zones` is handled). Then create the new migration and work top-down.

Ask me for clarification on any detail before coding.
