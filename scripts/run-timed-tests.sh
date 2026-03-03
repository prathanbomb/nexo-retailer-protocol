#!/bin/bash
# Run tests with execution time monitoring
# Usage: ./scripts/run-timed-tests.sh [--std|--alloc|--embassy] [additional cargo test args]
#
# Exit codes:
#   0 - Tests passed within time threshold
#   1 - Tests failed
#   2 - Tests passed but exceeded time threshold

set -e

# Default configuration
THRESHOLD_SECONDS=${THRESHOLD_SECONDS:-600}  # 10 minutes default
FEATURE_SET="std"
EXTRA_ARGS=""

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --std)
            FEATURE_SET="std"
            shift
            ;;
        --alloc)
            FEATURE_SET="alloc"
            shift
            ;;
        --embassy)
            FEATURE_SET="embassy,alloc"
            shift
            ;;
        --threshold)
            THRESHOLD_SECONDS="$2"
            shift 2
            ;;
        *)
            EXTRA_ARGS="$EXTRA_ARGS $1"
            shift
            ;;
    esac
done

# Determine cargo test command based on feature set
if [ "$FEATURE_SET" = "std" ]; then
    CARGO_CMD="cargo test --features std $EXTRA_ARGS"
elif [ "$FEATURE_SET" = "alloc" ]; then
    CARGO_CMD="cargo test --no-default-features --features alloc $EXTRA_ARGS"
elif [ "$FEATURE_SET" = "embassy,alloc" ]; then
    CARGO_CMD="cargo test --features embassy,alloc $EXTRA_ARGS"
else
    CARGO_CMD="cargo test --features $FEATURE_SET $EXTRA_ARGS"
fi

echo "========================================="
echo "Test Execution with Time Monitoring"
echo "========================================="
echo ""
echo "Feature set: $FEATURE_SET"
echo "Time threshold: ${THRESHOLD_SECONDS}s ($((THRESHOLD_SECONDS / 60)) minutes)"
echo "Command: $CARGO_CMD"
echo ""

# Record start time
START_TIME=$(date +%s)
echo "Starting test execution at $(date)"

# Run tests
set +e
$CARGO_CMD
TEST_EXIT_CODE=$?
set -e

# Record end time
END_TIME=$(date +%s)
DURATION=$((END_TIME - START_TIME))
MINUTES=$((DURATION / 60))
SECONDS=$((DURATION % 60))

echo ""
echo "========================================="
echo "Test Execution Summary"
echo "========================================="
echo "Duration: ${MINUTES}m ${SECONDS}s (${DURATION}s)"
echo "Exit code: $TEST_EXIT_CODE"
echo ""

# Check time threshold
if [ $DURATION -gt $THRESHOLD_SECONDS ]; then
    echo "WARNING: Tests exceeded ${THRESHOLD_SECONDS}s threshold"
    if [ $TEST_EXIT_CODE -eq 0 ]; then
        echo "Tests passed but took too long"
        exit 2
    fi
else
    echo "Test execution time within acceptable threshold"
fi

exit $TEST_EXIT_CODE
