# Workouts & Sessions

Workouts (also referred to as sessions) are containers for the exercises you perform during a training session. Creating a workout is only **step 1** — you must add exercises and log sets before any training data is recorded. See [logging.md](logging.md) for set-level commands.

## Creating a Workout

To start a new workout, use the `workout create` command. Provide a short `--type`, a `--date` in `YYYY-MM-DD HH:MM:SS` format, and optional `--notes`.

```bash
repslog workout create --type "Legs" --date "2026-04-23 10:00:00" --notes "Focus on form"
```

The command returns the **workout ID**, which you use to add exercises and log sets.

### Full Workflow Overview

Every session follows the same pattern:

1. `workout create` — allocate a session container (this step)
2. `workout-exercise add <ID> "Exercise Name"` — attach exercises (or use `set quick`)
3. `set add`, `set add-cardio`, or `set add-cluster` — log actual training data
4. `workout update <ID> --duration <minutes> --feeling <1-5>` — complete session metadata

Run `repslog workout create --help` for modality-specific guidance inline.

## End-to-End Examples

### Strength / Calisthenics

```bash
ID=$(repslog workout create --type "Calisthenics" --date "2026-07-05 10:00:00")
WE=$(repslog workout-exercise add "$ID" "pull up")
repslog set add "$WE" --reps 8 --weight 82 --phase full --rir 1.0 --effective-reps 5
# weighted vest on later sets
repslog set add "$WE" --reps 5 --weight 82 --phase full --external-load 10
# or clusters (body weight required on clusters too)
repslog set add-cluster "$WE" --reps "3,3,2" --weight 82 --phase full --rir "0,0,1" --rest 15
repslog workout update "$ID" --duration 40 --feeling 4
```

> **Bodyweight rule:** `--weight` is your body mass in kg on every calisthenics set. Use `--external-load` for vests/belts (negative for band assistance). Do not omit weight — `stats volume` ignores sets without it.

### Running / Cardio (recommended structured path)

> **Important:** For runs, always use `set add-cardio` with structured fields. Storing distance, pace, HR zones, or lap splits only in `--notes` disables automatic summaries, lap tables, HR zone bars, and stats aggregation.

```bash
ID=$(repslog workout create --type "Run" --date "2026-07-05 10:00:00" --notes "brief summary")
WE=$(repslog workout-exercise add "$ID" "running")
repslog set add-cardio "$WE" \
  --distance 7.14 \
  --duration 2569 \
  --avg-heart-rate 149 \
  --max-heart-rate 169 \
  --pace 5.99 \
  --calories 231 \
  --hr-zones '{"z1_seconds":15,"z2_seconds":135,"z3_seconds":174,"z4_seconds":2155,"z5_seconds":50}' \
  --laps '[{"lap_number":1,"distance_km":1.0,"duration_seconds":358,"pace_min_per_km":5.97}]'
repslog workout update "$ID" --duration 43 --feeling 4
```

**Never** put structured cardio metrics into `--notes` alone.

### Static Holds / Timed Work

Use `set add --duration <seconds>` (not reps):

```bash
WE=$(repslog workout-exercise add "$ID" "wall sit")
repslog set add "$WE" --duration 60 --weight 82 --phase full --notes "Wall sit hold"
```

## Data Entry Best Practices

### Workout Type Conventions

`--type` is free-form (not enforced), but short canonical labels improve filtering and stats grouping:

- **Recommended:** Calisthenics, Run, Push, Pull, Legs, Upper, Full Body, Static Holds, Yoga, Cardio
- **Avoid:** Long descriptions or sentences in `--type` — put those in `--notes` instead
- **Avoid:** Case variants and synonyms for the same modality (pick one spelling and stick to it)

### Completing the Record

After logging sets, update session-level metadata:

```bash
repslog workout update <WORKOUT_ID> \
  --duration 60 \
  --feeling 5 \
  --notes "Excellent session, felt very strong."
```

`--duration` is in minutes; `--feeling` is 1–5. These fields are often underused but help with training summaries and retrospective analysis.

### Avoiding Exercise Duplicates

Before adding a custom exercise name, search the catalog:

```bash
repslog exercise search "pull"
```

Near-duplicate names (e.g. "Pull Up", "Pull Ups", "pullups") fragment history and stats. Prefer seeded or existing entries when they match your movement.

### Unilateral Training

For left/right work (Bulgarian split squats, pistols, single-leg bridges), tag each set:

```bash
repslog set add <WE_ID> --reps 8 --weight 82 --phase full --side left
repslog set add <WE_ID> --reps 8 --weight 82 --phase full --side right
```

Or use `set add-unilateral` for symmetric pairs. See [logging.md](logging.md) for details.

### Exercise Goals

Set a rep target when adding an exercise to a workout. `workout view` shows Goal vs Actual progress:

```bash
WE=$(repslog workout-exercise add "$ID" "pull up" --goal-reps 50)
repslog set add "$WE" --reps 8 --weight 82 --phase full --rir 1.0
repslog set add "$WE" --reps 7 --weight 82 --phase full --rir 0.5
```

Use `--goal-reps` instead of describing targets only in notes — structured goals are summed against logged sets (per-side aware when `--side` is used).

### LLM / Agent Canonical Run Recipe

When scripting or prompting an LLM to log a run:

1. `repslog workout create --type Run --date YYYY-MM-DD HH:MM:SS [--notes "brief summary"]`
2. `repslog workout-exercise add <ID> running`
3. `repslog set add-cardio <WE_ID> --distance ... --duration ... --avg-heart-rate ... --max-heart-rate ... --pace ... --calories ... --hr-zones '...' --laps '[...]'`
4. `repslog workout update <ID> --duration <minutes> --feeling <1-5>`

Do **not** put distance, pace, HR zones, or lap splits in `--notes` alone.

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

When a cardio set was imported from FIT (or otherwise has rows in `activity_trackpoints`), `workout view` also shows a **TRACK METRICS** section derived on the fly from the sample stream:

- **Moving vs stopped time** and **moving pace**
- Pace distribution (median / range / CV) and **best efforts** (400 m, 1 km, 1 mi, 5 min, 10 min)
- Cadence stats and estimated stride (device cadence units as stored)
- Elevation min/max, smoothed ascent/descent, grade-adjusted pace (GAP), VAM when climb is meaningful
- HR min and cardiac drift %; track-recomputed zones only when stored zones are empty and DOB was snapshotted on the set
- Route summary (loop vs point-to-point, GPS path length, start–end gap)

If the device did not store multi-lap splits, a **COMPUTED KM SPLITS** table is shown from the distance series. Device `laps` still take priority when present.

JSON (`repslog workout view <id> --json`) exposes the same payload as:

- `cardio_summary.track` — metrics for the first cardio set that has trackpoints
- `exercises[].sets[].track_metrics` — per-set metrics (source of truth)

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
repslog workout-exercise add <WORKOUT_ID> "squat (barbell)"
```

This returns a **Workout-Exercise ID (WE_ID)**, which is used to log individual sets. See [logging.md](logging.md) for set commands.

## Deleting a Workout

If you made a mistake, you can delete a workout and all its associated sets:

```bash
repslog workout delete <WORKOUT_ID>
```