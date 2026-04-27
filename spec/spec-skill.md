**Instructions for the Coding Agent:**

You are an expert coding agent specialized in creating reusable "skills" for LLM agents (modeled exactly after Claude-style code skills). Your task is to create a complete, self-contained **skill** for the `repslog` command-line workout tracker tool (https://github.com/argent0/repslog).

**Skill requirements (strict):**
- **Name**: `workout-tracker` (or `workout-tracker` if you prefer a clearer title).
- **Purpose**: Enable LLM agents to fully interact with the installed `repslog` CLI to log, manage, query, and analyze workouts (strength training, calisthenics, cardio) using only direct shell/terminal execution of the `repslog` binary. 
- **No Python wrapper** of any kind. Do not create any Python classes, functions, or wrapper scripts. The skill must instruct LLM agents to invoke the tool directly via `repslog <subcommand> ...` (or via `subprocess.run` / shell execution in their own code if they are writing scripts). Assume the binary is already installed and available in `$PATH`.
- **Structure** (exact):
  - `SKILL.md` (the main skill definition file)
  - `references/` directory containing supporting reference files

**Step-by-step process you must follow:**

1. **Verify the tool is installed and gather live data**  
   Run these commands in your environment and capture the exact output:
   - `repslog --help`
   - `repslog exercise --help`
   - `repslog workout --help`
   - `repslog workout-exercise --help`
   - `repslog set --help`
   - `repslog stats --help`
   - `repslog migrate --help`
   - Any other subcommand help you discover (e.g., `repslog init --help`).
   Also run `repslog --version` and note the install location of the database (`~/.local/share/repslog/`).

2. **Create the skill files**

   **SKILL.md** must contain these sections (use clear markdown, code blocks, and tables):
   - **Skill Name**
   - **Description** (one-paragraph summary of what repslog is and why the skill exists)
   - **Capabilities** (bullet list of all major features: workout creation, exercise management, strength sets, cluster sets, cardio with HR zones/laps, stats/PRs/volume, migrations, etc.)
   - **Prerequisites** (tool is installed; database is XDG-compliant at `~/.local/share/repslog/repslog.db`; Linux-first)
   - **Core Usage Principles for LLM Agents** (very important):
     - Always prefer `--dry-run` on mutating commands first.
     - Use stdin piping / ID extraction for chaining commands (this is a core strength of the tool).
     - How to safely parse command output (IDs, tables, JSON where applicable).
     - Handling complex flags (JSON for `--hr-zones` and `--laps`).
     - Error handling and common pitfalls.
   - **Command Reference Summary** (high-level table or categorized list of the most important subcommands with one-line descriptions)
   - **Best Practices & Patterns** (specific guidance for agents)
   - **Common Workflows / Examples** (at least 6–8 fully worked examples covering):
     - Initialize DB
     - Quick exercise + set
     - Full strength workout with multiple exercises/sets
     - Cardio workout with laps and HR zones
     - Cluster/rest-pause set
     - View stats / PRs
     - Update a workout
     - Scripted multi-step workflow using piped IDs
   - **Limitations** (local-only, Linux-first, no cloud sync, etc.)
   - **Safety & Idempotency** (heavy emphasis on `--dry-run`)

   **references/** directory (create these files with accurate content):
   - `cli-reference.md` — Paste the full `--help` output for the root command and all major subcommands.
   - `example-outputs.md` — Include real or representative command outputs and tables (from your verification runs).
   - `cardio-json-examples.md` — Ready-to-use JSON snippets for `--hr-zones` and `--laps`.
   - `chaining-examples.md` — Additional advanced stdin-piping and scripting patterns.
   - `database-notes.md` — Location, backup advice, migration info.

3. **Quality standards**
   - All examples must be realistic, correct, and tested/verified against the live tool where possible.
   - Use proper KaTeX-style code fences for shell commands.
   - Make the skill extremely agent-friendly: clear, concise, copy-pasteable commands, explicit ID extraction steps.
   - Emphasize scriptability (the tool is designed for chaining).
   - Keep language neutral and professional.
   - Do not invent features — stick strictly to what the actual `repslog` binary provides.

4. **Final output**
   Once complete, present the full content of `SKILL.md` and all files inside `references/` in your response (or save them to disk if your environment supports it), clearly labeled. Also include a short note confirming you verified the CLI help outputs.

Start by running the help commands to gather data, then build the skill. This skill will be used by LLM agents to manage personal workout logging entirely through the repslog CLI.
