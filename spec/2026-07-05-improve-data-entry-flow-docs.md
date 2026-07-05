# Spec: Improve Data Entry Flow Documentation

**Date:** 2026-07-05
**Status:** Proposed
**Related:** Report `2026-07-05-workout-create-doc.md` (analysis of documentation gaps and data quality issues in `repslog.db`)
**Priority:** High (for LLM discoverability and long-term data hygiene)

## 1. Problem Statement

The `repslog` CLI provides powerful, structured support for strength, calisthenics, cardio, static holds, and unilateral training. However, the primary entry point command `workout create --help` is minimal and does not surface:

- The multi-step workflow required to actually log training data.
- Modality-specific guidance (especially critical `set add-cardio` for runs).
- Conventional values and best practices for `--type`.
- Cross-references to detailed docs.

As a result, the sample database exhibits recurring data quality problems:
- Structured cardio metrics stored only in free-text `notes` (unqueryable, no rich `workout view` output).
- Inconsistent / misused `workout_type` (free-form prose, case variants, synonyms).
- Under-populated workout metadata (`duration_minutes`, `overall_feeling`).
- Duplicate exercise names fragmenting history.
- Unused advanced features (`--side`, `goal_reps`).

These issues make stats, PRs, and LLM-driven analysis less reliable and increase future cleanup burden.

## 2. Goal

Improve **documentation and discoverability** (CLI help text + user docs) so that both human users and LLM agents are guided toward correct, structured data entry patterns from the moment they invoke `workout create`. This directly mitigates the observed misuse patterns without changing runtime behavior or adding enforcement.

## 3. Proposed Changes

### 3.1 Update `src/cli.rs` — `WorkoutAction::Create`

**Current:**
```rust
/// Create a new workout
Create {
    #[arg(short, long = "type")]
    workout_type: Option<String>,
    ...
}
```

**Proposed:**

Replace with rich documentation modeled after `set add-cardio` and `set add-cluster`:

```rust
/// Create a new workout (training session container).
///
/// This is step 1 of logging any session. You must follow up with:
///   1. `repslog workout-exercise add <ID> "Exercise Name"`  (or use `set quick`)
///   2. `repslog set add`, `set add-cardio`, or `set add-cluster` to log data
///
/// For **Running / Cardio** (strongly recommended):
///   - Use `--type Run` (or "Running")
///   - Add exercise "Running"
///   - Use `set add-cardio` with structured --distance, --duration, --avg-heart-rate,
///     --max-heart-rate, --pace, --calories, --hr-zones JSON, --laps JSON
///   - Do NOT store distance/pace/HR/laps/zones only in --notes (data becomes unqueryable)
///
/// Conventional `--type` suggestions (free-form; not enforced):
///   Calisthenics, Run, Push, Pull, Legs, Upper, Full Body, Static Holds, Yoga, Cardio
///   Avoid long descriptions or sentences in --type — put those in --notes instead.
///
/// Date format: YYYY-MM-DD or YYYY-MM-DD HH:MM:SS (validated at runtime)
///
/// After logging sets, run:
///   `repslog workout update <ID> --duration <minutes> --feeling <1-5>`
///
/// See also: docs/workouts.md, docs/logging.md, and `repslog set add-cardio --help`
Create {
    #[arg(short, long = "type", help = "Workout type (e.g. Calisthenics, Run, Push, Legs). Free-form suggestions only.")]
    workout_type: Option<String>,
    #[arg(short, long, help = "Optional session notes (avoid putting structured metrics here for cardio)")]
    notes: Option<String>,
    #[arg(short, long, help = "Date in YYYY-MM-DD or YYYY-MM-DD HH:MM:SS format")]
    date: String,
    #[arg(long)]
    dry_run: bool,
}
```

Additionally, add to the `Workout` subcommand:
```rust
#[command(after_help = "Full workflow and modality examples: see docs/workouts.md and docs/logging.md\nCardio best practices: always use structured set add-cardio for queryable stats and rich views.")]
```

### 3.2 Enhance `docs/workouts.md`

Expand the "Creating a Workout" section and add a new "Data Entry Best Practices" section that directly addresses every issue from the analysis report.

