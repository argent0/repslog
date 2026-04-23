# repslog

`repslog` is a Linux-first (macOS compatible) command-line workout tracker designed for flexibility across various training modalities (strength, calisthenics, cardio, etc.). It uses SQLite for local storage and provides a scriptable CLI.

## Tech Stack
- **Language:** Rust
- **Database:** SQLite (via `sqlx`)
- **CLI Framework:** `clap` v4
- **Serialization:** `serde`, `serde_json`
- **Output Formatting:** `comfy-table`, `colored`
- **Time/Date:** `chrono`
- **Error Handling:** `anyhow`, `thiserror`
- **Asynchronous Runtime:** `tokio`

## Project Structure
- `migrations/`: SQL migration files.
- `src/main.rs`: Application entry point.
- `src/cli.rs`: CLI command and argument definitions using `clap`.
- `src/db.rs`: Database connection handling and migration logic.
- `src/repository.rs`: Data access layer (Repository pattern).
- `src/models/`: Database entity definitions.
- `src/commands/`: Implementation of CLI subcommands.
- `src/utils.rs`: Shared utility functions.
- `tests/`: Integration tests.

## Development Guidelines

### Database & Migrations
- Use `sqlx` macros for database queries where possible.
- New schema changes MUST be implemented as versioned SQL files in `migrations/`.
- The `init` command handles running migrations.

### CLI Design
- Follow the `repslog <group> <action>` structure.
- Ensure commands are non-interactive friendly and support `stdin` where appropriate.
- Use `comfy-table` for tabular output.

### Error Handling
- Use `anyhow::Result` for general command results.
- Define specific error types in `src/error.rs` using `thiserror` for library-level errors.

### Testing
- Integration tests are located in `tests/`.
- Use `repslog::db::setup_test_db` in tests to get an in-memory or temporary SQLite instance.
- Always add/update tests when modifying core logic or adding features.

## Common Tasks

### Adding a New Command
1. Define the command in `src/cli.rs`.
2. Implement the command logic in a new or existing file in `src/commands/`.
3. Wire the command in `src/main.rs`.

### Database Schema Changes
1. Create a new migration file: `migrations/XX_name.sql`.
2. Update structs in `src/models/` if necessary.
3. Update `src/repository.rs` to reflect changes.

### Running Tests
```bash
cargo test
```

### Formatting and Linting
```bash
cargo fmt
cargo clippy
```
