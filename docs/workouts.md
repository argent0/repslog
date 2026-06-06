# Workouts & Sessions

Workouts (also referred to as sessions) are containers for the exercises you perform during a training session.

## Creating a Workout

To start a new workout, use the `workout create` command. It's recommended to provide a type and date.

```bash
repslog workout create --type "Legs" --date "2026-04-23" --notes "Focus on form"
```

The command will return the **ID** of the newly created workout, which you will use to add exercises.

## Listing Workouts

To see your recent training history:

```bash
# List workouts from the last 7 days
repslog workout list --days 7

# List the last 10 workouts
repslog workout list --limit 10
```

## Viewing Workout Details

To see everything you did in a specific workout:

```bash
repslog workout view <WORKOUT_ID>
```

For cardio workouts, this view includes automated summaries, pace calculations, and heart rate zone bars.

When logging unilateral work with `--side`, the view groups or clearly labels Left/Right sets, shows per-side rep totals, and respects logical ordering. Exercise-level notes (on the workout-exercise) and goal_reps (if set) are also displayed with actual vs goal progress.

## Updating a Workout

You can update workout details like duration, feeling (1-5), or notes at any time:

```bash
repslog workout update <WORKOUT_ID> \
  --duration 60 \
  --feeling 5 \
  --notes "Excellent session, felt very strong."
```

## Adding Exercises to a Workout

Before you can log sets, you must add an exercise to your workout:

```bash
repslog workout-exercise add <WORKOUT_ID> "Squat (Barbell)"
```

This returns a **Workout-Exercise ID (WE_ID)**, which is used to log individual sets.

## Deleting a Workout

If you made a mistake, you can delete a workout and all its associated sets:

```bash
repslog workout delete <WORKOUT_ID>
```
