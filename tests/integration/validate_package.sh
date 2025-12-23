#!/usr/bin/env bash
# Test: Validate packaging script outputs correct artifacts
# Purpose: Ensure release packaging produces all required files

set -e

echo "Testing packaging script outputs..."
echo "--------------------------------------------------"

# Define expected artifacts
EXPECTED_ARTIFACTS=(
    "costpilot"
    "costpilot.wasm"
    "release_metadata.json"
    "README.md"
    "LICENSE"
)

PACKAGE_DIR="products/costpilot/target/release"
MISSING_ARTIFACTS=()

# Check each expected artifact
for artifact in "${EXPECTED_ARTIFACTS[@]}"; do
    if [ -f "$PACKAGE_DIR/$artifact" ] || [ -f "products/costpilot/$artifact" ]; then
        echo "✓ Found: $artifact"
    else
        echo "✗ Missing: $artifact"
        MISSING_ARTIFACTS+=("$artifact")
    fi
done

# Report results
echo "--------------------------------------------------"
if [ ${#MISSING_ARTIFACTS[@]} -eq 0 ]; then
    echo "✅ All packaging artifacts present"
    exit 0
else
    echo "❌ Missing artifacts: ${MISSING_ARTIFACTS[*]}"
    exit 1
fi
