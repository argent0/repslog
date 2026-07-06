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
  --load-type external \
  --muscles "quads,glutes" \
  --description "One leg elevated on a bench behind you."
```

Ring dips use apparatus and load semantics separately:

```bash
repslog exercise add "ring dip" \
  --category calisthenics \
  --equipment rings \
  --load-type body_mass
```

Update metadata on an existing exercise:

```bash
repslog exercise update "ring dip" --equipment rings --load-type body_mass
```

### Parameters

- `<NAME>`: The name of the exercise (required, unique).
- `--category <CAT>`: The type of exercise (e.g., strength, cardio, calisthenics, flexibility).
- `--equipment <EQ>`: Apparatus used (e.g., barbell, dumbbell, rings, parallel bars, none).
- `--load-type <TYPE>`: How `--weight` is interpreted: `body_mass`, `external`, or `none`. Defaults from category when omitted (`calisthenics` → `body_mass`, `cardio` → `none`, otherwise `external`).
- `--muscles <MUSCLES>`: A comma-separated list of muscle groups.
- `--description <DESC>`: A brief explanation of the exercise.

`exercise update` accepts the same optional fields plus `--clear-equipment` to remove apparatus.

## Exercise Categories

Common categories used in `repslog`:
- **strength**: Traditional weightlifting.
- **calisthenics**: Bodyweight movements.
- **cardio**: Running, cycling, swimming, etc.
- **flexibility**: Stretching and mobility work.
- **hiit**: High-intensity interval training.
