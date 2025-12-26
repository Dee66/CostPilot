#!/usr/bin/env bash
set -euo pipefail

# Test for SBOM generation
# Validates that the generate_sbom.sh script creates a valid SPDX file

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
TEST_DIR=$(mktemp -d)
trap "rm -rf '$TEST_DIR'" EXIT

SBOM_SCRIPT="${PROJECT_ROOT}/scripts/sbom/generate_sbom.sh"
SOURCE_DIR="${PROJECT_ROOT}/src"
OUTPUT_FILE="${TEST_DIR}/sbom.spdx.json"

if [[ ! -x "$SBOM_SCRIPT" ]]; then
  echo "ERROR: SBOM script not found or not executable: $SBOM_SCRIPT" >&2
  exit 1
fi

# Run SBOM generation
bash "$SBOM_SCRIPT" "$SOURCE_DIR" "$OUTPUT_FILE"

# Validate SBOM file exists
if [[ ! -f "$OUTPUT_FILE" ]]; then
  echo "ERROR: SBOM file not created: $OUTPUT_FILE" >&2
  exit 1
fi

# Validate SBOM file content
if ! grep -q '"spdxVersion":' "$OUTPUT_FILE"; then
  echo "ERROR: SBOM file does not contain SPDX metadata" >&2
  exit 1
fi

if ! grep -q '"dataLicense":' "$OUTPUT_FILE"; then
  echo "ERROR: SBOM file does not contain data license" >&2
  exit 1
fi

if ! grep -q '"SPDXID":' "$OUTPUT_FILE"; then
  echo "ERROR: SBOM file does not contain SPDX ID" >&2
  exit 1
fi

if ! grep -q '"name":' "$OUTPUT_FILE"; then
  echo "ERROR: SBOM file does not contain name field" >&2
  exit 1
fi

# If we reach here, the test passed
echo "✓ SBOM generation test passed"
