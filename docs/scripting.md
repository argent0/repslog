# Scripting & Automation

`repslog` is designed to be non-interactive friendly and fully scriptable. Many commands support reading IDs from `stdin`, allowing you to chain commands together using pipes.

## Chaining Commands with Pipes

You can pipe the output of one command (which returns an ID) into another. Some commands support reading the ID directly from `stdin`, while others are best used with `xargs`.

### Example: Create Workout and Add Exercise (using xargs)
```bash
repslog workout create --date "2026-04-23" --type "Push" | xargs -I {} repslog workout-exercise add {} "Bench Press"
```

In this example:
1. `workout create` outputs the new Workout ID.
2. `xargs` takes that ID and passes it as the first argument to `workout-exercise add`.

### Example: Add Exercise and Log a Set (direct piping)
```bash
repslog workout-exercise add 1 "Squat (Barbell)" | repslog set add --reps 10 --weight 100 --rir 1
```

In this example:
1. `workout-exercise add` outputs the new Workout-Exercise ID (WE_ID).
2. `repslog set add` reads that ID directly from `stdin` because it was not provided as an argument.

## Batch Logging

You can use standard shell tools like `xargs` or simple loops to batch log data.

```bash
# Log 3 identical sets
echo "10" | xargs -I {} repslog set add <WE_ID> --reps {} --weight 60 --rir 2
```

## JSON Support

Some flags, like `--hr-zones` and `--laps` in `set add-cardio`, require JSON strings. You can use tools like `jq` to construct these strings dynamically in your scripts.

```bash
ZONES=$(jq -n --arg z2 1800 '{"z2_seconds": ($z2|tonumber)}')
repslog set add-cardio <WE_ID> --distance 5 --duration 1500 --hr-zones "$ZONES"
```

## Non-Interactive Errors

When `repslog` is used in a script, it will exit with a non-zero status code if an error occurs, allowing your scripts to handle failures gracefully.

```bash
if ! repslog workout create --date "invalid-date"; then
  echo "Failed to create workout"
  exit 1
fi
```
