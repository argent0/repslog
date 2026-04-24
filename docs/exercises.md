# Managing Exercises

Exercises are the building blocks of your workouts. `repslog` comes with a set of default exercises, but you can easily add your own.

## Listing Exercises

To see all available exercises:

```bash
repslog exercise list
```

You can filter the list by category:

```bash
repslog exercise list --category strength
```

## Searching for Exercises

If you have a long list of exercises, you can search for specific terms:

```bash
repslog exercise search "Squat"
```

## Adding Custom Exercises

You can add custom exercises to fit your training routine.

```bash
repslog exercise add "Bulgarian Split Squat" \
  --category strength \
  --equipment dumbbell \
  --muscles "quads,glutes" \
  --description "One leg elevated on a bench behind you."
```

### Parameters

- `<NAME>`: The name of the exercise (required, unique).
- `--category <CAT>`: The type of exercise (e.g., strength, cardio, calisthenics, flexibility).
- `--equipment <EQ>`: The equipment needed (e.g., barbell, dumbbell, bodyweight, machine).
- `--muscles <MUSCLES>`: A comma-separated list of muscle groups.
- `--description <DESC>`: A brief explanation of the exercise.

## Exercise Categories

Common categories used in `repslog`:
- **strength**: Traditional weightlifting.
- **calisthenics**: Bodyweight movements.
- **cardio**: Running, cycling, swimming, etc.
- **flexibility**: Stretching and mobility work.
- **hiit**: High-intensity interval training.
