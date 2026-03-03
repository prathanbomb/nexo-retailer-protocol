#!/bin/bash
# Dependency audit for no_std compatibility
# Checks all feature combinations for std leakage
#
# Usage: ./scripts/audit_std_leak.sh [--all-features]
#
# Exit codes:
#   0 - All checks passed
#   1 - std leakage detected

set -e

TARGET=thumbv7em-none-eabihf
FAILED=0

echo "========================================="
echo "Dependency Audit for no_std Compatibility"
echo "========================================="
echo ""
echo "Target: $TARGET"
echo ""

# Function to check for std leakage
check_std_leak() {
    local FEATURES="$1"
    local DESCRIPTION="$2"

    echo "Checking: $DESCRIPTION"
    echo "  Command: cargo tree --target $TARGET $FEATURES"

    if cargo tree --target $TARGET $FEATURES 2>&1 | grep -iE "(std|standard)" | grep -v "no_std" | grep -v "nostd"; then
        echo ""
        echo "  ERROR: std dependency found!"
        echo "  This will cause compilation to fail on bare-metal targets."
        return 1
    else
        echo "  PASS: No std dependencies found"
        return 0
    fi
}

# Check for HashMap (requires Random trait)
check_hashmap() {
    local FEATURES="$1"
    local DESCRIPTION="$2"

    echo ""
    echo "Checking HashMap in: $DESCRIPTION"

    if cargo tree --target $TARGET $FEATURES 2>&1 | grep -i "hashmap"; then
        echo "  WARNING: HashMap found (may not work in no_std without Random trait)"
        echo "  Consider using BTreeMap instead"
        # This is a warning, not an error
    else
        echo "  PASS: No HashMap found"
    fi
}

echo "========================================="
echo "Feature Combination Checks"
echo "========================================="
echo ""

# Check 1: Pure no_std (no features)
if ! check_std_leak "--no-default-features" "Pure no_std (no features)"; then
    FAILED=1
fi

# Check 2: no_std with alloc
if ! check_std_leak "--no-default-features --features alloc" "no_std with alloc"; then
    FAILED=1
fi

# Check 3: no_std with embassy
if ! check_std_leak "--no-default-features --features embassy,alloc" "no_std with embassy"; then
    FAILED=1
fi

echo ""
echo "========================================="
echo "HashMap Checks (Warnings)"
echo "========================================="

check_hashmap "--no-default-features" "Pure no_std"
check_hashmap "--no-default-features --features alloc" "no_std with alloc"
check_hashmap "--no-default-features --features embassy,alloc" "no_std with embassy"

echo ""
echo "========================================="
echo "Dev Dependencies Check"
echo "========================================="
echo ""

# Check that dev-dependencies don't leak std into no_std builds
echo "Checking dev-dependencies don't leak std..."
if cargo tree --target $TARGET --no-default-features --dev-dependencies 2>&1 | grep -iE "(std|standard)" | grep -v "no_std" | grep -v "nostd"; then
    echo "  WARNING: Some dev-dependencies may use std"
    echo "  This is OK for dev-dependencies as they're not included in production builds"
else
    echo "  PASS: No std leakage from dev-dependencies"
fi

echo ""
echo "========================================="
if [ $FAILED -eq 0 ]; then
    echo "All checks passed!"
    echo "========================================="
    exit 0
else
    echo "FAILED: std leakage detected!"
    echo "========================================="
    exit 1
fi
