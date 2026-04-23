use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "repslog")]
#[command(about = "A Linux-first workout tracker", long_about = None)]
pub struct Cli {
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
    /// Initialize database and seed default exercises
    Init,
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
    /// Add a new exercise
    Add {
        name: String,
        #[arg(short, long)]
        category: String,
        #[arg(short, long)]
        equipment: Option<String>,
        #[arg(short, long)]
        muscles: Option<String>,
        #[arg(short, long)]
        description: Option<String>,
    },
    /// Search for exercises
    Search {
        term: String,
    },
}

#[derive(Subcommand)]
pub enum WorkoutAction {
    /// Create a new workout
    Create {
        #[arg(short, long = "type")]
        workout_type: Option<String>,
        #[arg(short, long)]
        notes: Option<String>,
        #[arg(short, long)]
        date: String,
    },
    /// List workouts
    List {
        #[arg(short, long, default_value_t = 10)]
        limit: i64,
        #[arg(short, long)]
        days: Option<i64>,
    },
    /// View details of a specific workout
    View {
        workout_id: i64,
    },
    /// Finish a workout
    Finish {
        workout_id: i64,
        #[arg(short, long)]
        duration: Option<i32>,
        #[arg(short, long)]
        feeling: Option<i32>,
    },
    /// Show current active workout
    Current,
    /// Delete a workout
    Delete {
        workout_id: i64,
    },
}

#[derive(Subcommand)]
pub enum WorkoutExerciseAction {
    /// Add an exercise to a workout
    Add {
        workout_id: i64,
        exercise_id_or_name: String,
        #[arg(short, long)]
        order: Option<i32>,
    },
    /// List exercises in a workout
    List {
        workout_id: i64,
    },
}

#[derive(Subcommand)]
pub enum SetAction {
    /// Add a set to a workout exercise.
    /// Example: repslog set add 1 --reps 10 --weight 60 --rir 0.0 --effective-reps 5
    Add {
        workout_exercise_id: Option<i64>,
        #[arg(short, long)]
        reps: Option<i32>,
        #[arg(short, long)]
        weight: Option<f64>,
        #[arg(short, long)]
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
    },
    /// Add a rest-pause/cluster set sequence.
    /// Example: repslog set add-cluster 1 --reps "10,5,5" --weight 100 --rir "0,0,1" --effective-reps "6,4,3" --rest 15
    #[command(name = "add-cluster")]
    AddCluster {
        workout_exercise_id: Option<i64>,
        #[arg(short, long)]
        weight: Option<f64>,
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
    },
    /// List sets for a workout exercise
    List {
        workout_exercise_id: i64,
    },
    /// Convenience: add exercise + first set in one go
    Quick {
        workout_id: i64,
        exercise_name_or_id: String,
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
}
