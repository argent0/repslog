#!/bin/bash
set -e

# verify_examples.sh
# This script verifies the examples provided in the documentation.

# Setup a temporary directory for the test database
export TMP_DIR=$(mktemp -d)
export XDG_DATA_HOME="$TMP_DIR"
export REPSLOG_BIN="./target/debug/repslog"

echo "Verifying documentation examples using temporary XDG_DATA_HOME: $XDG_DATA_HOME"

# Ensure we are using the local build
REPSLOG="$REPSLOG_BIN"

# 1. Initialization
echo "Testing: repslog init"
$REPSLOG init

# 2. Exercise Management
echo "Testing: repslog exercise list"
$REPSLOG exercise list > /dev/null

echo "Testing: repslog exercise search"
$REPSLOG exercise search "Squat" > /dev/null

echo "Testing: repslog exercise add"
$REPSLOG exercise add "Bulgarian Split Squat" \
  --category strength \
  --equipment dumbbell \
  --muscles "quads,glutes" \
  --description "One leg elevated on a bench behind you."

# 3. Workouts
echo "Testing: repslog workout create"
WORKOUT_ID=$($REPSLOG workout create --type "Legs" --date "2026-04-23" --notes "Focus on form")
echo "Created workout ID: $WORKOUT_ID"

echo "Testing: repslog workout list"
$REPSLOG workout list --days 7 > /dev/null

echo "Testing: repslog workout view"
$REPSLOG workout view $WORKOUT_ID > /dev/null

echo "Testing: repslog workout-exercise add"
WE_ID=$($REPSLOG workout-exercise add $WORKOUT_ID "Squat (Barbell)")
echo "Created Workout-Exercise ID: $WE_ID"

# 4. Logging Sets
echo "Testing: repslog set add"
$REPSLOG set add $WE_ID --reps 10 --weight 100 --rir 1.0 --effective-reps 5

echo "Testing: repslog set add-cluster"
$REPSLOG set add-cluster $WE_ID \
  --reps "10,5,5" \
  --weight 100 \
  --rir "0,0,1" \
  --effective-reps "6,4,3" \
  --rest 15

echo "Testing: repslog set add-cardio"
# Need to add Running exercise first or use the default if it exists
RUN_WE_ID=$($REPSLOG workout-exercise add $WORKOUT_ID "Running")
$REPSLOG set add-cardio $RUN_WE_ID \
  --distance 5 \
  --duration 1500 \
  --avg-heart-rate 150 \
  --max-heart-rate 170 \
  --pace 5.0 \
  --calories 300 \
  --hr-zones '{"z1_seconds": 300, "z2_seconds": 600, "z3_seconds": 600}'

# 5. Stats
echo "Testing: repslog stats prs"
$REPSLOG stats prs --exercise "Squat (Barbell)" > /dev/null

echo "Testing: repslog stats volume"
$REPSLOG stats volume --period 30d > /dev/null

echo "Testing: repslog stats summary"
$REPSLOG stats summary --days 30 > /dev/null

# 6. Scripting / Piping
echo "Testing: piping workout create to workout-exercise add using xargs"
NEW_WORKOUT_ID=$($REPSLOG workout create --date "2026-04-24" --type "Push")
echo "$NEW_WORKOUT_ID" | xargs -I {} $REPSLOG workout-exercise add {} "Pushups" > /dev/null

echo "Testing: piping workout-exercise add to set add (stdin supported)"
WE_ID=$($REPSLOG workout-exercise add 1 "Dips")
echo "$WE_ID" | $REPSLOG set add --reps 10 --rir 0 > /dev/null

echo "Cleanup: Removing temporary directory $TMP_DIR"
rm -rf "$TMP_DIR"

echo "All documentation examples verified successfully!"
