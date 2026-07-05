# Report: `workout create` Documentation Gap & Data Entry Analysis

**Date:** 2026-07-05  
**Spec:** `spec/2026-07-05-workout-create-doc.md`  
**Database analyzed:** `repslog.db` (33 workouts, 44 exercises, ~500+ sets)

## 1. Executive Summary

`repslog workout create --help` tells a user (or LLM) almost nothing about how to log different workout modalities. It exposes three optional flags and a mandatory date, with **no descriptions, no examples, and no mention of the follow-on commands** required to actually record training data.

The tool does **not** restrict `workout_type` values — it accepts any free-form string. The sample database shows this freedom is frequently misused: inconsistent labels, descriptive text stuffed into the type field, and — most critically — runs logged as prose in `notes` instead of structured cardio sets.

Cardio support in `repslog` is mature (`set add-cardio` with distance, duration, HR zones, laps, calories), but that capability is invisible from `workout create --help` and underused in practice.

## 2. Current `workout create --help` Output

```
Create a new workout

Usage: repslog workout create [OPTIONS] --date <DATE>

Options:
      --db <PATH>            Path to SQLite database file (overrides default XDG location)
  -w, --type <WORKOUT_TYPE>  
      --json                 Output results in JSON format instead of human-readable tables
  -n, --notes <NOTES>        
  -d, --date <DATE>          
      --dry-run              Show what would be created (no changes)
  -h, --help                 Print help
```

### Gaps for LLM / script consumers

| Issue | Impact |
|-------|--------|
| `--type` has no help text or doc comment in `src/cli.rs` | LLM cannot infer valid or conventional values |
| No workflow description | User may think `workout create` alone completes logging |
| No cross-references | `workout-exercise add`, `set add`, `set add-cardio`, `set add-cluster` are undocumented here |
| No modality-specific examples | Runs, strength, static holds each need different set commands |
| `--date` format not stated in help | Only validated at runtime (`YYYY-MM-DD` or `YYYY-MM-DD HH:MM:SS`) |
| Contrast with `set add-cardio --help` | That command includes a full inline example; `workout create` does not |

The existing user-facing docs (`docs/workouts.md`, `docs/logging.md`, `README.md`) describe the multi-step flow, but they are **not surfaced in CLI help**. An LLM invoking `--help` in isolation gets no guidance.

## 3. Does `repslog` Restrict Workout Type?

**No.** Workout type is a free-form optional string.

### Code evidence

- **CLI:** `workout_type: Option<String>` in `WorkoutAction::Create` (`src/cli.rs`)
- **Repository:** Inserted directly into SQLite with no validation (`src/repository.rs`)
- **Schema:** `workout_type TEXT` with a comment listing suggestions, not constraints (`migrations/001_initial_schema.sql`)

```sql
workout_type TEXT,  -- "push", "pull", "legs", "full_body", "run", "yoga", etc.
```

There is no enum, no `value_parser`, no CHECK constraint, and no normalization (case, synonyms).

### Observed values in `repslog.db`

| `workout_type` | Count |
|----------------|-------|
| Calisthenics | 17 |
| Running | 3 |
| Run | 3 |
| Upper | 2 |
| Legs | 2 |
| strength | 1 |
| calisthenics | 1 |
| Upper Body | 1 |
| Static Holds | 1 |
| Cardio | 1 |
| `Run (steady 8 km effort with strong final kick)` | 1 |

**11 distinct values** across 33 workouts, including case variants (`Calisthenics` vs `calisthenics`) and synonyms (`Run`, `Running`, `Cardio`). Workout #11 stores a full sentence in the type field — clear evidence that unconstrained strings invite misuse.

## 4. Correct Workflows by Modality

`workout create` only allocates a container and returns an ID. All training data lives in downstream tables.

### Strength / Calisthenics

