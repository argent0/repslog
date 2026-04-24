**Prompt for the coding agent:**

You are an expert Rust developer specializing in refactoring CLI applications built with Clap, SQLite (via sqlx or similar), and modular command patterns.

**Project:** https://github.com/argent0/repslog  
**Goal:** Perform a complete, clean refactor to eliminate all statefulness and workout status concepts as described below.

### Core Requirements (non-negotiable)
- **There is no "current" workout ever.** Delete the `workout current` subcommand entirely (including its handler, any `get_current_workout` or similar repository methods, and any related logic).
- **All commands must be fully explicit and stateless.**  
  Any command that needs to operate on a workout (create, view, update, add exercises/sets, etc.) **must** require an explicit workout identifier. No defaults, no implicit lookup of a "current" workout, no global state. Use a consistent `--workout-id <ID>` flag (or positional argument where it already exists) everywhere it makes sense.
- **Remove the workout status BS completely.**  
  Workouts can (and must) be updatable at any time, even long after they were previously "finished."  
  - Remove any `finished_at`, `finished`, `is_active`, `status`, or equivalent field/column from the `Workout` model and database schema.  
  - Delete or fully deprecate the `workout finish` command (or repurpose it into a generic `workout update` if it adds value for setting duration/feeling/notes, but without any locking behavior).  
  - Remove every check, guard, or special case that treats "finished" workouts differently. All workouts are always mutable.
- **All commands must be stateless.** The app should never rely on any implicit current-state lookup. Every operation that needs context must receive it explicitly via arguments.

### What to change (step-by-step guidance)
1. **CLI layer (`src/cli.rs`)**  
   - Remove the `current` subcommand under `workout`.  
   - Ensure every workout-related subcommand requires an explicit workout identifier (add `--workout-id` where missing).  
   - Update help text and argument parsing accordingly.

2. **Models (`src/models/`)**  
   - Update the `Workout` struct (and any related structs) to remove all status/finished fields.  
   - Adjust any serialization/deserialization if needed.

3. **Repository / DB layer (`src/repository.rs`, `src/db.rs`)**  
   - Remove any methods related to "current" workout or status queries.  
   - Update all CRUD operations for workouts so they no longer enforce or check status.  
   - Create a new migration in the `migrations/` directory that safely drops the finished/status column(s) while preserving existing data (use a migration that is backwards-compatible where possible; existing workouts should simply become normal updatable records).

4. **Command handlers (`src/commands/`)**  
   - Refactor every command that previously relied on current workout or status checks.  
   - Make `set add`, `workout-exercise add`, `set quick`, etc. always require the explicit workout ID (or workout-exercise ID as before).  
   - Allow updates to workouts, exercises, and sets at any time.

5. **README.md**  
   - Remove all mentions of `workout current` and any "current workout" workflow.  
   - Update all usage examples to always show explicit workout IDs.  
   - Remove or update the `workout finish` section to reflect the new stateless model.  
   - Emphasize that workouts are always updatable.

6. **Tests, other files**  
   - Update or delete any tests that relied on current workout or status.  
   - Ensure `repslog init`, migrations, stats, etc. still work cleanly.  
   - Keep the excellent scriptability (stdin piping of IDs) and non-interactive design intact.

### Desired new user flow (example)
```bash
repslog workout create --type "Legs" --date "2026-04-23" --notes "..."   # returns ID
repslog workout-exercise add <workout-id> "Squat (Barbell)"
repslog set add <we-id> --reps 10 --weight 100 ...
repslog workout update <workout-id> --duration 60 --feeling 5 --notes "updated later"  # works anytime
```

Keep the rest of the app (advanced cardio/strength features, SQLite storage, migrations, color output, etc.) unchanged unless they were tied to status/current logic.

**Implementation rules:**
- Make changes minimal but complete — only touch what is necessary for the new stateless design.
- Preserve backward compatibility for existing databases as much as possible (new migration should not break old data).
- Keep the code clean, idiomatic Rust, and well-documented.
- After changes, the entire CLI should feel stateless and explicit.

Implement this refactor step-by-step, commit by logical commit if possible, and verify that `cargo check`, `cargo test`, and the app still build and run correctly. Provide a short summary of changes when finished.
