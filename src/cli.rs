use crate::models::{HeartRateZones, Laps};
use crate::phase;
use clap::{Parser, Subcommand};

fn parse_phase(s: &str) -> Result<String, String> {
    phase::normalize_phase(s)
        .map(|p| p.to_string())
        .map_err(|e| e.to_string())
}

#[derive(Parser)]
#[command(name = "repslog")]
#[command(about = "A Linux-first workout tracker", long_about = None)]
pub struct Cli {
    /// Path to SQLite database file (overrides default XDG location)
    #[arg(long, global = true, value_name = "PATH")]
    pub db: Option<String>,

    /// Path to config.toml (default: ~/.config/repslog/config.toml)
    #[arg(long, global = true, value_name = "PATH")]
    pub config: Option<String>,

    /// Output results in JSON format instead of human-readable tables
    #[arg(long, global = true)]
    pub json: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Exercise management
    Exercise {
        #[command(subcommand)]
        action: ExerciseAction,
    },
    /// Workout management
    #[command(
        after_help = "Full workflow and modality examples: see docs/workouts.md and docs/logging.md\n\
                      Cardio best practices: always use structured set add-cardio for queryable stats and rich views."
    )]
    Workout {
        #[command(subcommand)]
        action: WorkoutAction,
    },
    /// Alias for workout management
    Session {
        #[command(subcommand)]
        action: WorkoutAction,
    },
    /// Manage exercises within a workout
    #[command(name = "workout-exercise")]
    WorkoutExercise {
        #[command(subcommand)]
        action: WorkoutExerciseAction,
    },
    /// Manage sets within a workout exercise
    Set {
        #[command(subcommand)]
        action: SetAction,
    },
    /// View statistics
    Stats {
        #[command(subcommand)]
        action: StatsAction,
    },
    /// Database migrations
    Migrate {
        /// Show current version vs. latest available (no changes)
        #[arg(short, long)]
        status: bool,
        /// Show exactly what would be applied (no changes)
        #[arg(short, long)]
        dry_run: bool,
        /// Force re-execution of migrations even if already applied
        #[arg(short, long)]
        force: bool,
    },
    /// Initialize database and seed default exercises
    Init {
        /// Show what would be initialized (no changes)
        #[arg(long)]
        dry_run: bool,
    },
    /// Import external activity files (FIT, etc.)
    Import {
        #[command(subcommand)]
        action: ImportAction,
    },
    /// Configuration file management
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
}

