**Here is the complete, ready-to-use prompt you can give to a coding agent (or copy-paste into Cursor/Claude/GPT/etc.):**

---

You are an expert Rust developer specializing in high-quality, production-grade CLI tools. Your mission is to build a complete, well-architected, and immediately usable CLI application called **`repslog`**.

### Project Overview
`repslog` is a Linux-first (but macOS-compatible) command-line workout tracker that can handle **any kind of training** — traditional strength training, calisthenics/bodyweight movements, cardio (running, cycling, swimming), HIIT, yoga, timed holds (planks, L-sits), etc. It must be extremely flexible on the data it stores per set while remaining easy to use.

The tool must:
- Use **SQLite** as the local database (single file stored in the proper XDG data directory: `~/.local/share/repslog/repslog.db` on Linux).
- Be fully scriptable/non-interactive friendly: many commands must support reading values from **stdin** (e.g. `echo "1" | repslog set add --workout-exercise-id` or piping IDs from previous commands).
- Follow a clean subcommand structure (`repslog <group> <action>`).
- Include a proper migration system so the schema can evolve safely.
- Produce beautiful, colored, tabular output for lists and views.
- Be written with modern Rust best practices (2024+ edition, idiomatic code, excellent error handling, clear separation of concerns).

### Exact Database Schema (must be implemented exactly as shown, with any helpful indexes)
```sql
PRAGMA foreign_keys = ON;

CREATE TABLE exercises (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    category TEXT NOT NULL,           -- e.g. "strength", "calisthenics", "cardio", "flexibility", "hiit"
    muscle_groups TEXT,               -- JSON array or comma-separated string, e.g. '["chest","triceps"]'
    equipment TEXT,                   -- "barbell", "dumbbell", "bodyweight", "none", etc.
    description TEXT,
    is_custom INTEGER DEFAULT 0,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE workouts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    started_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    finished_at TEXT,
    workout_type TEXT,                -- "push", "pull", "legs", "full_body", "run", "yoga", etc.
    notes TEXT,
    overall_feeling INTEGER CHECK (overall_feeling BETWEEN 1 AND 5),
    duration_minutes INTEGER,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE workout_exercises (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    workout_id INTEGER NOT NULL REFERENCES workouts(id) ON DELETE CASCADE,
    exercise_id INTEGER NOT NULL REFERENCES exercises(id),
    "order" INTEGER NOT NULL,
    notes TEXT
);

CREATE TABLE exercise_sets (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    workout_exercise_id INTEGER NOT NULL REFERENCES workout_exercises(id) ON DELETE CASCADE,
    set_number INTEGER NOT NULL,
    reps INTEGER,
    weight_kg REAL,
    distance_km REAL,
    duration_seconds INTEGER,
    rpe REAL,                         -- Rate of Perceived Exertion (e.g. 7.5)
    rest_seconds INTEGER,
    notes TEXT,
    extra_metrics TEXT,               -- JSONB-style for future extensibility
    created_at TEXT DEFAULT CURRENT_TIMESTAMP
);

-- Migration tracking table
CREATE TABLE IF NOT EXISTS migrations (
    version INTEGER PRIMARY KEY,
    applied_at TEXT DEFAULT CURRENT_TIMESTAMP
);
```

### Required CLI Command Structure
Use `clap` v4 (derive API preferred) with excellent help text, examples, and colors. The structure must be:

```
repslog
├── exercise
│   ├── list [--search <term>] [--category <cat>]
│   ├── add <name> [--category <cat>] [--equipment <eq>] [--muscles "chest,triceps"] [--description "..."]
│   └── search <term>
├── workout
│   ├── create [--type <type>] [--notes "..."]          # returns the new workout ID
│   ├── list [--limit N] [--days N]
│   ├── view <workout_id>
│   ├── finish <workout_id> [--duration <min>] [--feeling <1-5>]
│   ├── current                                         # shows active workout if any
│   └── delete <workout_id>
├── session                                             # alias for workout commands
├── workout-exercise
│   ├── add <workout_id> <exercise_id_or_name> [--order N]
│   └── list <workout_id>
├── set
│   ├── add <workout_exercise_id> [--reps N] [--weight <kg>] [--duration <sec>] [--distance <km>] [--rpe X.Y] [--notes "..."]
│   ├── list <workout_exercise_id>
│   └── quick <workout_id> <exercise_name_or_id>        # convenience: add exercise + first set in one go
├── stats
│   ├── prs [--exercise <name>]
│   ├── volume [--exercise <name>] [--period 30d|90d|1y]
│   └── summary [--days N]
└── init                                                # create DB + run migrations + seed default exercises (include plenty of calisthenics)
```

Support flexible set data (only the fields that make sense for the exercise type can be provided; the rest stay NULL).

### Project Structure & Best Practices (must follow)
```
repslog/
├── Cargo.toml
├── migrations/                  # versioned .sql files (01_init.sql, 02_add_xxx.sql, etc.)
├── src/
│   ├── main.rs
│   ├── cli.rs                   # all clap definitions
│   ├── config.rs                # XDG paths, settings
│   ├── db.rs                    # connection pool + migration runner
│   ├── models/                  # Exercise, Workout, WorkoutExercise, Set structs + FromRow/Serialize
│   ├── commands/                # exercise.rs, workout.rs, set.rs, stats.rs, etc.
│   ├── repository.rs            # (or inline in commands — your choice, but keep it clean)
│   ├── utils.rs
│   └── error.rs                 # custom error type with anyhow + thiserror
├── README.md                    # full usage examples + installation
└── .gitignore
```

**Dependencies you should include** (add any reasonable extras):
- `clap` (derive, color)
- `rusqlite` or `sqlx` (your choice; sqlx with migrate! macro is excellent)
- `anyhow`, `thiserror`
- `directories` (for XDG)
- `chrono` or `time` (for dates)
- `serde`, `serde_json`
- `comfy-table` or `tabled` (for pretty output)
- `colored` / `owo-colors`
- `env_logger` or `tracing`

### Final Instructions
Generate the **complete initial codebase** for `repslog`. Start by outputting:
1. The full `Cargo.toml`
2. The complete directory structure
3. Every source file and migration file in order, with high-quality, well-commented, idiomatic Rust code.

The code must be production-ready, easily extensible, and something a serious lifter would be happy to use daily and contribute to. Make zero mistakes on the schema, command names, stdin support, or Rust practices.

Begin now.


