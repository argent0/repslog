# repslog Feature Specification

**Date**: 2026-06-05  
**Author**: Aner (via AI assistant)  
**Goal**: Improve repslog for real-world unilateral strength training workflows, especially corrections and left/right tracking.

## 1. Set Management Commands

### 1.1 `repslog set update`
- Update any field on an existing set (reps, weight, notes, rir, effective-reps, rest, etc.)
- Usage example:
  ```bash
  repslog set update 287 --reps 10 --weight 20 --notes "Left leg"
  ```

### 1.2 `repslog set delete`
- Delete a specific set by ID
- Usage:
  ```bash
  repslog set delete 287
  ```
- Should ask for confirmation unless `--force`

### 1.3 `repslog set move`
- Reorder sets within a workout-exercise (change set number / display order)
- Useful after corrections or when adding sets out of order

## 2. Unilateral / Side Tracking

### 2.1 `--side` flag on `set add`
- New option: `--side left|right|both`
- Automatically prefixes notes or stores structured side data
- Example:
  ```bash
  repslog set add 84 --reps 6 --side left
  repslog set add 84 --reps 6 --side right
  ```

### 2.2 Per-side exercise view
- When viewing a workout, group or clearly separate Left vs Right sets
- Optional: Show totals per side (e.g. "Left: 38 reps | Right: 38 reps")

### 2.3 Unilateral exercise templates
- Quick command to add matching left + right sets in one go:
  ```bash
  repslog set add-unilateral 83 --reps "8,10,10,10" --side both
  ```

## 3. Weight Tracking

### 3.1 Full `--weight` support
- Make `--weight` a first-class field alongside `--reps`
- Allow weight-only sets (e.g. for progressive overload where reps are not the focus)
- Display both weight and reps clearly in `workout view`

### 3.2 Weight history per exercise
- `repslog stats` or new command to show weight progression over time for an exercise

## 4. Total Reps Support

### 4.2 Goal vs Actual tracking
- Store `goal_reps` at the workout-exercise level (already partially done via notes)
- Show progress toward goal in the view

## 5. Improved Display & Organization

### 5.1 Sorted set display
- `workout view` should always show sets in logical order (by side then set number, or by insertion order with clear grouping)

### 5.2 Better notes and metadata
- Support `--tempo "3s eccentric"` or `--rest 15` at set level
- Display tempo/rest clearly in the table

### 5.3 Exercise-level notes
- Allow notes on the `workout-exercise` record itself (not just per set)

## 6. Bulk / Ergonomic Commands

### 6.1 `repslog set add-cluster` enhancement
- Extend for unilateral work

### 6.2 Quick unilateral session helper
- New top-level command or flag:
  ```bash
  repslog workout add-unilateral-session --date 2026-06-04
  ```
  This could walk through common unilateral exercises or accept a template.

## 7. Other Quality-of-Life Improvements

- `repslog set list <WE_ID>` should show more context (exercise name, workout date)
- Dry-run mode on more destructive commands (`delete`, `update`)
- Better error messages when required fields are missing
- Export workout to JSON/CSV for external analysis

## Priority Recommendation

**High priority** (biggest friction in current workflow):
1. `set update` + `set delete`
2. `--side` flag + improved unilateral display
3. Full `--weight` support

**Medium priority**:
- Better sorting and tempo fields

---

*This spec was generated from a real logging session involving heavy unilateral lower body work with multiple corrections.*
