# repslog

A Linux-first command-line workout tracker designed for flexibility across strength training, calisthenics, and cardio.

## Features
- **SQLite Storage:** Local, single-file database (XDG compliant: `~/.local/share/repslog/`).
- **Cardio First-Class Support:** Detailed tracking for runs (heart rate zones, pace, calories, and structured laps/splits).
- **Advanced Strength Training:** Support for RPE, RIR, Effective Reps, Rest-Pause/Cluster sets, and first-class unilateral (left/right) tracking with corrections (update/delete/move).
- **Scriptable:** Non-interactive friendly; supports reading IDs from `stdin`.
- **Beautiful Output:** Color-coded tabular views using `comfy-table` with dedicated, runner-friendly displays for cardio workouts including visual HR zone bars and lap breakdowns.

## Installation
```bash
cargo install --path .
```

## Usage Guide

All commands that modify the database support a `--dry-run` flag to preview changes without applying them.

The `--db <PATH>` global option lets you target a specific SQLite file (handy for testing or isolated runs): `repslog --db /tmp/test.db ...`

The `--json` global option makes list/view/stats/create output machine-readable JSON (for use with `jq` etc.).

### 1. Initialize
Sets up the database and seeds default exercises.
```bash
repslog init
```

### 2. Manage Exercises
```bash
# List all exercises
repslog exercise list

# Search for specific exercises
repslog exercise search "Squat"

# Add a custom exercise
repslog exercise add "Bulgarian Split Squat" --category strength --equipment dumbbell --muscles "quads,glutes"
```

### 3. Workouts (Sessions)
Workouts are the top-level containers for your training. `workout create` is step 1 only — you must add exercises and log sets afterward. See `docs/workouts.md` (Data Entry Best Practices) and `docs/logging.md` for the full workflow.
```bash
# Create a new workout (date is mandatory: YYYY-MM-DD)
repslog workout create --type "Legs" --date "2026-04-23" --notes "Focus on form"

# List recent workouts (now includes automated cardio summaries!)
repslog workout list --days 7

# View full details of a workout
# Pure strength workouts show sets/reps, while cardio workouts get a 
# dedicated summary section with pace, HR zones, and laps.
repslog workout view 1
```

### 4. Adding Exercises to a Workout
```bash
# Add an exercise to workout ID 1
repslog workout-exercise add 1 "Squat (Barbell)"

# List exercises added to a workout
repslog workout-exercise list 1
```

### 5. Logging Sets

#### Strength / Hypertrophy
```bash
# Basic set: 10 reps at 100kg with 1 RIR
repslog set add <we_id> --reps 10 --weight 100 --rir 1.0 --effective-reps 5

# Rest-Pause / Cluster Sets
# Log 3 mini-sets (10, 5, 5 reps) with 15s rest between them
repslog set add-cluster <we_id> --reps "10,5,5" --weight 100 --rir "0,0,1" --effective-reps "6,4,3" --rest 15
```

#### Cardio / Running (recommended structured path)
Always log runs with `set add-cardio` — do not store distance, pace, HR, or laps only in workout `--notes`. Includes explicit support for Samsung Health style metrics and structured lap/split tracking.
```bash
# Recommended: structured run tracking with HR zones and laps/splits
repslog set add-cardio <we_id> \
  --distance 7.98 \
  --duration 2701 \
  --avg-heart-rate 154 \
  --max-heart-rate 175 \
  --pace 5.64 \
  --calories 620 \
  --hr-zones '{"z1_seconds": 120, "z2_seconds": 1800, "z3_seconds": 600, "z4_seconds": 120, "z5_seconds": 61}' \
  --laps '[
    {"lap_number":1,"distance_km":1.0,"duration_seconds":332,"pace_min_per_km":5.533},
    {"lap_number":2,"distance_km":1.0,"duration_seconds":345,"pace_min_per_km":5.75},
    {"lap_number":3,"distance_km":1.0,"duration_seconds":338,"pace_min_per_km":5.633},
    {"lap_number":8,"distance_km":0.98,"duration_seconds":326,"pace_min_per_km":5.567}
  ]'
```
*(Example: Lap1 1km 5:32 5'32"/km, Lap2 1km 5:45 5'45", ... Lap8 0.98km 5:26 5'34"/km)*

#### Convenience Commands
```bash
# Quick: Add exercise and first set in one command
repslog set quick 1 "Pushups"
```

**New: Beautiful Cardio Display**
When you view a cardio workout, repslog now provides a high-signal summary:
- **Aggregated Totals:** Distance, Time, Pace, Avg/Max HR, and Calories.
- **Visual HR Zones:** A color-coded bar (Cyan/Green/Yellow/Magenta/Red) showing distribution across Z1-Z5 with percentages.
- **Lap Table:** Clear breakdown of every split with distance, time, and pace.

### 6. Updating & Stats
```bash
# Update workout details like duration, feeling (1-5), or notes anytime
repslog workout update 1 --duration 60 --feeling 5 --notes "Updated notes"

# View Personal Records
...
repslog stats prs --exercise "Squat"

# View Volume Trends
repslog stats volume --period 90d

# Training Summary
repslog stats summary --days 30
```

### 7. Scripting & Stdin
You can pipe IDs from one command to another for faster logging:
```bash
# Example: Create workout and immediately add an exercise using the piped ID
repslog workout create --date "2026-04-23" | repslog workout-exercise add "Running"
```

### 8. Database Migrations
Keep your schema up-to-date.
```bash
repslog migrate                 # Apply all pending migrations
repslog migrate --status        # Show current vs. latest version
repslog migrate --dry-run       # Show what would be applied
repslog migrate --force         # Force re-apply all migrations (idempotent, fixes partial applications)
```

## LLM Agent Skills

`repslog` includes a set of skills and reference documentation for LLM agents (like Claude Code). When installed via the Arch Linux package, these are located at `/usr/share/repslog/skills/workout-tracker/`.

To use them with Claude Code (or any compatible agent), copy or symlink them to your local skills directory:

```bash
mkdir -p ~/.claude/skills/workout-tracker
cp -r /usr/share/repslog/skills/workout-tracker/* ~/.claude/skills/workout-tracker/
```

## Documentation

Comprehensive documentation is available in the `docs/` folder.

### Verifying Documentation Examples
The documentation includes testable examples. You can verify that all documented commands are working correctly by running the verification script:

```bash
# Build the latest version first
cargo build
# Run the verification script
./docs/verify_examples.sh
```

This script creates a temporary isolated environment to test the full lifecycle of `repslog` as described in the docs.

## Project Structure
- `src/cli.rs`: CLI definitions (clap v4)
- `src/db.rs`: Database connection and migrations
- `src/repository.rs`: Data access layer
- `src/models/`: Database entity definitions
- `src/commands/`: Command logic
- `migrations/`: SQL schema evolution
