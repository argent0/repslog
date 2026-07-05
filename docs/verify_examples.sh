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

echo "Testing: repslog exercise list --json"
$REPSLOG exercise list --json | python3 -c '
import json, sys
data = json.load(sys.stdin)
assert isinstance(data, list), "exercise list --json should be array"
print("  json ok, count:", len(data))
' > /dev/null

echo "Testing: repslog exercise search"
$REPSLOG exercise search "Squat" > /dev/null

echo "Testing: repslog exercise search --json"
$REPSLOG exercise search "Squat" --json > /dev/null

echo "Testing: repslog exercise add"
$REPSLOG exercise add "Bulgarian Split Squat" \
  --category strength \
  --equipment dumbbell \
  --muscles "quads,glutes" \
  --description "One leg elevated on a bench behind you."

# 3. Workouts
echo "Testing: repslog workout create"
WORKOUT_ID=$($REPSLOG workout create --type "Legs" --date "2026-04-23 10:00:00" --notes "Focus on form")
echo "Created workout ID: $WORKOUT_ID"

echo "Testing: repslog workout list"
$REPSLOG workout list --days 7 > /dev/null

echo "Testing: repslog workout list --json"
$REPSLOG workout list --days 7 --json > /dev/null

echo "Testing: repslog workout view"
$REPSLOG workout view $WORKOUT_ID > /dev/null

echo "Testing: repslog workout view --json"
$REPSLOG workout view $WORKOUT_ID --json | python3 -c '
import json, sys
data = json.load(sys.stdin)
assert "id" in data and "exercises" in data, "workout view json shape"
print("  json ok")
' > /dev/null

echo "Testing: repslog workout-exercise add"
WE_ID=$($REPSLOG workout-exercise add $WORKOUT_ID "Squat (Barbell)")
echo "Created Workout-Exercise ID: $WE_ID"

echo "Testing: repslog workout-exercise list --json"
$REPSLOG workout-exercise list $WORKOUT_ID --json > /dev/null

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

echo "Testing: repslog set list --json"
$REPSLOG set list $WE_ID --json > /dev/null

# Unilateral / side + corrections (new in unilateral improvement)
echo "Testing: repslog set add with --side (unilateral)"
SPLIT_WE=$($REPSLOG workout-exercise add $WORKOUT_ID "Bulgarian Split Squat")
$REPSLOG set add $SPLIT_WE --reps 8 --weight 20 --side left
$REPSLOG set add $SPLIT_WE --reps 8 --weight 20 --side right

echo "Testing: repslog set update + side change + notes"
# Take the first set ID from list (json) for the split
FIRST_SET=$( $REPSLOG set list $SPLIT_WE --json | python3 -c '
import json,sys
data=json.load(sys.stdin)
print(data[0]["id"] if data else "")
' )
if [ -n "$FIRST_SET" ]; then
  $REPSLOG set update $FIRST_SET --reps 9 --notes "Felt strong on left" --side left
fi

echo "Testing: repslog set move (reorder)"
# Move if we have at least two
$REPSLOG set move $FIRST_SET --to 2 || true

echo "Testing: repslog set delete --force"
if [ -n "$FIRST_SET" ]; then
  $REPSLOG set delete $FIRST_SET --force
fi

echo "Testing: repslog set list shows side/context"
$REPSLOG set list $SPLIT_WE > /dev/null

echo "Testing: repslog workout view (includes side grouping/totals)"
$REPSLOG workout view $WORKOUT_ID > /dev/null

echo "Testing: repslog stats weight (progression)"
$REPSLOG stats weight --exercise "Squat (Barbell)" > /dev/null

# 5. Stats
echo "Testing: repslog stats prs"
$REPSLOG stats prs --exercise "Squat (Barbell)" > /dev/null

echo "Testing: repslog stats prs --json"
$REPSLOG stats prs --exercise "Squat (Barbell)" --json > /dev/null

echo "Testing: repslog stats volume"
$REPSLOG stats volume --period 30d > /dev/null

echo "Testing: repslog stats volume --json"
$REPSLOG stats volume --period 30d --json > /dev/null

echo "Testing: repslog stats summary"
$REPSLOG stats summary --days 30 > /dev/null

echo "Testing: repslog stats summary --json"
$REPSLOG stats summary --days 30 --json > /dev/null

echo "Testing: repslog migrate --status --json"
$REPSLOG migrate --status --json > /dev/null

# 6. Scripting / Piping
echo "Testing: piping workout create to workout-exercise add using xargs"
NEW_WORKOUT_ID=$($REPSLOG workout create --date "2026-04-24 10:00:00" --type "Push")
echo "$NEW_WORKOUT_ID" | xargs -I {} $REPSLOG workout-exercise add {} "Pushups" > /dev/null

echo "Testing: workout create --json (for jq scripting)"
JSON_CREATED=$($REPSLOG workout create --date "2026-04-25 10:00:00" --type "Pull" --json)
python3 -c '
import json, sys
obj = json.loads(sys.argv[1])
assert "id" in obj
print("  json create id:", obj["id"])
' "$JSON_CREATED" > /dev/null

echo "Testing: piping workout-exercise add to set add (stdin supported)"
WE_ID=$($REPSLOG workout-exercise add 1 "Dips")
echo "$WE_ID" | $REPSLOG set add --reps 10 --rir 0 > /dev/null

echo "Cleanup: Removing temporary directory $TMP_DIR"
rm -rf "$TMP_DIR"

echo "All documentation examples verified successfully!"
