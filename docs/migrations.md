# Database & Migrations

`repslog` uses a local SQLite database to store all your training data. As the tool evolves, the database schema may need to change. `repslog` includes a built-in migration system to handle these changes safely.

## The Migrations Table

The database includes a `migrations` table that tracks which versions of the schema have been applied.

## Running Migrations

When you update `repslog`, you should run the `migrate` command to ensure your database is up-to-date:

```bash
repslog migrate
```

### Interactive data migrations

Some migrations change existing data. Migration **012** lowercases every exercise name and **merges** rows that would collide after lowercasing (e.g. `Running` and `running`).

- On a TTY (interactive terminal), `repslog migrate` may ask which exercise id to keep when a merge is ambiguous.
- Non-interactive runs (pipes, CI, `--json`) auto-pick the survivor: most workout uses, then lowest id.
- After 012, catalog names must stay lowercase (enforced when adding exercises).

## Migration Commands

### Check Status
See your current version and if there are any pending migrations:

```bash
repslog migrate --status
```

### Dry Run
See the SQL that would be executed without actually applying it:

```bash
repslog migrate --dry-run
```

### Force Apply
In case of a partial or failed migration, you can force the system to re-apply migrations (note: this is idempotent and should be used with caution):

```bash
repslog migrate --force
```

## Database Location

The database file is stored in your user's data directory:
- **Linux:** `~/.local/share/repslog/repslog.db`
- **macOS:** `~/Library/Application Support/repslog/repslog.db`

You can manually back up this file to preserve your training history.

Use the `--db <PATH>` global option (e.g. `repslog --db /tmp/test.db migrate`) to target a specific database file instead of the default location. This is especially handy for testing.
