# Getting Started

This guide will help you get `repslog` installed and set up for your first workout.

## Installation

`repslog` is built with Rust. You can install it from source using `cargo`.

```bash
# Clone the repository (if you haven't already)
# git clone https://github.com/username/repslog.git
# cd repslog

# Install the binary
cargo install --path .
```

## Initialization

Before using `repslog`, you need to initialize the database. This command creates the SQLite database file and applies the initial schema and default exercises.

```bash
repslog init
```

You can use `--dry-run` to see what would be initialized without actually creating the database:
```bash
repslog init --dry-run
```

### Data Location

By default, `repslog` follows the XDG Base Directory Specification:
- **Linux:** `~/.local/share/repslog/repslog.db`
- **macOS:** `~/Library/Application Support/repslog/repslog.db`

### Custom Database Location

Use the global `--db` flag to specify a custom database file path. This is useful for testing, scripting isolation, or using multiple databases:

```bash
repslog --db /tmp/test-workouts.db init
repslog --db ./my-training.db workout create --date "2026-01-01 10:00:00" --type "Test"
repslog --db /tmp/test-workouts.db exercise list
```

The flag can appear before or after the subcommand.

## Verification

After initialization, you can verify that the default exercises have been loaded:

```bash
repslog exercise list
```

You should see a table of standard exercises like "squat (barbell)", "pushups", and "running".

## Database Migrations

If you are upgrading from an older version of `repslog`, you may need to run migrations to update your database schema:

```bash
repslog migrate
```

To check the status of migrations:

```bash
repslog migrate --status
```

To see which migrations would be applied without applying them:
```bash
repslog migrate --dry-run
```
