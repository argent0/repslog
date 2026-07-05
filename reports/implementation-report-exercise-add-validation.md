# Implementation Report: Exercise Add Validation & Duplicate Prevention

**Date:** 2026-07-05  
**Feature:** Harden `repslog exercise add` against near-duplicate catalog entries.  
**Motivation:** Issue **5.7** in `reports/2026-07-05-workout-create-doc.md` — duplicate exercise names (e.g. `Pull Up`, `Pull Ups`, `Pullups`) fragment history and stats.  
**Author/Context:** Implemented via Grok interactive session in `/home/aner/rust/repslog`.

## 1. Overview and Motivation

The analyzed `repslog.db` contained 12 near-duplicate custom exercise entries alongside seeded catalog items. The report recommended using `repslog exercise search` before `exercise add`, but that guidance lived only in `docs/workouts.md` and was not enforced at the CLI entry point.

Prior `exercise add` behavior:

- Accepted any free-form string (including Title Case / CamelCase).
- Relied on SQLite `UNIQUE` on `exercises.name` for **exact** string matches only.
- No similarity checks, no naming conventions, no expanded `--help`.

This implementation adds **naming rules**, **duplicate rejection**, **similar-name warnings**, and **expanded CLI help** at the point of catalog insertion.

## 2. Design Decisions

- **Lowercase-only for new custom exercises**: Reject names containing uppercase letters with a corrective hint (`Pull Ups` → suggests `pull ups`). Seeded exercises from `init` retain their existing Title Case names; only the `exercise add` path enforces the rule.
- **Singular preference (advisory)**: Warn when a word looks plural (ends in `s`, length ≥ 3, not `ss` e.g. `press`). Suggest singular form (`pull ups` → `pull up`). Non-blocking — user can still proceed.
- **Duplicate detection (blocking)**: Compare a normalized **similarity key** (alphanumeric lowercase, strip trailing plural `s`) against the full catalog. If keys match, reject with an error pointing to the existing entry and `exercise search`.
- **Similar-name detection (advisory)**: Warn (stderr) when a new name is close to an existing entry but not an exact key match. Uses:
  - Substring containment on similarity keys (min length 5)
  - Levenshtein distance ≤ 1 on keys (both length ≥ 5)
  - Token overlap: ≥ 2 shared words at ≥ 80% of the smaller token set (catches `nordic curl` vs `nordic hamstring curl`)
- **No schema migration**: Pure validation layer; existing rows unchanged.
- **No auto-rename / merge command**: Out of scope; this change prevents new fragmentation rather than cleaning historical data.
- **Warnings on stderr, ID on stdout**: Preserves piping behavior for scripting (`WE_ID=$(repslog exercise add ...)` still works; warnings do not pollute stdout).

## 3. Code Changes

### `src/utils.rs`

New public helpers:

| Function | Purpose |
|----------|---------|
| `normalize_exercise_name` | Trim, collapse whitespace, reject uppercase |
| `suggest_singular_exercise_name` | Detect plural words; return singular suggestion |
| `exercise_similarity_key` | Alphanumeric lowercase key with plural stripping |
| `find_exercise_name_conflicts` | Return `Duplicate` or `Similar` conflicts against catalog |

Private helpers: `exercise_names_similar`, `exercise_name_tokens`, `exercise_name_tokens_overlap`, `levenshtein_distance`, `is_likely_plural_word`, `singularize_word`.

Unit tests cover normalization, plural suggestion, duplicate detection (`pull ups` vs seeded `Pullups`), and similar detection (`nordic curl` vs `Nordic Hamstring Curl`).

### `src/commands/exercise.rs`

`ExerciseAction::Add` handler now:

1. Normalizes and validates the name (lowercase).
2. Warns if plural form detected.
3. Loads full catalog and runs conflict detection.
4. Errors on duplicates; warns on similar names.
5. Inserts the normalized lowercase name.

### `src/cli.rs`

Expanded `ExerciseAction::Add` doc comment and `<NAME>` argument help:

