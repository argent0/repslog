# Logging Sets

Logging sets is the core of `repslog`. The tool supports traditional strength training, rest-pause clusters, and detailed cardio metrics.

## Basic Strength Set

To log a standard set, use the `set add` command with a Workout-Exercise ID (`WE_ID`).

```bash
# Log 10 reps at 100kg with 1 RIR and 5 effective reps
repslog set add <WE_ID> --reps 10 --weight 100 --rir 1.0 --effective-reps 5
```

### Parameters
- `--reps <INT>`: Number of repetitions.
- `--weight <FLOAT>`: Weight in kg.
- `--rir <FLOAT>`: Reps In Reserve (e.g., 0.0 for failure).
- `--effective-reps <INT>`: Number of stimulating reps.
- `--rpe <FLOAT>`: Rate of Perceived Exertion (1-10).
- `--notes <TEXT>`: Optional notes for the set.

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
