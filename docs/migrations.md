# Database & Migrations

`repslog` uses a local SQLite database to store all your training data. As the tool evolves, the database schema may need to change. `repslog` includes a built-in migration system to handle these changes safely.

## The Migrations Table

The database includes a `migrations` table that tracks which versions of the schema have been applied.

## Running Migrations

When you update `repslog`, you should run the `migrate` command to ensure your database is up-to-date:

```bash
repslog migrate
```

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