#[derive(Subcommand)]
pub enum ConfigAction {
    /// Write a config file with default sanity limits.
    ///
    /// Default path: ~/.config/repslog/config.toml
    /// Refuses to overwrite an existing file unless --force is set.
    ///
    /// Examples:
    ///   repslog config generate
    ///   repslog config generate --path /tmp/repslog.toml
    ///   repslog config generate --force
    Generate {
        /// Output path (default: XDG config location)
        #[arg(long, value_name = "PATH")]
        path: Option<String>,
        /// Overwrite an existing config file
        #[arg(long)]
        force: bool,
    },
    /// Show the resolved config path and whether a file is loaded
    Path {
        /// Path to inspect (default: XDG config location)
        #[arg(long, value_name = "PATH")]
        path: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum ImportAction {
    /// Import a running workout from a Garmin/Zepp/Amazfit FIT file.
    ///
    /// Creates a workout, attaches a catalog exercise, and logs one cardio set
    /// with distance, duration, HR, pace, calories, cadence, and elevation when present.
    ///
    /// The exercise is taken from FIT session.sport (e.g. "running") unless overridden.
    /// The catalog exercise must already exist — import will not create it.
    ///
    /// Example:
    ///   repslog import fit Zepp20260710164935.fit
    Fit {
        /// Path to the .fit file
        path: String,
        /// Override exercise name (default: FIT session.sport, lowercased). Must exist in catalog.
        #[arg(long)]
        exercise: Option<String>,
        /// Workout type label (default: Run)
        #[arg(long = "type")]
        workout_type: Option<String>,
        /// Optional notes (import provenance is appended)
        #[arg(short, long)]
        notes: Option<String>,
        /// Allow re-import of a previously imported file (keeps the old workout; drops hash lock)
        #[arg(long)]
        force: bool,
        /// Persist GPS/HR track samples into activity_trackpoints
        #[arg(long = "store-track")]
        store_track: bool,
        /// Optional HR zone upper bounds in bpm for zones 1-5 (comma-separated).
        /// Example: 120,140,160,175,190 — computes time-in-zone from record samples.
        #[arg(long = "hr-zone-bounds", value_parser = parse_hr_zone_bounds)]
        hr_zone_bounds: Option<[f64; 5]>,
        /// Show what would be imported (no changes)
        #[arg(long)]
        dry_run: bool,
    },
}

fn parse_hr_zone_bounds(s: &str) -> std::result::Result<[f64; 5], String> {
    let parts: Vec<&str> = s
        .split(',')
        .map(str::trim)
        .filter(|p| !p.is_empty())
        .collect();
    if parts.len() != 5 {
        return Err(
            "expected 5 comma-separated bpm bounds for zones 1-5 (e.g. 120,140,160,175,190)".into(),
        );
    }
    let mut out = [0.0; 5];
    for (i, p) in parts.iter().enumerate() {
        out[i] = p
            .parse::<f64>()
            .map_err(|_| format!("invalid bpm value '{}'", p))?;
        if out[i] <= 0.0 {
            return Err(format!("zone bound {} must be > 0", i + 1));
        }
        if i > 0 && out[i] < out[i - 1] {
            return Err("zone bounds must be non-decreasing".into());
        }
    }
    Ok(out)
}

#[derive(Subcommand)]
pub enum ExerciseAction {
    /// List all exercises
    List {
        #[arg(short, long)]
        search: Option<String>,
        #[arg(short, long)]
        category: Option<String>,
    },
    /// Add a custom exercise to the catalog.
    ///
    /// Names must be lowercase and singular (e.g. `pull up`, not `Pull Ups` or `pull ups`).
    /// Do not embed rep phase in the name (e.g. `pistol squat (eccentric only)`); use `--phase`
    /// on `set add` instead. Pass `--allow-phase-in-name` only for legacy cleanup.
    /// Search the catalog first to avoid near-duplicates that fragment history and stats.
    ///
    /// Examples:
    ///
    ///   repslog exercise search "pull"
    ///
    ///   repslog exercise add "ring dip" --category calisthenics --equipment rings --load-type body_mass
    ///
    ///   repslog exercise add "bulgarian split squat" --category strength --equipment dumbbell --load-type external
    Add {
        /// Exercise name (lowercase, singular; use spaces between words)
        #[arg(help = "Exercise name (lowercase, singular, e.g. pull up)")]
        name: String,
        #[arg(short, long)]
        category: String,
        /// Apparatus used (e.g. barbell, rings, parallel bars, none)
        #[arg(short, long)]
        equipment: Option<String>,
        /// How --weight is interpreted: body_mass, external, or none
        #[arg(long = "load-type")]
        load_type: Option<String>,
        #[arg(short, long)]
        muscles: Option<String>,
        #[arg(short, long)]
        description: Option<String>,
        /// Allow eccentric/concentric in the exercise name (not recommended)
        #[arg(long = "allow-phase-in-name")]
        allow_phase_in_name: bool,
        /// Show what would be added (no changes)
        #[arg(long)]
        dry_run: bool,
    },
    /// Update exercise metadata (equipment, load type, category, etc.)
    ///
    /// Example: repslog exercise update "ring dip" --equipment rings --load-type body_mass
    Update {
        exercise_id_or_name: String,
        #[arg(short, long)]
        category: Option<String>,
        #[arg(short, long)]
        equipment: Option<String>,
        /// Clear apparatus (sets equipment to NULL)
        #[arg(long = "clear-equipment")]
        clear_equipment: bool,
        #[arg(long = "load-type")]
        load_type: Option<String>,
        #[arg(short, long)]
        muscles: Option<String>,
        #[arg(short, long)]
        description: Option<String>,
        /// Show what would be updated (no changes)
        #[arg(long)]
        dry_run: bool,
    },
    /// Search for exercises
    Search { term: String },
}

#[derive(Subcommand)]
pub enum WorkoutAction {
    /// Create a new workout (training session container).
    ///
    /// This is step 1 of logging any session. You must follow up with:
    ///   1. `repslog workout-exercise add <ID> "Exercise Name"`  (or use `set quick`)
    ///   2. `repslog set add`, `set add-cardio`, or `set add-cluster` to log data
    ///
    /// For **Running / Cardio** (strongly recommended):
    ///   - Use `--type Run`
    ///   - Add exercise "running"
    ///   - Use `set add-cardio` with structured --distance, --duration, --avg-heart-rate,
    ///     --max-heart-rate, --pace, --calories, --hr-zones JSON, --laps JSON
    ///   - Do NOT store distance/pace/HR/laps/zones only in --notes (data becomes unqueryable)
    ///
    /// Conventional `--type` suggestions (free-form; not enforced):
    ///   Calisthenics, Run, Push, Pull, Legs, Upper, Full Body, Static Holds, Yoga, Cardio
    ///   Avoid long descriptions or sentences in --type — put those in --notes instead.
    ///
    /// Date format: YYYY-MM-DD HH:MM:SS (validated at runtime)
    ///
    /// For **Unilateral** (left/right work):
    ///   - Tag each set with `--side left` or `--side right` on `set add` or `set add-unilateral`
    ///
    /// For **Static Holds / Timed Work**:
    ///   - Use `--type Static Holds`
    ///   - Use `set add --duration <seconds>` instead of --reps
    ///
    /// After logging sets, run:
    ///   `repslog workout update <ID> --duration <minutes> --feeling <1-5>`
    ///
    /// See also: docs/workouts.md, docs/logging.md, and `repslog set add-cardio --help`
    Create {
        #[arg(
            short,
            long = "type",
            help = "Workout type (e.g. Calisthenics, Run, Push, Legs). Free-form suggestions only."
        )]
        workout_type: Option<String>,
        #[arg(
            short,
            long,
            help = "Optional session notes (avoid putting structured metrics here for cardio)"
        )]
        notes: Option<String>,
        #[arg(short, long, help = "Date/time in YYYY-MM-DD HH:MM:SS format")]
        date: String,
        /// Show what would be created (no changes)
        #[arg(long)]
        dry_run: bool,
    },
    /// List workouts
    List {
        #[arg(short, long, default_value_t = 10)]
        limit: i64,
        #[arg(short, long)]
        days: Option<i64>,
    },
    /// View details of a specific workout
    View { workout_id: i64 },
    /// Update a workout
    Update {
        workout_id: String,
        #[arg(short, long = "type")]
        workout_type: Option<String>,
        #[arg(short, long)]
        notes: Option<String>,
        #[arg(short, long)]
        duration: Option<i32>,
        #[arg(short, long)]
        feeling: Option<i32>,
        #[arg(short, long)]
        date: Option<String>,
        /// Show what would be updated (no changes)
        #[arg(long)]
        dry_run: bool,
    },
    /// Delete a workout
    Delete {
        workout_id: String,
        /// Show what would be deleted (no changes)
        #[arg(long)]
        dry_run: bool,
    },
}

