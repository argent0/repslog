# Implementation Report: `--db` Global Option for Custom Database Specification

**Date:** 2026-04 (post-implementation)  
**Feature:** Add `--db <PATH>` option to specify which SQLite DB file to use.  
**Author/Context:** Implemented via Grok 4.3 interactive session in /home/aner/rust/repslog.  
**Related PR/Change:** Follow-up to core CLI enhancement for testing flexibility.

## 1. Overview and Motivation

`repslog` is a CLI workout tracker using SQLite for storage (via sqlx), with the default DB location determined by the XDG Base Directory spec (via the `directories` crate in `src/config.rs`):

- Linux: `~/.local/share/repslog/repslog.db`
- macOS: `~/Library/Application Support/repslog/repslog.db`

Previously, overriding this required environment hacks (e.g., `XDG_DATA_HOME` in the `docs/verify_examples.sh` script).

The `--db` option was requested to:
- Make it easy to specify a custom DB file path directly on the CLI.
- Support testing scenarios (e.g., isolated temp DBs, file-based instead of always in-memory for some integration flows).
- Allow users/scripts to target specific DBs without side effects on the default one.
- Be a *global* option (works before/after subcommands, e.g., `repslog --db /tmp/test.db init` or `repslog workout list --db ./my.db`).

This aligns with the project's "non-interactive friendly and support stdin" CLI design using `clap` v4.

## 2. Design Decisions

- **Global flag**: Added to the top-level `Cli` struct in `clap` derive with `global = true`. This propagates to all subcommands automatically in help/usage.
- **Optional**: `Option<String>`. When `None`, falls back to existing `get_db_url()` logic (XDG).
- **Path handling**:
  - Plain paths (relative like `test.db`, absolute like `/tmp/foo.db`, or with subdirs) are supported.
  - Special cases: `:memory:` normalized to `sqlite::memory:`; full `sqlite:...` URLs passed through.
  - Parent directories are auto-created (mirroring what `get_db_path()` does for the default).
  - URL construction: `sqlite://` + path (consistent with prior code).
- **No env var support yet**: Although `clap` has `env` feature enabled, only `--db` was implemented per request (easy to extend later with `env = "REPSLOG_DB"`).
- **Library-friendly**: `setup_db` now takes `Option<&str>` so tests and library users can use custom paths without CLI parsing. `setup_test_db()` (in-memory + auto-migrations) remains unchanged per AGENTS.md guidelines.
- **No schema changes**: Purely runtime/config; no migrations needed.
- **Backwards compatible**: Default behavior identical when flag omitted.

## 3. Code Changes

### Core Changes

- **[src/cli.rs](/home/aner/rust/repslog/src/cli.rs)** (lines ~7-14):
  Added to `Cli`:
  ```rust
  /// Path to SQLite database file (overrides default XDG location)
  #[arg(long, global = true, value_name = "PATH")]
  pub db: Option<String>,
  ```
  (Placed before `#[command(subcommand)]`.)

- **[src/config.rs](/home/aner/rust/repslog/src/config.rs)**:
  - Kept `get_db_path()` and `get_db_url()` unchanged for default behavior.
  - Added new public helper:
    ```rust
    pub fn get_db_url_with_override(db_override: Option<&str>) -> Result<String> { ... }
    ```
    Handles path normalization, `create_dir_all` for parents, special sqlite/memory cases.

- **[src/db.rs](/home/aner/rust/repslog/src/db.rs)**:
  - Updated import to `get_db_url_with_override`.
  - Changed signature:
    ```rust
    pub async fn setup_db(db_path: Option<&str>) -> Result<SqlitePool>
    ```
  - `setup_test_db()` untouched (still uses hard-coded `sqlite::memory:` + forces migrations for test isolation).

- **[src/main.rs](/home/aner/rust/repslog/src/main.rs)**:
  - Updated single call site:
    ```rust
    let pool = setup_db(cli.db.as_deref()).await?;
    ```

No changes to commands, repository, error handling, or models (the pool is passed down as before).

### Test Updates (per AGENTS.md: "Always add/update tests when modifying core logic")

