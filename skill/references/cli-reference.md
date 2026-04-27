# repslog CLI Reference

This document contains the full `--help` output for `repslog` and its major subcommands.

## Root Command
```text
A Linux-first workout tracker

Usage: repslog <COMMAND>

Commands:
  exercise          Exercise management
  workout           Workout management
  session           Alias for workout management
  workout-exercise  Manage exercises within a workout
  set               Manage sets within a workout exercise
  stats             View statistics
  migrate           Database migrations
  init              Initialize database and seed default exercises
  help              Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

## Exercise Management (`repslog exercise`)
```text
Usage: repslog exercise <COMMAND>

Commands:
  list    List all exercises
  add     Add a new exercise
  search  Search for exercises
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

### `repslog exercise add`
```text
Usage: repslog exercise add [OPTIONS] --category <CATEGORY> <NAME>

Arguments:
  <NAME>  

Options:
  -c, --category <CATEGORY>        
  -e, --equipment <EQUIPMENT>      
  -m, --muscles <MUSCLES>          
  -d, --description <DESCRIPTION>  
      --dry-run                    Show what would be added (no changes)
  -h, --help                       Print help
```

## Workout Management (`repslog workout`)
```text
Usage: repslog workout <COMMAND>

Commands:
  create  Create a new workout
  list    List workouts
  view    View details of a specific workout
  update  Update a workout
  delete  Delete a workout
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

### `repslog workout create`
```text
Usage: repslog workout create [OPTIONS] --date <DATE>

Options:
  -w, --type <WORKOUT_TYPE>  
  -n, --notes <NOTES>        
  -d, --date <DATE>          
      --dry-run              Show what would be created (no changes)
  -h, --help                 Print help
```

## Workout-Exercise Management (`repslog workout-exercise`)
```text
Usage: repslog workout-exercise <COMMAND>

Commands:
  add   Add an exercise to a workout
  list  List exercises in a workout
  help  Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

### `repslog workout-exercise add`
```text
Usage: repslog workout-exercise add [OPTIONS] <WORKOUT_ID> <EXERCISE_ID_OR_NAME>

Arguments:
  <WORKOUT_ID>           
  <EXERCISE_ID_OR_NAME>  

Options:
  -o, --order <ORDER>  
      --dry-run        Show what would be added (no changes)
  -h, --help           Print help
```

## Set Management (`repslog set`)
```text
Usage: repslog set <COMMAND>

Commands:
  add          Add a set to a workout exercise
  add-cardio   Add a cardio set with mandatory heart rate and pace metrics
  add-cluster  Add a rest-pause/cluster set sequence
  list         List sets for a workout exercise
  quick        Convenience: add exercise + first set in one go
  help         Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

### `repslog set add`
```text
Usage: repslog set add [OPTIONS] [WORKOUT_EXERCISE_ID]

Arguments:
  [WORKOUT_EXERCISE_ID]  

Options:
  -r, --reps <REPS>
  -w, --weight <WEIGHT>
  -d, --duration <DURATION>
      --distance <DISTANCE>
      --rpe <RPE>
      --rir <RIR>
      --effective-reps <EFFECTIVE_REPS>
      --rest <REST_SECONDS>
  -n, --notes <NOTES>
      --avg-heart-rate <AVG_HEART_RATE>
      --max-heart-rate <MAX_HEART_RATE>
      --hr-zones <HR_ZONES>
      --pace <PACE>
      --calories <CALORIES>
      --laps <LAPS>
      --dry-run
  -h, --help
```

## Stats (`repslog stats`)
```text
Usage: repslog stats <COMMAND>

Commands:
  prs      Personal records
  volume   Training volume
  summary  Training summary
  help     Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```
