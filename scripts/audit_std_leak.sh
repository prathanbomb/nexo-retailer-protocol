#!/bin/bash
set -e

TARGET=thumbv7em-none-eabihf

echo "========================================="
echo "Dependency Audit for no_std Compatibility"
echo "========================================="
echo ""
echo "Target: $TARGET"
echo ""

echo "Checking for std leakage in no_std build..."
if cargo tree --target $TARGET --no-default-features | grep -i "std"; then
    echo ""
    echo "❌ ERROR: std dependency found in no_std build!"
    echo "This will cause compilation to fail on bare-metal targets."
    exit 1
else
    echo "✓ PASS: No std dependencies found"
fi

echo ""
echo "Checking for HashMap (requires Random trait)..."
if cargo tree --target $TARGET --no-default-features | grep -i "hashmap"; then
    echo "⚠ WARNING: HashMap found (may not work in no_std without Random trait)"
    echo "Consider using BTreeMap instead"
else
    echo "✓ PASS: No HashMap found"
fi

echo ""
echo "========================================="
echo "✓ All checks passed!"
echo "========================================="