- **[tests/init_test.rs](/home/aner/rust/repslog/tests/init_test.rs)**:
  - Added `temp_db_path()` helper using `std::env::temp_dir()` + timestamp for uniqueness (no extra deps like `tempfile`).
  - New test `test_setup_db_with_custom_path()`:
    - Uses `repslog::db::setup_db(Some(path_str))` (exercises the new path).
    - Calls `handle_init`, verifies file creation on disk, seeds exercises via repo, then cleans up.
  - Existing tests *unchanged* and continue to use `setup_test_db()` exclusively (as mandated by AGENTS.md).

All other test files (`workout_test.rs`, `set_test.rs`, etc.) untouched.

`cargo test` (full suite) passes, including the new test.

### Documentation Updates (per AGENTS.md strict rules)

- Updated in `docs/` (and high-level README):
  - [docs/getting-started.md](/home/aner/rust/repslog/docs/getting-started.md): New "Custom Database Location" subsection with examples.
  - [docs/migrations.md](/home/aner/rust/repslog/docs/migrations.md): Added note about `--db` under Database Location.
  - [README.md](/home/aner/rust/repslog/README.md): Brief mention in Usage Guide.
- Also synced the (now-to-be-removed) `skill/references/cli-reference.md` for consistency during dev, including root + group command help text (Usage lines now show `[OPTIONS]`, plus `--db` in Options blocks).
- **Verification required**: After edits, `cargo build && ./docs/verify_examples.sh` was executed and succeeded (no breakage to testable examples, even though examples don't use `--db` yet).

No changes needed to other docs (e.g., workouts.md) as they don't detail DB location.

### Other

- Ran `cargo fmt`, `cargo clippy` (pre-existing "too many arguments" warnings in repository.rs were not touched).
- `cargo build` succeeded.
- Manual smoke: `repslog --db /tmp/test_$$.db init` + list worked as expected (file created, no impact on default `repslog.db`).

## 4. Testing and Validation Performed

- Unit/integration: `cargo test` – all pass (new custom-path test exercises file-based DB creation + init flow).
- Doc verification: `./docs/verify_examples.sh` (uses XDG override internally; still passes).
- CLI help: `--db` appears globally in `repslog --help`, subcommand help (e.g. `repslog workout --help`), and leaf commands.
- Edge cases covered in impl:
  - Custom dir creation (e.g., `--db subdir/nested.db`).
  - Relative vs absolute paths.
  - No breakage to default path logic.
- AGENTS.md compliance: Tests still prefer `setup_test_db` for most cases; docs updated + verified; no unversioned schema changes.

Future test improvements could use `--db` in an integration test harness (e.g., via `assert_cmd` + temp files), but not added as it would require new dev-dependencies and wasn't requested.

## 5. Files Touched Summary

**Added:**
- `reports/implementation-report-db-option.md` (this report)

**Modified:**
- `src/cli.rs`
- `src/config.rs`
- `src/db.rs`
- `src/main.rs`
- `tests/init_test.rs`
- `docs/getting-started.md`
- `docs/migrations.md`
- `README.md`
- `skill/references/cli-reference.md` (temporary, prior to removal)
- `PKGBUILD` (cleanup in follow-up)

**Deleted (see removal task):**
- `skill/` (entire directory + git tracking)

No Cargo.toml changes (no new deps).

## 6. Notes / Future Work / Gotchas

- The default `repslog.db` at repo root (visible in `git status` initially) remains; it's likely for manual/dev use and not auto-ignored.
- `verify_examples.sh` still relies on `XDG_DATA_HOME` + `mktemp`; it could be modernized to use `--db $TMP_DIR/repslog.db` for all invocations (and pass `REPSLOG_BIN` with flag), but left as-is since not requested and it continues to work.
- Removing `skill/` (Grok/LLM agent skill definitions + references) cleans up the repo. This report lives in `reports/` as a new convention for implementation notes (outside the "testable documentation" in `docs/`).
- If re-adding similar skill support later, it should probably live under `docs/` or a dedicated `skills/` (but follow AGENTS.md update rules).
- Potential enhancement: Support `--db` via env var and/or config file.
- The implementation keeps `config.rs` as the single source for DB URL resolution.

## 7. Commands Run (for Reproducibility)

```bash
cargo fmt
cargo clippy
cargo build
cargo test
cargo build && ./docs/verify_examples.sh
# (plus manual --db tests)
```

All succeeded with exit 0.

---

This report documents the complete, tested, and documented addition of the requested `--db` feature while strictly adhering to project guidelines in AGENTS.md.
