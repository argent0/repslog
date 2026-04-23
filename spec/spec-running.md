**Requirements Document: Making Cardio Training (Especially Running) a First-Class Citizen in `repslog`**

You are extending the existing `repslog` codebase (Rust + SQLite CLI workout tracker with the robust migration system already implemented). Cardio training — **particularly outdoor/indoor running** — must now be treated as a first-class activity alongside strength, calisthenics, and rest-pause sets. The tool must natively and ergonomically support the exact metrics users see in **Samsung Health** (from Galaxy Watch / smart band exports or manual entry).

**Project Context (the haystack)**  
Samsung Health (and the connected smart band/watch) provides the following key data for every running / cardio session:
- Distance (km)
- Duration (total seconds)
- Average heart rate (bpm) — noted as approximate with possible sensor error
- Maximum heart rate (bpm)
- Heart-rate zone breakdown: time spent (or percentage of total time) in each of the **5 standard personalized zones** used by Samsung Health:
  - Zone 1 – Low Intensity / Recovery / Warm-up
  - Zone 2 – Fat Burn
  - Zone 3 – Cardio / Aerobic
  - Zone 4 – Anaerobic / Hard
  - Zone 5 – Peak / Maximum
- Calories burned (optional)
- Average pace (min/km) — derivable from distance + duration but useful to store explicitly for quick reference
- Notes (e.g., “HR sensor had ±5 bpm error on this run”)

Users want to log these values **explicitly** so they can later query volume by zone, average time-in-zone across runs, zone distribution trends, etc.

The solution must remain 100% unified with the existing schema (one `exercise_sets` row can represent an entire run or a single interval). No separate cardio table unless the chosen design absolutely requires it for normalization.

**Strict Non-Negotiable Rules (the needle – follow exactly)**  
This is a serious production tool used for real long-term training data (including LLM-generated or scripted imports). Therefore:
- **Never use defaults** for any cardio-specific field. Every value that has semantic meaning (distance, avg HR, max HR, zone times/percentages, etc.) must be explicitly supplied on the command line.
- If a required cardio argument is missing when the user is logging a cardio-type exercise, the command **must fail immediately** with a clear error telling the user exactly which flags are required.
- All new flags are **long-form only** (`--avg-heart-rate`, `--hr-zones`, etc.) with excellent help text, examples, and validation.
- The CLI must remain fully pipeable and non-interactive friendly.
- All changes **must** be delivered via the existing robust migration system (`repslog migrate`). No direct schema changes in Rust code outside migration files.
- Existing data and all previous commands remain untouched and backward-compatible.

**Required New Capabilities**

1. **Schema Extensions (via new migration)**  
   Add the following new nullable columns to `exercise_sets` (or equivalent normalized design the agent prefers) in migration file `004_add_cardio_support.sql` (or the next sequential number):
   - `avg_heart_rate_bpm` REAL
   - `max_heart_rate_bpm` REAL
   - `heart_rate_zones` TEXT  — **JSON string** containing time spent in each zone (preferred for queryability and future-proofing). Example:  
     `{"z1_seconds": 420, "z2_seconds": 1800, "z3_seconds": 900, "z4_seconds": 120, "z5_seconds": 60, "percentages": {"z1": 12.5, ...}}`  
     (Agent may choose exact JSON shape but it **must** support both absolute seconds and percentages for Samsung Health compatibility.)
   - `avg_pace_min_per_km` REAL (optional but first-class; nullable)
   - `calories_burned` INTEGER (nullable)

   The migration must be fully transactional, idempotent where possible, and recorded in the `migrations` table.

2. **CLI Changes (only extend – never remove or rename existing commands)**  
   - Extend `repslog set add` with new explicit flags:
     - `--avg-heart-rate <bpm>`
     - `--max-heart-rate <bpm>`
     - `--hr-zones <json>` (the exact JSON string above; provide clear example in help)
     - `--pace <min-per-km>` (e.g. 5.2)
     - `--calories <integer>`
   - Add a convenient high-level command `repslog set add-cardio` (or `repslog set add-run`) that requires the full set of cardio fields and automatically sets the exercise to a cardio category if not supplied. This command must still enforce explicit values for every field.
   - Update `set list`, `workout view`, `workout-exercise list`, and any relevant stats commands to display the new cardio fields in tables (use clear formatting, e.g. “Avg HR: 162 bpm | Max: 178 | Zones: Z2 68% Z3 22% …”).
   - Pre-seed (or ensure via `init` / migration) a default “Running” (or “Outdoor Run” / “Treadmill Run”) exercise in the `exercises` table with `category = "cardio"`.

3. **Output & Usability**  
   - Pretty-printed tables must clearly show HR metrics and zone distribution at a glance.
   - Add examples in help text showing a full running log command that matches typical Samsung Health output.
   - Support mixed workouts (e.g., strength + one running finisher) without breaking existing strength workflows.

4. **Integration with Existing Systems**  
   - The new migration must run cleanly on any prior schema version (v1 through current).
   - Every command must still check schema version and prompt `repslog migrate` if needed.
   - All stats commands should eventually be able to aggregate cardio data (e.g., total time in Zone 2 across all runs), but this is future work — the schema must make it possible.

**Final Instructions**  
Implement these cardio / running features on top of the existing codebase while preserving the high-quality, idiomatic Rust, explicitness, error handling, and migration robustness from all previous requirements. Produce the complete updated files (new migration, updated models, CLI definitions, command handlers, etc.).

The resulting tool must make a serious runner feel that `repslog` is just as powerful for zone-based cardio training as it is for effective-reps hypertrophy work.

Begin implementation now.