#[derive(Subcommand)]
pub enum WorkoutExerciseAction {
    /// Add an exercise to a workout.
    ///
    /// Example: repslog workout-exercise add 1 "Pull Ups" --goal-reps 50
    Add {
        workout_id: String,
        exercise_id_or_name: String,
        #[arg(short, long)]
        order: Option<i32>,
        /// Target rep count for this exercise (shown as Goal vs Actual in workout view)
        #[arg(long = "goal-reps")]
        goal_reps: Option<i32>,
        /// Show what would be added (no changes)
        #[arg(long)]
        dry_run: bool,
    },
    /// List exercises in a workout
    List { workout_id: i64 },
}

#[derive(Subcommand)]
pub enum SetAction {
    /// Add a set to a workout exercise.
    ///
    /// Barbell/dumbbell: `--weight` is the load on the bar.
    /// Body-mass (load_type=body_mass): `--weight` is your body mass in kg (required);
    ///   use `--external-load` for vest/belt weight (optional); use negative values for assistance.
    ///   Omitting body weight requires `--no-weight-recorded` (not recommended; prints a warning).
    /// Static holds: `--duration <seconds>` (omit --reps); body-mass holds still need `--weight`.
    /// Unilateral: add `--side left|right|both`
    ///
    /// Example (barbell):    repslog set add 1 --reps 10 --weight 60 --phase full --rir 0.0 --effective-reps 5
    /// Example (bodyweight): repslog set add 1 --reps 8 --weight 82 --phase full --external-load 5
    /// Example (eccentric):  repslog set add 1 --reps 3 --weight 82 --phase eccentric
    /// Example (hold):       repslog set add 1 --duration 60 --weight 82 --phase full --notes "Wall sit"
    /// Example (unilateral): repslog set add 1 --reps 8 --weight 82 --phase full --side left
    Add {
        workout_exercise_id: Option<String>,
        #[arg(short, long)]
        reps: Option<i32>,
        #[arg(
            short,
            long,
            help = "Load in kg (barbell/dumbbell) or body mass in kg for bodyweight exercises"
        )]
        weight: Option<f64>,
        /// Added load in kg on top of body weight (vest, belt). Bodyweight exercises only.
        #[arg(long = "external-load")]
        external_load: Option<f64>,
        /// Skip recording body weight (not recommended; set excluded from volume stats)
        #[arg(long = "no-weight-recorded")]
        no_weight_recorded: bool,
        #[arg(
            short,
            long,
            help = "Hold duration in seconds (for static/timed work; omit --reps)"
        )]
        duration: Option<i32>,
        #[arg(long)]
        distance: Option<f64>,
        #[arg(long)]
        rpe: Option<f64>,
        #[arg(long)]
        rir: Option<f64>,
        #[arg(long = "effective-reps")]
        effective_reps: Option<i32>,
        #[arg(long = "rest")]
        rest_seconds: Option<i32>,
        #[arg(short, long)]
        notes: Option<String>,
        /// Side for unilateral training (left, right, both). Stored separately for grouping and totals.
        #[arg(long, value_parser = ["left", "right", "both"])]
        side: Option<String>,
        /// Rep phase: full | eccentric | concentric (required)
        #[arg(long, value_parser = parse_phase)]
        phase: String,
        /// Average heart rate in bpm
        #[arg(long = "avg-heart-rate")]
        avg_heart_rate: Option<f64>,
        /// Maximum heart rate in bpm
        #[arg(long = "max-heart-rate")]
        max_heart_rate: Option<f64>,
        /// Heart rate zones JSON (e.g. '{"z1_seconds": 60, "z2_seconds": 300, ...}')
        #[arg(long = "hr-zones", value_parser = |s: &str| serde_json::from_str::<HeartRateZones>(s).map_err(|e| e.to_string()))]
        hr_zones: Option<HeartRateZones>,
        /// Average pace in min/km (e.g. 5.5)
        #[arg(long)]
        pace: Option<f64>,
        /// Calories burned
        #[arg(long)]
        calories: Option<i32>,
        /// Laps JSON (e.g. '[{"lap_number":1,"distance_km":1.0,"duration_seconds":332,"pace_min_per_km":5.533}, ...]')
        #[arg(long, value_parser = |s: &str| serde_json::from_str::<Laps>(s).map_err(|e| e.to_string()))]
        laps: Option<Laps>,
        /// Show what would be added (no changes)
        #[arg(long)]
        dry_run: bool,
    },
    /// Add a cardio set with mandatory heart rate and pace metrics.
    /// Example: repslog set add-cardio 1 --distance 5.0 --duration 1500 --avg-heart-rate 155 --max-heart-rate 180 --pace 5.0 --calories 450 --hr-zones '{"z1_seconds": 60, "z2_seconds": 1200, "z3_seconds": 240}' --laps '[{"km": 1, "time": "5:32", "pace": "5:32"}]'
    #[command(name = "add-cardio")]
    AddCardio {
        workout_exercise_id: Option<String>,
        #[arg(long)]
        distance: f64,
        #[arg(short, long)]
        duration: i32,
        #[arg(long = "avg-heart-rate")]
        avg_heart_rate: f64,
        #[arg(long = "max-heart-rate")]
        max_heart_rate: f64,
        #[arg(long = "hr-zones", value_parser = |s: &str| serde_json::from_str::<HeartRateZones>(s).map_err(|e| e.to_string()))]
        hr_zones: HeartRateZones,
        #[arg(long)]
        pace: f64,
        #[arg(long)]
        calories: i32,
        #[arg(long, value_parser = |s: &str| serde_json::from_str::<Laps>(s).map_err(|e| e.to_string()))]
        laps: Option<Laps>,
        #[arg(short, long)]
        notes: Option<String>,
        /// Side for unilateral training (left, right, both)
        #[arg(long, value_parser = ["left", "right", "both"])]
        side: Option<String>,
        /// Rep phase: full | eccentric | concentric (required)
        #[arg(long, value_parser = parse_phase)]
        phase: String,
        /// Show what would be added (no changes)
        #[arg(long)]
        dry_run: bool,
    },
    /// Add a rest-pause/cluster set sequence.
    /// Example (barbell):    repslog set add-cluster 1 --reps "10,5,5" --weight 100 --phase full --rir "0,0,1" --effective-reps "6,4,3" --rest 15
    /// Example (bodyweight): repslog set add-cluster 1 --reps "3,3,2" --weight 82 --phase full --external-load 5 --rir "0,0,1" --effective-reps "2,2,1" --rest 15
    #[command(name = "add-cluster")]
    AddCluster {
        workout_exercise_id: Option<String>,
        #[arg(
            short,
            long,
            help = "Load in kg (barbell/dumbbell) or body mass in kg for bodyweight exercises"
        )]
        weight: Option<f64>,
        #[arg(long = "external-load")]
        external_load: Option<f64>,
        #[arg(long = "no-weight-recorded")]
        no_weight_recorded: bool,
        /// Reps for each cluster separated by commas (e.g. "10,5,5")
        #[arg(short, long)]
        reps: String,
        /// RIR for each cluster separated by commas (e.g. "0,0,1")
        #[arg(long)]
        rir: String,
        /// Effective reps for each cluster separated by commas (e.g. "6,4,3")
        #[arg(long = "effective-reps")]
        effective_reps: String,
        /// Rest seconds between clusters (e.g. "15")
        #[arg(long = "rest")]
        rest_seconds: i32,
        #[arg(short, long)]
        notes: Option<String>,
        /// Side for unilateral training (left, right, both)
        #[arg(long, value_parser = ["left", "right", "both"])]
        side: Option<String>,
        /// Rep phase: full | eccentric | concentric (required)
        #[arg(long, value_parser = parse_phase)]
        phase: String,
        /// Show what would be added (no changes)
        #[arg(long)]
        dry_run: bool,
    },
    /// List sets for a workout exercise
    List { workout_exercise_id: i64 },
    /// Add an exercise to a workout. For bodyweight exercises, also logs the first set when
    /// --reps, --duration, or --weight is provided (body weight required unless --no-weight-recorded).
    Quick {
        workout_id: String,
        exercise_name_or_id: String,
        #[arg(short, long)]
        reps: Option<i32>,
        #[arg(
            short,
            long,
            help = "Body mass in kg for bodyweight exercises, or load for weighted exercises"
        )]
        weight: Option<f64>,
        #[arg(long = "external-load")]
        external_load: Option<f64>,
        #[arg(long = "no-weight-recorded")]
        no_weight_recorded: bool,
        #[arg(short, long)]
        duration: Option<i32>,
        #[arg(short, long)]
        notes: Option<String>,
        /// Rep phase: full | eccentric | concentric (required when logging the first set)
        #[arg(long, value_parser = parse_phase)]
        phase: Option<String>,
        /// Show what would be added (no changes)
        #[arg(long)]
        dry_run: bool,
    },
    /// Update any field on an existing set (reps, weight, notes, rir, side, phase, etc.).
    /// Example: repslog set update 287 --reps 10 --weight 82 --phase full --external-load 5 --notes "Left leg" --side left
    Update {
        set_id: String,
        #[arg(short, long)]
        reps: Option<i32>,
        #[arg(
            short,
            long,
            help = "Load in kg (barbell/dumbbell) or body mass in kg for bodyweight exercises"
        )]
        weight: Option<f64>,
        #[arg(long = "external-load")]
        external_load: Option<f64>,
        #[arg(long = "no-weight-recorded")]
        no_weight_recorded: bool,
        #[arg(long)]
        duration: Option<i32>,
        #[arg(long)]
        distance: Option<f64>,
        #[arg(long)]
        rpe: Option<f64>,
        #[arg(long)]
        rir: Option<f64>,
        #[arg(long = "effective-reps")]
        effective_reps: Option<i32>,
        #[arg(long = "rest")]
        rest_seconds: Option<i32>,
        #[arg(short, long)]
        notes: Option<String>,
        /// Side: left | right | both
        #[arg(long, value_parser = ["left", "right", "both"])]
        side: Option<String>,
        /// Rep phase: full | eccentric | concentric
        #[arg(long, value_parser = parse_phase)]
        phase: Option<String>,
        /// Show what would be updated (no changes)
        #[arg(long)]
        dry_run: bool,
    },
    /// Delete a specific set by ID. Asks for confirmation unless --force.
    Delete {
        set_id: String,
        /// Skip confirmation prompt
        #[arg(long)]
        force: bool,
        /// Show what would be deleted (no changes)
        #[arg(long)]
        dry_run: bool,
    },
    /// Reorder a set within its workout-exercise (changes display set number / order).
    /// Useful after corrections or when logging out of sequence.
    Move {
        set_id: String,
        /// Target 1-based position within the workout-exercise's sets
        #[arg(long)]
        to: i32,
        /// Show what would be moved (no changes)
        #[arg(long)]
        dry_run: bool,
    },
    /// Add matching left + right (or both) sets in one go for unilateral work.
    /// reps (and optional rir/effective-reps) are provided as comma lists, like add-cluster.
    /// Example: repslog set add-unilateral 83 --reps "8,10,10,10" --weight 82 --phase full --side both
    #[command(name = "add-unilateral")]
    AddUnilateral {
        workout_exercise_id: Option<String>,
        /// Reps for the sets (comma-separated). One set per value will be created per side.
        #[arg(short, long)]
        reps: String,
        #[arg(
            short,
            long,
            help = "Load in kg (barbell/dumbbell) or body mass in kg for bodyweight exercises"
        )]
        weight: Option<f64>,
        #[arg(long = "external-load")]
        external_load: Option<f64>,
        #[arg(long = "no-weight-recorded")]
        no_weight_recorded: bool,
        /// RIR values (comma-separated, same length as reps)
        #[arg(long)]
        rir: Option<String>,
        /// Effective-reps values (comma-separated)
        #[arg(long = "effective-reps")]
        effective_reps: Option<String>,
        #[arg(long = "rest")]
        rest_seconds: Option<i32>,
        #[arg(short, long)]
        notes: Option<String>,
        /// left | right | both (both creates a left+right pair for each rep value)
        #[arg(long, value_parser = ["left", "right", "both"], default_value = "both")]
        side: String,
        /// Rep phase: full | eccentric | concentric (required)
        #[arg(long, value_parser = parse_phase)]
        phase: String,
        /// Show what would be added (no changes)
        #[arg(long)]
        dry_run: bool,
    },
}

#[derive(Subcommand)]
pub enum StatsAction {
    /// Personal records
    Prs {
        #[arg(short, long)]
        exercise: Option<String>,
    },
    /// Training volume
    Volume {
        #[arg(short, long)]
        exercise: Option<String>,
        #[arg(short, long, default_value = "30d")]
        period: String,
    },
    /// Training summary
    Summary {
        #[arg(short, long, default_value_t = 30)]
        days: i64,
    },
    /// Load progression for a specific exercise (chronological).
    /// Bodyweight: shows body mass and external load; barbell: shows bar load.
    Weight {
        #[arg(short, long)]
        exercise: String,
    },
    /// Per-set history for an exercise across workouts in a date range.
    /// Lists every logged set (not totals) for workouts that include the exercise.
    History {
        #[arg(short, long)]
        exercise: String,
        #[arg(short, long, default_value_t = 30)]
        days: i64,
    },
}