```bash
ID=$(repslog workout create --type "Calisthenics" --date "2026-07-05")
WE=$(repslog workout-exercise add "$ID" "Pull Ups")
repslog set add "$WE" --reps 8 --rir 1.0 --effective-reps 5
# Clusters:
repslog set add-cluster "$WE" --reps "3,3,2" --rir "0,0,1" --rest 15
repslog workout update "$ID" --duration 40 --feeling 4
```

### Running / Cardio

```bash
ID=$(repslog workout create --type "Run" --date "2026-07-05")
WE=$(repslog workout-exercise add "$ID" "Running")
repslog set add-cardio "$WE" \
  --distance 7.14 \
  --duration 2569 \
  --avg-heart-rate 149 \
  --max-heart-rate 169 \
  --pace 5.99 \
  --calories 231 \
  --hr-zones '{"z1_seconds":15,"z2_seconds":135,"z3_seconds":174,"z4_seconds":2155,"z5_seconds":50}' \
  --laps '[{"lap_number":1,"distance_km":1.0,"duration_seconds":358,"pace_min_per_km":5.97}, ...]'
```

After this, `workout list` shows an auto-generated cardio summary (distance, time, pace, HR). `workout view` renders HR zone bars and a lap table.

### Static Holds / Timed Work

Use `set add --duration <seconds>` (not reps):

```bash
repslog set add "$WE" --duration 60 --notes "Wall sit hold"
```

Workout #25 in the sample DB demonstrates this pattern correctly.

## 5. Data Entry Issues in `repslog.db`

### 5.1 Critical: Run logged without structured data (Workout #37)

| Field | Value |
|-------|-------|
| Type | `Run` |
| Exercises | **0** |
| Sets | **0** |
| Notes | Full run summary as prose (distance, pace, HR, laps, zones) |

`workout view 37` shows only the notes block — no CARDIO SUMMARY, no lap table, no HR zone bar. The data is **unqueryable** for stats (`stats` commands aggregate from `exercise_sets`, not notes).

**Fix:** Add `Running` exercise + `set add-cardio` with structured fields, or delete and re-log.

### 5.2 Cardio workouts that *do* use structured data

Workouts #11, #12, #17, #21, #26, #42, #45 all have a `Running` exercise with one `add-cardio`-style set (distance, duration, HR, zones, laps populated). These display correctly in `workout view` and produce rich list summaries.

### 5.3 Redundant duplication in notes

Workouts #42 and #45 store the **same** lap splits, HR zones, and overview metrics in both:
- structured `exercise_sets` fields (correct), and
- lengthy `workouts.notes` (redundant)

This is not harmful for display (structured data drives the summary), but it wastes tokens when an LLM reads the DB and creates maintenance burden if metrics are ever corrected.

### 5.4 Misused `workout_type` field (Workout #11)

```
workout_type = "Run (steady 8 km effort with strong final kick)"
```

The type field was used as a session title. Conventional value would be `Run` with the description in `--notes`.

### 5.5 Inconsistent type vocabulary

Three different labels for the same modality: `Run`, `Running`, `Cardio`. This fragments filtering and stats grouping. A convention like `Run` (or normalized lowercase `run`) would improve consistency.

### 5.6 Underused workout-level metadata

| Field | Used | Total | Rate |
|-------|------|-------|------|
| `duration_minutes` | 16 | 33 | 48% |
| `overall_feeling` (1–5) | 4 | 33 | 12% |

Most runs omit `duration_minutes` even when `duration_seconds` exists on the cardio set. Running `workout update --duration N` after logging would align workout-level duration with set data.

### 5.7 Duplicate exercise catalog entries

The database contains near-duplicate custom exercises that split history:

| Canonical (seeded) | Duplicates (custom) |
|--------------------|---------------------|
| Pullups | Pull Up, Pull Ups |
| Dips | Ring Dips (acceptable variant) |
| Bulgarian Split Squat | Bulgarian split squat |
| Nordic Hamstring Curl | Nordic curl |
| Assisted Pistol Squat | Pistol squat, Pistol Squat (eccentric only) |

