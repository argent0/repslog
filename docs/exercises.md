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
repslog exercise add "bulgarian split squat" \
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

### Rep phase belongs on sets, not exercise names

Sets require `--phase full|eccentric|concentric` when logging (see [logging.md](logging.md)).
Use **one exercise per movement** and tag individual sets with the appropriate phase.

```bash
# Good: one pistol squat exercise, phase on each set
repslog exercise add "pistol squat" --category calisthenics --load-type body_mass
repslog set add $WE --reps 3 --weight 82 --phase eccentric
repslog set add $WE --reps 5 --weight 82 --phase full
```

`exercise add` **rejects** names that embed phase information (e.g. `pistol squat (eccentric only)`, `concentric press`). This keeps history and stats under a single exercise instead of splitting across variants.

To override (legacy imports only):

```bash
repslog exercise add "pistol squat (eccentric only)" \
  --category calisthenics \
  --load-type body_mass \
  --allow-phase-in-name
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
- `--allow-phase-in-name`: Skip the check that rejects eccentric/concentric in the exercise name (not recommended).

`exercise update` accepts the same optional fields plus `--clear-equipment` to remove apparatus.

## Exercise Categories

Common categories used in `repslog`:
- **strength**: Traditional weightlifting.
- **calisthenics**: Bodyweight movements.
- **cardio**: running, cycling, swimming, etc. (all exercise names are lowercase)
- **flexibility**: Stretching and mobility work.
- **hiit**: High-intensity interval training.
