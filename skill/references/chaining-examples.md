# Chaining Examples

`repslog` is designed to be scriptable. Many commands output the ID of the created resource on the last line, making it easy to chain commands using standard shell tools.

## Basic Chaining with `tail` and `xargs`

### Create Workout and Add Exercise
```bash
# 1. Create workout and capture ID
WORKOUT_ID=$(repslog workout create --date 2026-04-27 --type strength | tail -n 1)

# 2. Add exercise using the captured ID
WE_ID=$(repslog workout-exercise add $WORKOUT_ID Pushups | tail -n 1)

# 3. Add a set
repslog set add $WE_ID --reps 15 --rir 0
```

## Chaining with Stdin Piping
`repslog` subcommands that require an ID (like `workout-exercise add` or `set add`) can read the ID from `stdin` if it's not provided as an argument.

### One-Liner: Workout -> Exercise
```bash
repslog workout create --date 2026-04-27 | repslog workout-exercise add Pushups
```

### One-Liner: Workout -> Exercise -> Set
```bash
repslog workout create --date 2026-04-27 | \
  repslog workout-exercise add Pullups | \
  repslog set add --reps 10 --weight 0
```

## Advanced Scripting
You can build complex logging scripts that handle multiple exercises.

```bash
#!/bin/bash
DATE=$(date +%Y-%m-%d)

# Start workout
WID=$(repslog workout create --date $DATE --notes "Morning Session" | tail -n 1)

# Add Bench Press
WEID=$(repslog workout-exercise add $WID "Bench Press" | tail -n 1)
repslog set add $WEID --reps 10 --weight 60
repslog set add $WEID --reps 8 --weight 65
repslog set add $WEID --reps 6 --weight 70

# Add Squats
WEID=$(repslog workout-exercise add $WID "Squat (Barbell)" | tail -n 1)
repslog set add $WEID --reps 12 --weight 80
repslog set add $WEID --reps 10 --weight 90
```
*(Note: Always verify exercise names or IDs before running automated scripts.)*
