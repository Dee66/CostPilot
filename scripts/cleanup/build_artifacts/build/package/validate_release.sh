#!/usr/bin/env bash
set -euo pipefail

# Validation script for release archives
# Ensures no premium content or WASM files are included
# Checks archive size limits

ARCHIVE="$1"
MAX_SIZE_MB=15
MAX_SIZE_BYTES=$((MAX_SIZE_MB * 1024 * 1024))

echo "Validating archive: ${ARCHIVE}"

# Check if archive exists
if [ ! -f "${ARCHIVE}" ]; then
    echo "ERROR: Archive not found: ${ARCHIVE}"
    exit 1
fi

# Check archive size
ARCHIVE_SIZE=$(stat -c%s "${ARCHIVE}" 2>/dev/null || stat -f%z "${ARCHIVE}" 2>/dev/null || echo "0")
if [ "${ARCHIVE_SIZE}" -gt "${MAX_SIZE_BYTES}" ]; then
    echo "ERROR: Archive size ${ARCHIVE_SIZE} bytes exceeds limit of ${MAX_SIZE_MB}MB"
    exit 1
fi

echo "✓ Size check passed: ${ARCHIVE_SIZE} bytes (< ${MAX_SIZE_MB}MB)"

# Extract archive to temporary directory for inspection
TMP_DIR=$(mktemp -d)
trap "rm -rf ${TMP_DIR}" EXIT

if [[ "${ARCHIVE}" == *.zip ]]; then
    unzip -q "${ARCHIVE}" -d "${TMP_DIR}"
elif [[ "${ARCHIVE}" == *.tar.gz ]]; then
    tar xzf "${ARCHIVE}" -C "${TMP_DIR}"
else
    echo "ERROR: Unsupported archive format"
    exit 1
fi

# Check for forbidden files
FORBIDDEN_PATTERNS=(
    "premium/"
    "*.wasm"
    "*.wasm.enc"
    "license.json"
    "pro-engine"
    "heuristics/"
)

echo "Scanning for forbidden content..."
VIOLATIONS=0

for pattern in "${FORBIDDEN_PATTERNS[@]}"; do
    if find "${TMP_DIR}" -path "*${pattern}*" -o -name "${pattern}" 2>/dev/null | grep -q .; then
        echo "ERROR: Forbidden pattern found: ${pattern}"
        find "${TMP_DIR}" -path "*${pattern}*" -o -name "${pattern}" 2>/dev/null
        VIOLATIONS=$((VIOLATIONS + 1))
    fi
done

if [ "${VIOLATIONS}" -gt 0 ]; then
    echo "ERROR: Found ${VIOLATIONS} violations"
    exit 1
fi

echo "✓ No forbidden content found"

# Verify expected files are present
EXPECTED_FILES=(
    "README.md"
    "LICENSE"
    "checksums.txt"
)

for file in "${EXPECTED_FILES[@]}"; do
    if ! find "${TMP_DIR}" -name "${file}" | grep -q .; then
        echo "ERROR: Required file missing: ${file}"
        exit 1
    fi
done

echo "✓ All required files present"

# Check that binary exists
if ! find "${TMP_DIR}" -name "costpilot*" -type f -executable 2>/dev/null | grep -q . && \
   ! find "${TMP_DIR}" -name "costpilot.exe" -type f 2>/dev/null | grep -q .; then
    echo "ERROR: Binary not found in archive"
    exit 1
fi

echo "✓ Binary found"

echo ""
echo "Archive validation passed: ${ARCHIVE}"
