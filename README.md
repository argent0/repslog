# repslog

A Linux-first command-line workout tracker designed for flexibility across strength training, calisthenics, and cardio.

## Features
- **SQLite Storage:** Local, single-file database (XDG compliant: `~/.local/share/repslog/`).
- **Cardio First-Class Support:** Detailed tracking for runs (heart rate zones, pace, calories).
- **Advanced Strength Training:** Support for RPE, RIR, Effective Reps, and Rest-Pause/Cluster sets.
- **Scriptable:** Non-interactive friendly; supports reading IDs from `stdin`.
- **Beautiful Output:** Color-coded tabular views using `comfy-table`.

## Installation
```bash
cargo install --path .
```

## Usage Guide

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
Workouts are the top-level containers for your training.
```bash
# Create a new workout (date is mandatory: YYYY-MM-DD)
repslog workout create --type "Legs" --date "2026-04-23" --notes "Focus on form"

# List recent workouts
repslog workout list --days 7

# Show the currently active (unfinished) workout
repslog workout current

# View full details of a workout (including exercises and sets)
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

#### Cardio / Running
Includes explicit support for Samsung Health style metrics.
```bash
# Detailed run tracking
repslog set add-cardio <we_id> \
  --distance 5.0 \
  --duration 1500 \
  --avg-heart-rate 155 \
  --max-heart-rate 180 \
  --pace 5.0 \
  --calories 450 \
  --hr-zones '{"z1_seconds": 60, "z2_seconds": 1200, "z3_seconds": 240}'
```

#### Convenience Commands
```bash
# Quick: Add exercise and first set in one command
repslog set quick 1 "Pushups"
```

### 6. Finishing & Stats
```bash
# Finish workout and record duration/feeling (1-5)
repslog workout finish 1 --duration 60 --feeling 5

# View Personal Records
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
```

## Project Structure
- `src/cli.rs`: CLI definitions (clap v4)
- `src/db.rs`: Database connection and migrations
- `src/repository.rs`: Data access layer
- `src/models/`: Database entity definitions
- `src/commands/`: Command logic
- `migrations/`: SQL schema evolution
