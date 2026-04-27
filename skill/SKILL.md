---
name: repslog
description: Log, manage, and analyze strength, calisthenics, and cardio workouts using the repslog CLI. Use when you need to create workouts, add exercises and sets (standard, cluster/rest-pause, or cardio with HR zones/laps), view PRs/stats/volume, initialize the DB, or script repeatable workout logging pipelines with the repslog binary.
---

# Skill: repslog (workout-tracker)

## Description
`repslog` is a lightweight, Linux-first CLI workout tracker optimized for strength training, calisthenics, and cardio (with full HR zone + lap support).  

This skill teaches the agent how to fully control the installed `repslog` binary via direct shell commands — no wrappers, no extra code. The agent can create, update, query, and analyze workouts entirely through `repslog <subcommand>`.

## When to use this skill
- User asks to log a workout, add an exercise/set, or track a training session
- User wants to view personal records (PRs), training volume, or statistics
- User needs to initialize the database or run migrations
- Any scripting or automation involving workout data via the repslog CLI

## Prerequisites
- `repslog` binary is installed and available in `$PATH`
- Database lives at `~/.local/share/repslog/repslog.db` (XDG-compliant)
- Linux-first (macOS fully supported; WSL for Windows)

## Core Usage Principles
- **Safety**: Always test mutating commands with `--dry-run` first.
- **Chaining**: Capture IDs with `| tail -n 1` or pipe directly via stdin.
- **JSON fields**: `--hr-zones` and `--laps` accept valid JSON (see `references/cardio-json-examples.md`).
- **Parsing**: Use `tail -n 1` for new IDs; prefer name-based matching when possible.
- **Scriptability**: The tool is explicitly designed for one-liner chained workflows.

## Capabilities
- Exercise library (search/add/list)
- Workout lifecycle (create/list/view/update/delete)
- All set types: standard strength, cluster/rest-pause, cardio (distance, pace, calories, HR zones, laps)
- One-shot quick logging
- Rich stats (PRs, volume, summaries)
- Database init and migrations

## Command Reference
See `references/cli-reference.md` for complete `--help` output of every subcommand.

## Best Practices & Patterns
- Search for an exercise before creating it
- Prefer `--dry-run` on any `add`/`create`/`update` command
- Chain commands using ID piping for full workouts in a single line
- Add `--notes` for context (RPE, fatigue, PR attempts)
- Store complex JSON in variables or heredocs

## Common Workflows

### 1. Initialize (run once)
```bash
repslog init
```

### 2. Quick one-shot log
```bash
repslog set quick --exercise "Pushups" --reps 25 --rir 1 --notes "Felt strong"
```

### 3. Full chained strength workout
```bash
WID=$(repslog workout create --date "$(date +%Y-%m-%d)" --type strength --notes "Push/Pull" | tail -n 1)
repslog workout-exercise add "$WID" "Bench Press" | xargs -I {} repslog set add {} --weight 85 --reps 8 --rir 2
```

### 4. Cardio with HR zones & laps
```bash
WEID=$(repslog workout create --type cardio | repslog workout-exercise add "Running" | tail -n 1)
repslog set add-cardio "$WEID" --distance 5.0 --duration 1500 --pace 5.0 \
  --hr-zones '{"z1_seconds":60,"z2_seconds":1200,"z3_seconds":240,"z4_seconds":0,"z5_seconds":0}' \
  --laps '[{"km":1,"time":"5:32","pace":"5:32"}]'
```

### 5. View PRs
```bash
repslog stats prs --exercise "Deadlift"
```

## References
- `references/cli-reference.md` — Full help output
- `references/cardio-json-examples.md` — Ready JSON snippets
- `references/example-outputs.md` — Real outputs + parsing tips
- `references/chaining-examples.md` — Advanced piping patterns
- `references/database-notes.md` — DB location and backup info

## Limitations
- Local SQLite only (no cloud sync)
- Optimized for terminal use

## Safety
Every mutating command supports `--dry-run`. Always verify before executing.
