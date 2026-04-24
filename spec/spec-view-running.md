You are an expert Rust CLI developer with deep experience in Clap, SQLx/SQLite, and comfy-table. Your task is to **add significantly better display support for running workouts** in the repslog app[](https://github.com/argent0/repslog).

### Project Overview
- Linux-first command-line workout tracker written in Rust.
- Uses SQLite (single file at `~/.local/share/repslog/`) with migrations.
- Strong existing support for **strength training** (RPE, RIR, effective reps, rest-pause, clusters) and **cardio/running** (first-class citizen).
- Cardio sets are stored via `set add-cardio` with fields:
  - `distance` (km)
  - `duration` (seconds)
  - `avg_heart_rate`, `max_heart_rate`
  - `pace` (min/km)
  - `calories`
  - `hr_zones` (JSON: seconds per zone z1–z5)
  - `laps` (JSON array of structured laps: lap_number, distance_km, duration_seconds, pace_min_per_km)
- UI is 100% CLI using `clap v4` for commands and `comfy-table` for color-coded, nicely formatted tables.
- Key display commands live in `src/commands/workout.rs` (workout list + workout view) and `src/commands/set.rs`.

### Current State (What Already Works)
- Data entry for running workouts is excellent and recent (structured laps were just added).
- `repslog workout list --days N` shows workouts in a table.
- `repslog workout view <id>` shows full workout details including all sets.
- Cardio data is already being fetched from the models/repository.

### What "Better Support for Displaying Running Workouts" Means
The current display treats cardio sets like any other set and does not take advantage of the rich running-specific data. We want **runner-friendly, glanceable, beautiful output** similar to what dedicated running apps show.

### Specific Requirements
1. **Workout List (`repslog workout list`)**
   - Add a new "Summary" column (or enhance the existing one).
   - For running/cardio workouts, show something clean like:
     - "Run • 8.2 km • 45:12 • 5:31/km • 162 bpm"
   - Use color coding (e.g., green for good pace, yellow for HR zones).
   - Keep the table compact and aligned.

2. **Workout View (`repslog workout view <id>`) – Main Focus**
   - When a workout contains cardio/running sets, display them in a **dedicated, beautiful section**.
   - Show **overall run summary** first (total distance, total time, average pace, avg/max HR, calories, HR zone distribution as percentages or a simple text bar).
   - Then a **clean laps/splits table** using comfy-table:
     - Columns: Lap #, Distance, Time, Pace (formatted as mm:ss/km), HR if available.
     - Nice formatting: convert seconds → mm:ss, pace as mm:ss/km, distance with 2 decimals.
   - If multiple cardio sets exist in one workout (rare but possible), group them logically.
   - Fall back gracefully to current behavior for pure strength workouts.

3. **Bonus Polish (Highly Recommended)**
   - Add helper functions in `src/models/` or a new `src/display/` module to format:
     - Seconds → human-readable duration ("45:12")
     - Pace (min/km) → mm:ss/km
     - HR zones → percentages + simple visual (e.g., ████░░ for zone distribution)
   - Consistent color scheme (use existing comfy-table colors where possible).
   - Make sure JSON deserialization of laps/hr_zones is robust and handles missing data.

4. **Non-Goals**
   - Do **not** change data models or add-cardio command.
   - Do **not** add maps/GPS (keep it pure CLI).
   - Do **not** add new top-level commands unless absolutely necessary.

### Implementation Guidelines
- Work in small, reviewable changes.
- Update `src/commands/workout.rs` and any helper functions you create.
- Reuse existing repository queries where possible.
- Keep everything backward-compatible.
- Add helpful comments explaining the running display logic.
- Test mentally with the example lap data from the README (7.98 km run with 8 laps).

After implementing, provide:
1. A summary of files changed.
2. Before/after examples of the new `workout list` and `workout view` output for a running workout.
3. Any new helper functions you added.

Start by exploring the relevant files (`src/commands/workout.rs`, `src/commands/set.rs`, models, repository) and then implement the enhanced display. Let's make repslog the best CLI running log available!
