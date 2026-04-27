# Database Notes

`repslog` uses SQLite for local data storage.

## Location
On Linux systems, the database is located at:
`~/.local/share/repslog/repslog.db`

You can verify the location or check for existence with:
```bash
ls -l ~/.local/share/repslog/repslog.db
```

## Schema Management
`repslog` manages its own migrations. You can check the migration status or apply updates using the `migrate` command.

- **Check Status**: `repslog migrate --status`
- **Apply Migrations**: `repslog init` or `repslog migrate` (without dry-run)

## Backups
Since it's a single SQLite file, backups are simple:
```bash
cp ~/.local/share/repslog/repslog.db ~/repslog_backup.db
```

## Direct Querying
If you have `sqlite3` installed, you can query the database directly for custom reports:
```bash
sqlite3 ~/.local/share/repslog/repslog.db "SELECT * FROM exercises LIMIT 5;"
```
*(Caution: Do not modify the database schema directly as it may break the application.)*