Using `repslog exercise search` before `exercise add` would reduce fragmentation. 12 similarly-named entries inflate the catalog without adding information.

### 5.8 Advanced features available but unused

| Feature | Sets/entries using it | Notes |
|---------|----------------------|-------|
| `set add-cluster` | 132 sets | **Good** — heavily used for rest-pause |
| `effective_reps` | 135 sets | **Good** |
| `rir` | 181 sets | **Good** |
| `--side` (unilateral) | **0 sets** | Leg workouts (#23, #32, #35, #38, #44) log left/right work without side tagging |
| `goal_reps` on workout_exercises | **0** | Feature unused despite plans mentioning goals in notes |
| `set add` with `--duration` | Used in #25 | **Good** for static holds |

Unilateral leg sessions (Bulgarian squats, pistols, single-leg bridges) would benefit from `--side left|right` per the unilateral feature docs, enabling per-side totals in `workout view`.

### 5.9 Workout #47: incomplete logging

`strength` workout with 5 exercises but only **7 sets** total — likely an in-progress or abandoned session relative to the detailed protocol described in notes.

## 6. What `--help` Should Convey (Recommendations)

These are documentation/CLI improvements, not code changes made in this report:

1. **Expand `Create` doc comment** in `src/cli.rs` with a multi-step overview and per-modality examples (mirror `set add-cardio` style).
2. **Add `long_help`** or field-level `help = "..."` on `--type` listing conventional values (`Push`, `Pull`, `Legs`, `Run`, `Calisthenics`, `Static Holds`, etc.) while noting they are suggestions, not enforced.
3. **Document the date format** on `--date` via clap `value_parser` help or `help = "YYYY-MM-DD"`.
4. **Add `after_help`** block pointing to `docs/workouts.md` and `docs/logging.md`.
5. **Consider a `repslog workout create --help=cardio`** or subcommand examples — or a top-level `repslog guide run` — for LLM discoverability.

### Suggested canonical run recipe for LLM system prompts

```
1. repslog workout create --type Run --date YYYY-MM-DD [--notes "brief summary"]
2. repslog workout-exercise add <ID> Running
3. repslog set add-cardio <WE_ID> --distance ... --duration ... --avg-heart-rate ... --max-heart-rate ... --pace ... --calories ... --hr-zones '...' --laps '[...]'
4. repslog workout update <ID> --duration <minutes> --feeling <1-5>
```

**Do not** put distance, pace, HR zones, or lap splits in `--notes` alone.

## 7. Summary Table: Facility Utilization

| `repslog` facility | Properly used? | Evidence |
|--------------------|----------------|----------|
| `workout create` | Yes | All 33 workouts created |
| `workout-exercise add` | Mostly | 32/33 have exercises; #37 missing |
| `set add` (strength) | Yes | Hundreds of rep-based sets |
| `set add-cluster` | Yes | 132 cluster sets |
| `set add-cardio` | Partial | 7/8 run-type workouts structured; #37 missed |
| `set add --duration` (holds) | Yes | Workout #25 |
| `--side` unilateral | No | 0 usage despite unilateral sessions |
| `goal_reps` | No | 0 usage |
| `overall_feeling` | Rare | 4/33 workouts |
| `workout update --duration` | Partial | 48% have duration |
| Exercise deduplication | No | 12 duplicate name variants |
| Consistent `workout_type` | No | 11 distinct values, 1 misused as title |

## 8. Conclusion

`repslog` does not restrict workout types — and the sample database shows why some soft guidance would help. The primary gap for LLM consumers is not missing features but **missing discoverability**: `workout create --help` is the entry point, yet it describes none of the multi-step logging pipeline or the cardio-specific `set add-cardio` command that unlocks pace, HR zone, and lap analytics.

The most significant data quality issue is Workout #37, a run stored entirely in notes with zero exercises or sets, making it invisible to structured queries and stats. Seven other runs demonstrate the correct pattern and produce the rich CLI output `repslog` was designed for.