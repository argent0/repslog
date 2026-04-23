# repslog

A Linux-first (but macOS-compatible) command-line workout tracker.

## Features
- SQLite local storage (XDG compliant)
- Flexible data per set (reps, weight, duration, distance, RPE)
- Support for any training type (strength, calisthenics, cardio, etc.)
- Scriptable/non-interactive friendly (stdin support)
- Beautiful tabular output

## Installation
```bash
cargo install --path .
```

## Usage

### 1. Initialize
```bash
repslog init
```

### 2. Manage Exercises
```bash
repslog exercise list
repslog exercise add "Muscle Up" --category calisthenics --equipment bodyweight --muscles "back,triceps"
```

### 3. Start a Workout
```bash
repslog workout create --type "Push" --notes "Feeling strong today"
repslog workout current
```

### 4. Add Exercises to Workout
```bash
repslog workout-exercise add <workout_id> "Pushups"
```

### 5. Log Sets
```bash
# Add a set with 15 reps and 7.5 RPE
repslog set add <workout_exercise_id> --reps 15 --rpe 7.5
```

### 6. Finish Workout
```bash
repslog workout finish <workout_id> --duration 45 --feeling 4
```

### 7. View Stats
```bash
repslog stats prs
repslog stats summary --days 30
```

## Project Structure
- `src/cli.rs`: CLI definitions (clap v4)
- `src/db.rs`: Database connection and migrations
- `src/models/`: Database structs
- `src/commands/`: Command handlers
- `migrations/`: SQL migrations
