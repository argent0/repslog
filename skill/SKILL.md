# Skill: workout-tracker (repslog)

## Description
`repslog` is a Linux-first, command-line workout tracker designed for high flexibility and scriptability. It allows users and agents to log diverse training modalities including strength training, calisthenics, and cardio. This skill enables LLM agents to fully interact with the `repslog` CLI to manage exercises, log workouts, track sets (including cluster and cardio sets), and analyze training statistics using direct shell execution.

## Capabilities
- **Exercise Management**: List, search, and add custom exercises with categories, muscles, and equipment.
- **Workout Tracking**: Create, view, update, and delete workout sessions.
- **Advanced Set Types**:
    - **Strength/Calisthenics**: Standard sets with weight, reps, RIR (Reps In Reserve), and effective reps.
    - **Cardio**: Track distance, duration, pace, calories, heart rate zones, and laps.
    - **Cluster/Rest-Pause**: Log complex set sequences with predefined rest periods.
- **Statistics & Analysis**: View PRs, training volume, and summaries across exercises and timeframes.
- **Scriptability**: Native support for stdin piping and ID extraction for automated logging workflows.
- **Safety**: Robust `--dry-run` support for all mutating commands.

## Prerequisites
- **Binary**: `repslog` must be installed and available in the `$PATH`.
- **Database**: SQLite database located at `~/.local/share/repslog/repslog.db`.
- **OS**: Optimized for Linux (XDG-compliant), compatible with macOS.

## Core Usage Principles for LLM Agents
1. **Always Use `--dry-run` First**: Before applying any changes, run the command with `--dry-run` to verify the action and see the generated IDs.
2. **ID Extraction**: Most creation commands output the new ID on the last line. Use `tail -n 1` to capture IDs for chaining.
3. **Stdin Chaining**: Use the tool's ability to read IDs from stdin for concise one-liners (e.g., `repslog workout create | repslog workout-exercise add Pushups`).
4. **JSON Arguments**: For cardio data (`--hr-zones`, `--laps`), provide properly escaped JSON strings.
5. **Idempotency**: Check if an exercise or workout exists (via `list` or `search`) before creating a duplicate.

## Command Reference Summary
| Command | Description |
|---------|-------------|
| `repslog init` | Initialize DB and seed default exercises. |
| `repslog exercise list/add` | Manage the exercise library. |
| `repslog workout create/list/view` | Manage workout sessions. |
| `repslog workout-exercise add` | Link an exercise to a workout. |
| `repslog set add` | Log a standard strength set. |
| `repslog set add-cardio` | Log a cardio session with HR/laps. |
| `repslog set add-cluster` | Log rest-pause/cluster sets. |
| `repslog stats prs/volume` | Analyze training progress. |

## Best Practices & Patterns
- **Search Before Add**: Always `repslog exercise search <name>` before adding a new exercise to avoid duplicates.
- **Use Names or IDs**: `workout-exercise add` accepts either the exercise ID or the exact name. Use names for readability in scripts, IDs for precision.
- **Effective Reps**: Track "Effective Reps" (reps performed close to failure) for better growth analysis.
- **Notes**: Use the `--notes` flag on workouts and sets to capture context like "felt tired" or "new PR attempt".

## Common Workflows / Examples

### 1. Initialize Database
```bash
repslog init
```

### 2. Quick Exercise + Set (Convenience)
```bash
repslog set quick --exercise Pushups --reps 20 --rir 2
```

### 3. Full Strength Workout Workflow
```bash
# Create workout
WID=$(repslog workout create --date $(date +%Y-%m-%d) --type strength | tail -n 1)

# Add Bench Press and a set
WEID=$(repslog workout-exercise add $WID "Bench Press" | tail -n 1)
repslog set add $WEID --weight 80 --reps 8 --rir 1

# Add Pullups and a set
WEID=$(repslog workout-exercise add $WID "Pullups" | tail -n 1)
repslog set add $WEID --reps 12 --rir 0
```

### 4. Cardio Workout with Laps and HR Zones
```bash
WEID=$(repslog workout create --date 2026-04-27 --type cardio | repslog workout-exercise add Running | tail -n 1)

repslog set add-cardio $WEID \
  --distance 5.0 --duration 1500 \
  --avg-heart-rate 155 --max-heart-rate 180 --pace 5.0 --calories 450 \
  --hr-zones '{"z1_seconds": 60, "z2_seconds": 1200, "z3_seconds": 240}' \
  --laps '[{"km": 1, "time": "5:32", "pace": "5:32"}]'
```

### 5. Cluster/Rest-Pause Set
```bash
# Adds a sequence of 10, 5, and 5 reps with 15s rest between them
repslog set add-cluster $WEID --reps "10,5,5" --weight 100 --rir "0,0,1" --effective-reps "6,4,3" --rest 15
```

### 6. View PRs for an Exercise
```bash
repslog stats prs --exercise "Deadlift"
```

### 7. Scripted Multi-step Workflow
```bash
repslog workout create --date 2026-04-27 | \
  repslog workout-exercise add "Muscle Up" | \
  repslog set add --reps 5 --rir 1
```

## Limitations
- **Local Only**: Data is stored in a local SQLite file; no built-in cloud synchronization.
- **Linux First**: Designed for Linux environments; paths and behaviors may vary on other systems.
- **Terminal Dependent**: Tables and colored output are optimized for standard TTYs.

## Safety & Idempotency
- **Dry-Run**: ALL mutating commands (`add`, `create`, `update`, `delete`, `migrate`, `init`) support the `--dry-run` flag. **Use it extensively.**
- **Migrations**: Use `repslog migrate --status` to check if the database schema is up to date before performing complex operations.
