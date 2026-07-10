# Importing Activities

`repslog` can import running workouts from FIT files exported by devices and apps such as Zepp / Amazfit, Garmin, and similar watches.

## Import a FIT run

The exercise is taken from the FIT file’s `session.sport` field (lowercased). That name **must already exist** in your catalog — import never creates exercises.

```bash
repslog import fit path/to/activity.fit
```

For a typical Amazfit/Zepp run, sport is `running`, which matches the exercise seeded by `repslog init`.

Optional override when your catalog name differs from the FIT sport:

```bash
repslog import fit path/to/activity.fit --exercise "easy run"
```

This single command:

1. Parses the FIT session (distance, duration, HR, calories, cadence, elevation, multi-lap splits)
2. Resolves the exercise from FIT sport (or `--exercise`)
3. Creates a workout (`--type` defaults to `Run`) with the activity start time
4. Attaches the exercise and logs one structured cardio set
5. Stores GPS/HR track samples into `activity_trackpoints` when the file includes a record stream
6. Records import provenance (SHA-256 of the file) so the same file is not imported twice

Example with the sample Amazfit export:

```bash
repslog import fit Zepp20260710164935.fit
repslog workout view <WORKOUT_ID>
```

### Missing exercise

If the resolved name is not in the catalog, import **aborts** (nothing is written) and suggests:

- Similarly named catalog entries (when any exist)
- How to add the exercise, then re-import

```bash
repslog exercise add "running" --category cardio --equipment none --load-type none
repslog import fit path/to/activity.fit
```

### Options

| Flag | Description |
|------|-------------|
| `--exercise <NAME>` | Override catalog exercise (default: FIT `session.sport`, lowercased). Must already exist |
| `--type <LABEL>` | Workout type (default: `Run`) |
| `--notes <TEXT>` | Notes (import provenance is appended) |
| `--force` | Allow re-import of a previously imported file (previous workout is kept; hash lock is cleared) |
| `--hr-zone-bounds A,B,C,D,E` | Upper HR bounds (bpm) for zones 1–5; computes time-in-zone from record samples when the FIT file has no zone data |
| `--dry-run` | Preview without writing |
| `--json` | Machine-readable summary |

```bash
repslog import fit run.fit --notes "easy evening" \
  --hr-zone-bounds 120,140,160,175,190
```

### What is imported

| FIT field | repslog field |
|-----------|----------------|
| session start | `workouts.started_at` |
| session.sport | catalog exercise name (unless `--exercise`) |
| total distance / timer time | set distance + duration, workout duration |
| avg/max heart rate | cardio set HR |
| calories | `calories_burned` |
| avg running cadence | `avg_cadence_spm` |
| total ascent / descent | `total_ascent_m` / `total_descent_m` |
| derived pace | `avg_pace_min_per_km` |
| laps (≥2) | set `laps` JSON (single full-activity lap is skipped) |
| record stream | `activity_trackpoints` (always stored when present) |

HR zones are left empty unless the FIT file includes time-in-zone data or you pass `--hr-zone-bounds`.

### Sanity checks

After parsing, every imported metric is checked against absolute ranges from config (or built-in defaults) **before** any row is written. Out-of-range FIT values fail the command; nothing is stored. See [logging.md](logging.md#sanity-checks) and `repslog config generate`.

### Idempotency

Re-importing the same file bytes fails with a message pointing at the existing workout ID. Use `--force` to clear the hash lock and create a **new** workout (the previous one is not deleted).

### Manual alternative

You can still log runs by hand with `set add-cardio` — see [logging.md](logging.md). Prefer FIT import when you have a device export so metrics stay structured and queryable.
