#!/usr/bin/env bash
set -euo pipefail

# Validation script for Pro Engine build artifacts
# Enforces sanity checks required by CI

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DIST_DIR="${PROJECT_ROOT}/dist"

# Detect target
if [ -z "${TARGET:-}" ]; then
    TARGET="$(bash "${PROJECT_ROOT}/scripts/print_target.sh")"
fi

echo "Validating Pro Engine build for ${TARGET}..."

ENC_FILE="${DIST_DIR}/pro-engine-${TARGET}.wasm.enc"
META_FILE="${DIST_DIR}/pro-engine-${TARGET}.meta.json"
SHA256_FILE="${DIST_DIR}/pro-engine-${TARGET}.sha256"

# Check 1: Encrypted file exists
if [ ! -f "${ENC_FILE}" ]; then
    echo "ERROR: Encrypted WASM not found: ${ENC_FILE}"
    exit 1
fi
echo "✓ Encrypted WASM exists"

# Check 2: No unencrypted WASM in dist/
if find "${DIST_DIR}" -name "*.wasm" -not -name "*.wasm.enc" | grep -q .; then
    echo "ERROR: Unencrypted WASM found in dist/"
    find "${DIST_DIR}" -name "*.wasm" -not -name "*.wasm.enc"
    exit 1
fi
echo "✓ No unencrypted WASM in dist/"

# Check 3: Metadata exists
if [ ! -f "${META_FILE}" ]; then
    echo "ERROR: Metadata file not found: ${META_FILE}"
    exit 1
fi
echo "✓ Metadata file exists"

# Check 4: Metadata has required fields
REQUIRED_FIELDS=("version" "target" "build_timestamp" "wasm_size_bytes" "encryption" "license_required")
for field in "${REQUIRED_FIELDS[@]}"; do
    if ! grep -q "\"${field}\"" "${META_FILE}"; then
        echo "ERROR: Metadata missing required field: ${field}"
        exit 1
    fi
done
echo "✓ Metadata has all required fields"

# Check 5: SHA256 file exists
if [ ! -f "${SHA256_FILE}" ]; then
    echo "ERROR: SHA256 file not found: ${SHA256_FILE}"
    exit 1
fi
echo "✓ SHA256 file exists"

# Check 6: Encrypted file size limit (10MB)
MAX_SIZE=$((10 * 1024 * 1024))
ENC_SIZE=$(stat -c%s "${ENC_FILE}" 2>/dev/null || stat -f%z "${ENC_FILE}" 2>/dev/null)
if [ "${ENC_SIZE}" -gt "${MAX_SIZE}" ]; then
    echo "ERROR: Encrypted WASM size ${ENC_SIZE} exceeds limit of 10MB"
    exit 1
fi
echo "✓ Encrypted WASM size ${ENC_SIZE} bytes (< 10MB)"

# Check 7: Verify JSON is valid
if ! python3 -c "import json; json.load(open('${META_FILE}'))" 2>/dev/null; then
    echo "ERROR: Metadata file is not valid JSON"
    exit 1
fi
echo "✓ Metadata is valid JSON"

# Check 8: Verify encrypted file is valid JSON envelope
if ! python3 -c "import json; d=json.load(open('${ENC_FILE}')); assert 'nonce' in d and 'ciphertext' in d" 2>/dev/null; then
    echo "ERROR: Encrypted file is not valid JSON envelope"
    exit 1
fi
echo "✓ Encrypted file has valid envelope format"

echo ""
echo "All validation checks passed for Pro Engine ${TARGET}"