- Names must be **lowercase and singular**
- Search-before-add workflow with examples
- Example: `repslog exercise add "bulgarian split squat" --category strength --equipment dumbbell`

### `tests/exercise_test.rs` (new)

Integration tests:

| Test | Verifies |
|------|----------|
| `test_exercise_add_rejects_uppercase` | Title Case rejected with lowercase hint |
| `test_exercise_add_rejects_near_duplicate_of_seeded` | `pull ups` blocked against seeded `Pullups` |
| `test_exercise_add_warns_on_similar_name` | `nordic curl` allowed with similar warning path |
| `test_exercise_add_warns_on_plural_name` | `ring dips` stored; plural warning emitted |
| `test_exercise_add_stores_normalized_lowercase` | Whitespace collapsed, lowercase stored |

## 4. CLI Behavior Examples

```bash
# Rejected — uppercase
repslog exercise add "Pull Ups" --category calisthenics
# Error: Exercise names must be lowercase. Use: pull ups

# Rejected — near-duplicate of seeded Pullups
repslog exercise add "pull ups" --category calisthenics
# Warning: Prefer singular exercise names (e.g. 'pull up' instead of 'pull ups').
# Error: Exercise already exists as 'Pullups' (id: 2). Use `repslog exercise search`...

# Allowed with warnings
repslog exercise add "nordic curl" --category strength
# Warning: 'nordic curl' is similar to existing 'nordic hamstring curl' (id: N)...
# Added exercise nordic curl with ID ...

# Clean add
repslog exercise add "bulgarian split squat" --category strength
# Added exercise bulgarian split squat with ID ...
```

## 5. Relationship to Issue 5.7

| Report finding | Mitigation in this change |
|----------------|---------------------------|
| Near-duplicate names split history | Duplicate key match blocks adds like `pull ups` when `Pullups` exists |
| No search-before-add at CLI | `--help` documents `exercise search` workflow |
| Inconsistent casing (`Calisthenics` vs `calisthenics`) | New custom names forced lowercase |
| Plural variants (`Pull Up` / `Pull Ups`) | Singular advisory warning; key normalization treats plural/singular as duplicate |
| 12 existing duplicates in sample DB | Not auto-merged; prevention only for new entries |

## 6. Testing & Verification

```bash
cargo test          # 11 unit + 5 exercise integration tests pass
cargo clippy -- -D warnings
```

All existing integration tests remain green. No documentation files were updated in this pass (CLI help is self-describing; `docs/workouts.md` already has an "Avoiding Exercise Duplicates" section).

## 7. Known Limitations & Follow-ups

- **Seeded exercise names** (`Pullups`, `Bench Press`, etc.) remain Title Case from `init`; `workout-exercise add` still requires exact name match — lowercase custom names won't alias seeded entries unless the user types the seeded spelling.
- **Similar warnings are non-blocking** — a determined user can still add `nordic curl` beside `nordic hamstring curl`.
- **No merge/rename command** — cleaning the 12 existing duplicates in production DBs would need a separate `exercise merge` or manual SQL migration.
- **Plural heuristic is simple** — words ending in `ss` (`press`, `class`) are excluded; irregular plurals (`feet`, `leaves`) are not handled.
- **Catalog scan is O(n)** — loads all exercises on each add; fine at current scale (~50 entries), may need indexed lookup if catalog grows large.

## 8. Files Changed

| File | Change |
|------|--------|
| `src/utils.rs` | Name normalization, similarity detection, unit tests |
| `src/commands/exercise.rs` | Validation pipeline in `Add` handler |
| `src/cli.rs` | Expanded `exercise add` help text |
| `tests/exercise_test.rs` | New integration test file |

## 9. Conclusion

`exercise add` is now the first line of defense against catalog fragmentation identified in the 2026-07-05 data-entry report. Lowercase and singular conventions are documented in `--help` and enforced or advised at runtime; near-duplicates are blocked before they enter the database. Historical duplicates remain a separate cleanup task.