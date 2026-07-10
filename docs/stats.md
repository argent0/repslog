# Statistics & Progress

`repslog` provides several commands to help you visualize your progress over time.

## Personal Records (PRs)

To see your best performances for a specific exercise:

```bash
repslog stats prs --exercise "squat (barbell)"
```

This will show your max weight, max reps at certain weights, and other relevant milestones.

## Volume Trends

Track how much work you're doing over time:

```bash
# View volume for the last 30 days
repslog stats volume --period 30d

# View volume for a specific exercise over the last year
repslog stats volume --exercise "bench press" --period 1y
```

Volume is computed as `reps × load`. For `load_type=external` exercises, load is `--weight`. For `load_type=body_mass` exercises, load is `--weight` (body mass) plus `--external-load` (vest/belt; negative for assistance). Sets logged with `--no-weight-recorded` contribute **zero** volume.

## Training Summary

Get a high-level overview of your training frequency and variety:

```bash
# Summary of the last 30 days
repslog stats summary --days 30
```

The summary includes:
- Total workouts.
- Most frequent exercises.
- Total volume moved (for strength).
- Total distance covered (for cardio).
- Average session feeling and duration.

## Load Progression

See the load history for a specific exercise over time (great for tracking progressive overload):

```bash
repslog stats weight --exercise "bulgarian split squat"
repslog stats weight --exercise "pull up"
```

Output is ordered by date and includes the set number, load, reps, and any notes. For `load_type=body_mass` exercises, load shows body mass and external load (e.g. `82.0 kg BW +5.0 kg`). Sets without recorded body weight are omitted.

## Set History (per workout)

List every logged set for an exercise across workouts in a date range — useful for seeing push-up (or any exercise) performance session by session without aggregating totals:

```bash
# All push-up sets from workouts in the last 30 days (default)
repslog stats history --exercise "push up"

# Last 7 days, JSON for scripting
repslog stats history --exercise "pike push up" --days 7 --json
```

Each row is one set with the workout date, workout ID, set number, reps (or duration for holds), weight, side, and notes. Only workouts that actually include the exercise appear. Exercise names must match the catalog entry exactly (same as `stats weight`); use `repslog exercise search` to find the correct name.
