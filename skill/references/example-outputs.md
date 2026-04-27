# repslog Example Outputs

This document shows representative outputs for common `repslog` commands.

## Exercise List
```text
$ repslog exercise list
+----+-----------------+--------------+----------------------------------------+------------+
| ID | Name            | Category     | Muscles                                | Equipment  |
+===========================================================================================+
| 1  | Pushups         | calisthenics | ["chest", "triceps", "shoulders"]      | bodyweight |
|----+-----------------+--------------+----------------------------------------+------------|
| 2  | Pullups         | calisthenics | ["back", "biceps"]                     | bodyweight |
|----+-----------------+--------------+----------------------------------------+------------|
| 3  | Dips            | calisthenics | ["chest", "triceps", "shoulders"]      | bodyweight |
|----+-----------------+--------------+----------------------------------------+------------|
...
| 11 | Running         | cardio       | ["legs", "cardiovascular"]             | none       |
+----+-----------------+--------------+----------------------------------------+------------+
```

## Creating a Workout
```text
$ repslog workout create --date 2026-04-27 --type strength
Created workout with ID 10
10
```

## Adding an Exercise to a Workout
```text
$ repslog workout-exercise add 10 Pushups
Added exercise Pushups (ID: 1) to workout 10 with WE ID 9
9
```

## Adding a Strength Set
```text
$ repslog set add 9 --reps 10 --weight 60 --rir 0 --effective-reps 5
Added set 1 to workout-exercise 9 with set ID 13
13
```

## Adding a Cardio Set
```text
$ repslog set add-cardio 12 --distance 5.0 --duration 1500 --avg-heart-rate 155 --max-heart-rate 180 --pace 5.0 --calories 450 --hr-zones '{"z1_seconds": 60, "z2_seconds": 1200, "z3_seconds": 240, "z4_seconds": 0, "z5_seconds": 0}' --laps '[{"km": 1, "time": "5:32", "pace": "5:32"}]'
Added cardio set to workout-exercise 12 with set ID 14
14
```

## Personal Records (PRs)
```text
$ repslog stats prs --exercise Pushups
+----------+-------+--------+-----+----------------+
| Date     | Reps  | Weight | RIR | Effective Reps |
+==================================================+
| 2026-04-20 | 15   | 0      | 0   | 8              |
+----------+-------+--------+-----+----------------+
```
*(Note: Output formats may vary slightly depending on your terminal width and colors)*
