# Logging Sets

Logging sets is the core of `repslog`. The tool supports traditional strength training, rest-pause clusters, and detailed cardio metrics.

> **Workflow prerequisite:** All set commands require a prior `workout create` and `workout-exercise add` (or use the `set quick` convenience command). See [workouts.md](workouts.md) for the full multi-step flow and data-entry best practices.

## Basic Strength Set

To log a standard set, use the `set add` command with a Workout-Exercise ID (`WE_ID`).

```bash
# Log 10 reps at 100kg with 1 RIR and 5 effective reps
repslog set add <WE_ID> --reps 10 --weight 100 --rir 1.0 --effective-reps 5
```

### Parameters
- `--reps <INT>`: Number of repetitions.
- `--weight <FLOAT>`: Weight in kg.
- `--duration <INT>`: Hold duration in seconds (for static/timed work; omit `--reps`).
- `--rir <FLOAT>`: Reps In Reserve (e.g., 0.0 for failure).
- `--effective-reps <INT>`: Number of stimulating reps.
- `--rpe <FLOAT>`: Rate of Perceived Exertion (1-10).
- `--side <left|right|both>`: Side for unilateral sets (see [Unilateral / Side Tracking](#unilateral--side-tracking)).
- `--notes <TEXT>`: Optional notes for the set.

## Static Holds / Timed Work

For isometric holds and timed exercises, use `--duration` instead of `--reps`:

```bash
WE=$(repslog workout-exercise add <WORKOUT_ID> "Wall Sit")
repslog set add $WE --duration 60 --notes "Wall sit hold"
repslog set add $WE --duration 45 --notes "Second hold"
```

Use `--type "Static Holds"` on `workout create` for session-level filtering. `workout view` displays duration-based sets in the Details column.

## Rest-Pause / Cluster Sets

For advanced hypertrophy training, you can log multiple "mini-sets" in a single command using `set add-cluster`.

```bash
# Log 3 mini-sets (10, 5, 5 reps) with 15s rest between them
repslog set add-cluster <WE_ID> \
  --reps "10,5,5" \
  --weight 100 \
  --rir "0,0,1" \
  --effective-reps "6,4,3" \
  --rest 15
```

This creates three separate set entries grouped together, making it easy to track the total volume and density of the cluster.

## Cardio & Running

Cardio sets support detailed metrics compatible with smart watch exports (like Samsung Health).

```bash
repslog set add-cardio <WE_ID> \
  --distance 7.98 \
  --duration 2701 \
  --avg-heart-rate 154 \
  --max-heart-rate 175 \
  --pace 5.64 \
  --calories 620 \
  --hr-zones '{"z1_seconds": 120, "z2_seconds": 1800, "z3_seconds": 600, "z4_seconds": 120, "z5_seconds": 61}' \
  --laps '[
    {"lap_number":1,"distance_km":1.0,"duration_seconds":332,"pace_min_per_km":5.533},
    {"lap_number":2,"distance_km":1.0,"duration_seconds":345,"pace_min_per_km":5.75}
  ]'
```

### Heart Rate Zones
The `--hr-zones` flag accepts a JSON string containing the time spent in each of the 5 standard zones.

### Laps & Splits
The `--laps` flag accepts a JSON array of lap objects, allowing you to track performance across different segments of your run.

## Convenience Commands

### Quick Add
If you want to add an exercise and its first set in one go:

```bash
repslog set quick <WORKOUT_ID> "Pushups"
```

This is useful for bodyweight exercises where you might only need to log reps later.

## Listing Sets

To see all sets for a specific workout-exercise:

```bash
repslog set list <WE_ID>
```

## Corrections: Update, Delete, Move

Mistakes happen (especially during unilateral sessions). Use these commands to fix without deleting the whole workout.

```bash
repslog set update 287 --reps 10 --weight 20 --notes "Left leg" --side left
repslog set move 287 --to 1
repslog set delete 287 --force   # --force skips the confirmation prompt
```

- `set update` accepts any combination of fields (reps, weight, rir, notes, side, rest, etc.).
- `set delete` asks for confirmation in interactive terminals unless `--force`.
- `set move` reorders within the workout-exercise (renumbers cleanly 1..N).

## Unilateral / Side Tracking

Add `--side left|right|both` when logging.

```bash
repslog set add 84 --reps 6 --weight 20 --side left
repslog set add 84 --reps 6 --weight 20 --side right
```

`workout view` will show a Side column, list sets in logical order (left before right), and print per-side rep totals when sides are present.

For quick symmetric work:

```bash
repslog set add-unilateral 83 --reps "8,10,10,10" --weight 20 --side both
```

This creates left+right pairs (or all left / all right if you specify).

Weight-only sets (no reps) are supported for load tracking / progressive overload:

```bash
repslog set add 99 --weight 22.5
```
