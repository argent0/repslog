# Cardio JSON Examples

The `repslog set add-cardio` command requires JSON input for `--hr-zones` and `--laps`.

## Heart Rate Zones (`--hr-zones`)
The JSON should contain durations in seconds for each zone (Z1 to Z5).

### Basic Example
```json
{
  "z1_seconds": 300,
  "z2_seconds": 1200,
  "z3_seconds": 600,
  "z4_seconds": 300,
  "z5_seconds": 0
}
```

### High Intensity Example
```json
{
  "z1_seconds": 60,
  "z2_seconds": 240,
  "z3_seconds": 600,
  "z4_seconds": 1200,
  "z5_seconds": 300
}
```

## Laps (`--laps`)
The laps JSON is an array of objects.

### Simple Laps
```json
[
  {
    "km": 1,
    "time": "5:32",
    "pace": "5:32"
  },
  {
    "km": 2,
    "time": "5:15",
    "pace": "5:15"
  },
  {
    "km": 3,
    "time": "5:45",
    "pace": "5:45"
  }
]
```

### Full Data Laps
```json
[
  {
    "lap_number": 1,
    "distance_km": 1.0,
    "duration_seconds": 332,
    "pace_min_per_km": 5.533
  },
  {
    "lap_number": 2,
    "distance_km": 1.0,
    "duration_seconds": 315,
    "pace_min_per_km": 5.25
  }
]
```
*(Note: `repslog` is flexible with lap JSON field names as long as they are identifiable, but the first format is the most common for manual entry.)*
