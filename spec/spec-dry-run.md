You are an expert Rust + Clap developer working on https://github.com/argent0/repslog (a CLI workout tracker using Clap v4, Diesel/SQLite).

**Task**  
Add a `--dry-run` flag (with short option `-n` if it fits the existing style) **only to the mutating subcommands**. Do **not** make it a global/top-level flag.

**Subcommands that must receive the flag** (all others stay unchanged):
- `init`
- `exercise add`
- `workout create`
- `workout update`
- `workout-exercise add`
- `set add`
- `set add-cardio`
- `set add-cluster`
- `set quick` (if the command exists)
- `workout delete` (if it exists and performs writes)

**Alignment requirement**  
Implement the flag **exactly like the existing `migrate --dry-run`** (see `src/commands/migrate.rs` and how the flag is declared in `src/cli.rs` under the `Migrate` variant). The flag should appear in the help text the same way.

**Exact behavior when `--dry-run` (or `-n`) is used**  
1. Perform **full argument parsing and validation** exactly as in normal mode (all existing checks, existence checks, type parsing, etc. must still run).  
2. **Never touch the database for writes** — no INSERT, UPDATE, DELETE, or any repository call that mutates data.  
3. Produce **exactly the same human-readable success output** that a real execution would produce (no extra “Dry run: …” messages, no different formatting).  
4. For any command that normally outputs/returns a created ID (workout create, workout-exercise add, exercise add, set add, etc.):
   - Output the placeholder ID in the format **`DRY-RUN-N`** where `N` is the actual next auto-increment sequence number that *would have been inserted* if the operation had run (i.e. simulate what the next primary key would be).
   - Example: instead of “Created workout 42”, output “Created workout DRY-RUN-42” (or whatever the success message normally prints, just with the placeholder ID).
5. Validation errors, argument errors, and other failures must still be shown exactly as they are today.

**Implementation guidelines**  
- Add `dry_run: bool` (with the appropriate `#[arg(...)]` attribute) to the relevant structs inside `src/cli.rs` (ExerciseAction::Add, WorkoutAction::Create, etc.).
- Propagate the `dry_run` flag down to the command handler functions in `src/commands/` (exercise.rs, workout.rs, set.rs, init.rs, etc.).
- In the repository layer (`src/repository.rs`) or wherever the actual DB operations live, add an optional `dry_run: bool` parameter (or a new method like `create_dry_run(...)`) so the logic can be reused cleanly.
- When `dry_run` is `true`:
  - Skip the actual write.
  - Compute the “would-be” next ID (safe to do a read query on `sqlite_sequence` or `MAX(id)+1` for the relevant table — reads are allowed).
  - Return/emit the placeholder ID so the success-printing code remains unchanged.
- Keep the existing code style, error handling (`crate::error::Result`), and output formatting untouched.

**Additional requirements**  
- Update help text automatically via Clap (no manual changes needed).
- Make sure piping still works in dry-run mode (e.g. `repslog workout create --dry-run | xargs repslog workout-exercise add --dry-run` should receive the DRY-RUN-xxx ID).
- Do not change any read-only commands (list, view, stats, search, etc.).
- After implementation, the project must still compile and all existing tests (if any) must pass.

Implement the feature, commit the changes with a clear message, and verify that `--dry-run` works on at least `workout create` and `set add` as a minimum.
