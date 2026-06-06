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