**Key additions:**

- Full end-to-end examples for **Strength/Calisthenics** and **Running/Cardio** (copy/adapt the canonical recipes from the source report).
- Explicit warning box or note: "For runs, always use `set add-cardio` — storing everything in notes (as seen in some historical entries) disables automatic summaries, lap tables, HR zone bars, and stats aggregation."
- "Workout Type Conventions" subsection recommending a short canonical list and discouraging prose in the type field.
- "Completing the Record" subsection: always follow up with `workout update --duration --feeling`.
- "Avoiding Exercise Duplicates": recommend `exercise search` before adding custom variants.
- "Unilateral Training": remind to use `--side left|right|both` (and `set add-unilateral`).
- Link to the suggested canonical LLM prompt recipe for run logging.

Also update the intro and cross-reference `logging.md` more prominently.

### 3.3 Minor Updates to `docs/logging.md` and `README.md`

- In `logging.md`, add a short "Workflow Prerequisite" note at top: "All set commands require a prior `workout create` + `workout-exercise add` (or use the `set quick` convenience command)."
- In `README.md` usage guide, ensure the cardio example is presented as the **recommended structured path**, and add a one-sentence pointer to the new best-practices section in docs.
- Keep examples in sync so `./docs/verify_examples.sh` continues to pass.

### 3.4 (Optional but Recommended) Non-Doc Enhancements

While the primary focus is documentation:
- Consider adding a lightweight `value_parser` or `possible_values` hint (even if not strict) for `--type` in a future minor release.
- Or a `repslog guide cardio` / `repslog workout create --help=cardio` style subcommand for even better LLM discoverability (out of scope for this doc-focused proposal).

## 4. Canonical Recipes to Embed

### Strength / Calisthenics
```bash
ID=$(repslog workout create --type "Calisthenics" --date "2026-07-05")
WE=$(repslog workout-exercise add "$ID" "Pull Ups")
repslog set add "$WE" --reps 8 --rir 1.0 --effective-reps 5
# or clusters
repslog set add-cluster "$WE" --reps "3,3,2" --rir "0,0,1" --rest 15
repslog workout update "$ID" --duration 40 --feeling 4
```

### Running / Cardio (LLM-friendly canonical form)
```bash
ID=$(repslog workout create --type "Run" --date "2026-07-05" --notes "brief summary")
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
repslog workout update "$ID" --duration 43 --feeling 4
```
**Never** put the structured fields into `--notes` alone.

## 5. Success Metrics

After implementation:
- `repslog workout create --help` output contains workflow overview, modality examples, and best-practice warnings.
- New runs logged via LLM or interactive use follow the structured `add-cardio` path (zero new "notes-only" cardio entries).
- Higher adoption of `workout update --duration` and `--feeling`, and `--side` on unilateral sessions.
- Fewer duplicate exercise names (measured via `exercise list`).
- Documentation examples remain verifiable.

## 6. Implementation Plan

1. Edit `src/cli.rs` (Create variant + after_help).
2. Rewrite/expand relevant sections of `docs/workouts.md` (add Best Practices subsection).
3. Minor sync edits to `README.md` and `docs/logging.md`.
4. `cargo fmt && cargo clippy`.
5. `cargo build && ./docs/verify_examples.sh`.
6. Commit with message referencing this spec and the source analysis report.
7. (Future) Consider backfilling/fixing historical bad entries in `repslog.db` (e.g. Workout #37) as a separate data-cleanup task.

## 7. Out of Scope

- Adding runtime validation or CHECK constraints on `workout_type`.
- Changing DB schema or adding new commands.
- Automatic normalization of existing data.

These can be considered later if documentation improvements prove insufficient.

## 8. Conclusion

By making the **correct data entry flow** visible and self-documenting at the CLI entry point and in the primary user guides, we prevent the recurrence of the issues observed in the analyzed database. This improves immediate usability for humans and LLMs, long-term data quality for stats/PRs/views, and reduces technical debt from unstructured or duplicated records.

This is a pure documentation + help-text improvement that aligns with repslog's existing philosophy of flexibility while gently guiding users toward the rich structured features the tool already provides.