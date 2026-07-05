# Statistics & Progress

`repslog` provides several commands to help you visualize your progress over time.

## Personal Records (PRs)

To see your best performances for a specific exercise:

```bash
repslog stats prs --exercise "Squat (Barbell)"
```

This will show your max weight, max reps at certain weights, and other relevant milestones.

## Volume Trends

Track how much work you're doing over time:

```bash
# View volume for the last 30 days
repslog stats volume --period 30d

# View volume for a specific exercise over the last year
repslog stats volume --exercise "Bench Press" --period 1y
```

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

## Weight Progression

See the load history for a specific exercise over time (great for tracking progressive overload on bilateral or unilateral lifts):

```bash
repslog stats weight --exercise "Bulgarian Split Squat"
```

Output is ordered by date and includes the set number, weight, reps, and any notes for that set.

## Set History (per workout)

List every logged set for an exercise across workouts in a date range — useful for seeing push-up (or any exercise) performance session by session without aggregating totals:

```bash
# All push-up sets from workouts in the last 30 days (default)
repslog stats history --exercise "push up"

# Last 7 days, JSON for scripting
repslog stats history --exercise "pike push up" --days 7 --json
```

Each row is one set with the workout date, workout ID, set number, reps (or duration for holds), weight, side, and notes. Only workouts that actually include the exercise appear. Exercise names are matched with a substring search (same as `stats weight`).
