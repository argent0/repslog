# Scripting & Automation

`repslog` is designed to be non-interactive friendly and fully scriptable. Many commands support reading IDs from `stdin`, allowing you to chain commands together using pipes.

## Chaining Commands with Pipes

You can pipe the output of one command (which returns an ID) into another. Some commands support reading the ID directly from `stdin`, while others are best used with `xargs`.

### Example: Create Workout and Add Exercise (using xargs)
```bash
repslog workout create --date "2026-04-23 10:00:00" --type "Push" | xargs -I {} repslog workout-exercise add {} "bench press"
```

In this example:
1. `workout create` outputs the new Workout ID.
2. `xargs` takes that ID and passes it as the first argument to `workout-exercise add`.

### Example: Add Exercise and Log a Set (direct piping)
```bash
repslog workout-exercise add 1 "squat (barbell)" | repslog set add --reps 10 --weight 100 --phase full --rir 1
```

In this example:
1. `workout-exercise add` outputs the new Workout-Exercise ID (WE_ID).
2. `repslog set add` reads that ID directly from `stdin` because it was not provided as an argument.

## Batch Logging

You can use standard shell tools like `xargs` or simple loops to batch log data.

```bash
# Log 3 identical sets
echo "10" | xargs -I {} repslog set add <WE_ID> --reps {} --weight 60 --phase full --rir 2
```

## JSON Support

Some flags, like `--hr-zones` and `--laps` in `set add-cardio`, require JSON strings. You can use tools like `jq` to construct these strings dynamically in your scripts.

```bash
ZONES=$(jq -n --arg z2 1800 '{"z2_seconds": ($z2|tonumber)}')
repslog set add-cardio <WE_ID> --phase full --distance 5 --duration 1500 --hr-zones "$ZONES"
```

## JSON Output with --json

Use the global `--json` flag on read commands (list, view, stats, etc.) and ID-producing commands to get structured machine-readable output. This is ideal for scripting with `jq`.

```bash
# List as JSON array
repslog exercise list --json

# Get a workout with full nested exercises and sets
repslog workout view 1 --json | jq '.exercises[0].sets'

# Stats as JSON
repslog stats prs --json
repslog stats summary --days 7 --json

# Capture IDs from JSON output (works with dry-run too)
WORKOUT_ID=$(repslog workout create --date "2026-04-23 10:00:00" --json | jq -r '.id')
echo "Created $WORKOUT_ID"
```

Mutation commands that normally print bare IDs (for piping) will output `{"id": "N"}` (or `{"id": "DRY-RUN-N"}`) when `--json` is used.

Migrate status and init also support `--json`.

## Non-Interactive Errors

When `repslog` is used in a script, it will exit with a non-zero status code if an error occurs, allowing your scripts to handle failures gracefully.

```bash
if ! repslog workout create --date "invalid-date"; then
  echo "Failed to create workout"
  exit 1
fi
```

## Dry Run Mode

Mutation commands support a `--dry-run` flag. This is useful for testing scripts without modifying the database. 

When `--dry-run` is used:
1. Arguments and logic are fully validated.
2. The database is **not** modified.
3. The command outputs a placeholder ID in the format `DRY-RUN-N` (e.g., `DRY-RUN-10`), where `N` is the next expected ID.

This allows you to test full pipelines:
```bash
# Test a pipeline without making any changes
repslog workout create --dry-run --date "2026-04-24 10:00:00" | xargs -I {} repslog workout-exercise add {} "pushups" --dry-run
```
