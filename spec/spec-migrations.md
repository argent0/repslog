**Additional Requirements Document: Robust & Production-Grade Migration System for `repslog`**

You are extending the existing `repslog` codebase (Rust + SQLite CLI workout tracker). The migration system must now become fully robust, explicit, and safe for long-term real-world use. Users (and LLM-driven scripts) will have years of training data in their `~/.local/share/repslog/repslog.db` file, so schema evolution must never risk data loss or leave the database in an inconsistent state.

**Core Goal (the haystack)**  
Provide a reliable, version-controlled migration system that lets any user — regardless of how old their database is — safely upgrade to the latest schema with a single command.

**Strict Non-Negotiable Rules (the needle)**  
- The system **must** support migrating from **any prior schema version** directly to the latest version (e.g., v1 → v7, or v3 → v7) without manual intervention or intermediate steps.  
- Never rely on auto-migration on every command run. The user must explicitly run `repslog migrate` when an upgrade is needed.  
- No defaults or implicit behavior that could silently change data semantics.  
- All migration logic must be deterministic, transactional where possible, and fully auditable.  
- The tool remains fully scriptable and non-interactive friendly.

**Required New Capabilities**

1. **New Top-Level Command: `repslog migrate`**  
   Add the command `repslog migrate` (with optional flags):
   ```bash
   repslog migrate                 # Apply all pending migrations to reach latest version
   repslog migrate --status        # Show current version vs. latest available (no changes)
   repslog migrate --dry-run       # Show exactly what would be applied (no changes)
   ```
   - Output must be clear and informative:
     - Current schema version
     - Target (latest) version
     - List of migrations that will be applied
     - Success message: “Successfully migrated from version 3 to version 5 (2 migrations applied).”
   - If already up-to-date: “Database is already at the latest version (5). No changes needed.”

2. **Schema Version Tracking in the Database**  
   - Use (or enhance) the existing `migrations` table to reliably track applied versions:
     ```sql
     CREATE TABLE IF NOT EXISTS migrations (
         version     INTEGER PRIMARY KEY,
         name        TEXT NOT NULL,           -- e.g. "002_add_rir_effective_reps.sql"
         applied_at  TEXT DEFAULT CURRENT_TIMESTAMP,
         checksum    TEXT                     -- optional: hash of the migration file for safety
     );
     ```
   - The application must maintain a constant `LATEST_SCHEMA_VERSION` (or read it from the highest migration file) in code.
   - The system must always be able to determine the exact current version of any existing database.

3. **Migration File Structure & Execution**  
   - All schema and data changes live in the `migrations/` directory.
   - Files must be named with zero-padded sequential numbers, e.g.:
     - `001_initial_schema.sql`
     - `002_add_rir_and_effective_reps_support.sql`
     - `003_add_rest_pause_cluster_support.sql`
     - etc.
   - Each file contains pure SQLite SQL (DDL + DML) plus header comments describing the change.
   - The Rust migration runner must:
     - Discover all migration files at compile time or runtime.
     - Apply them in strict numerical order.
     - Record each successfully applied migration in the `migrations` table.
     - Run inside a transaction (where SQLite allows) for atomicity.
     - Support both schema changes (`ALTER TABLE`, `CREATE TABLE`) and data migrations (backfills, default-value updates, etc.).
   - Migrations must be **idempotent where possible** (e.g., use `IF NOT EXISTS` for tables/columns when appropriate).

4. **Safety & Robustness Guarantees**  
   - If any migration fails, the process must stop immediately, roll back the failed transaction (if inside one), and leave the database exactly as it was before the command.
   - Provide clear, actionable error messages with the failing migration name and the exact SQLite error.
   - `repslog init` (for brand-new databases) must create the DB at the latest version automatically.
   - Every other command (`workout create`, `set add`, `stats`, etc.) must check the schema version on startup. If outdated, the command must exit with a clear message:  
     “Database schema is outdated (version 3). Please run `repslog migrate` first.”

5. **Integration with Previous & Future Features**  
   - The RIR / effective-reps / rest-pause changes (from the prior requirements document) must be implemented as migration `002_add_rir_and_effective_reps_support.sql` (or the next sequential number).
   - All future features must follow the same migration pattern.
   - The migration system must be the single source of truth for schema changes — never alter the schema directly in Rust code outside of migration files.

**Final Instructions**  
Update the codebase to include this complete, robust migration system. Produce:
- The updated `db.rs` (or dedicated `migration.rs`) with the new runner logic.
- The new `migrate` command implementation in `cli.rs` / `commands/`.
- All existing migration files re-organized with proper numbering.
- The new migration file for RIR / effective reps (if not already present).
- Updated README.md section explaining migrations.
- Any helper utilities needed for version checking.

Implement with the same production-grade quality, error handling, and explicitness required in previous specifications. The resulting migration system must feel trustworthy enough for a lifter to rely on for years of data.

Begin implementation now.
