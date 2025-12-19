#!/usr/bin/env bash
set -euo pipefail

# Automated artifact verification for CostPilot platform bundles
# Validates: binary integrity, required files, edition enforcement, schemas, determinism, size

PLATFORM="${1:-}"
ARTIFACT_PATH="${2:-}"

if [ -z "${PLATFORM}" ] || [ -z "${ARTIFACT_PATH}" ]; then
    echo "ERROR: Usage: verify_artifact.sh <platform> <artifact_path>"
    echo "  platform: linux-x64, macos-x64, macos-arm64, windows-x64"
    exit 1
fi

if [ ! -f "${ARTIFACT_PATH}" ]; then
    echo "ERROR: Artifact not found: ${ARTIFACT_PATH}"
    exit 1
fi

echo "Verifying artifact: ${ARTIFACT_PATH}"
echo "Platform: ${PLATFORM}"

# Create temp directory for extraction
TEMP_DIR=$(mktemp -d)
trap "rm -rf ${TEMP_DIR}" EXIT

# Extract artifact
echo "→ Extracting artifact..."
if [[ "${ARTIFACT_PATH}" == *.zip ]]; then
    unzip -q "${ARTIFACT_PATH}" -d "${TEMP_DIR}"
elif [[ "${ARTIFACT_PATH}" == *.tar.gz ]]; then
    tar -xzf "${ARTIFACT_PATH}" -C "${TEMP_DIR}"
else
    echo "ERROR: Unsupported artifact format"
    exit 1
fi

# Find extracted directory
BUNDLE_DIR=$(find "${TEMP_DIR}" -mindepth 1 -maxdepth 1 -type d | head -1)
if [ -z "${BUNDLE_DIR}" ]; then
    echo "ERROR: No directory found in artifact"
    exit 1
fi

echo "→ Bundle directory: ${BUNDLE_DIR}"

# Determine binary name
BINARY_NAME="costpilot"
if [[ "${PLATFORM}" == *"windows"* ]]; then
    BINARY_NAME="costpilot.exe"
fi

# ========================================
# VALIDATION: Required Files
# ========================================
echo ""
echo "Checking required files..."

REQUIRED_FILES=(
    "${BINARY_NAME}"
    "LICENSE"
    "README.md"
    "VERSION"
)

for file in "${REQUIRED_FILES[@]}"; do
    if [ ! -f "${BUNDLE_DIR}/${file}" ]; then
        echo "ERROR: Missing required file: ${file}"
        exit 1
    fi
    echo "  ✓ ${file}"
done

# Check for directories
if [ ! -d "${BUNDLE_DIR}/examples" ]; then
    echo "ERROR: Missing examples/ directory"
    exit 1
fi
echo "  ✓ examples/"

# ========================================
# VALIDATION: Binary Exists and Executable
# ========================================
echo ""
echo "Checking binary..."

BINARY_PATH="${BUNDLE_DIR}/${BINARY_NAME}"
if [ ! -f "${BINARY_PATH}" ]; then
    echo "ERROR: Binary not found: ${BINARY_NAME}"
    exit 1
fi

# Check if executable (Unix only)
if [[ "${PLATFORM}" != *"windows"* ]]; then
    if [ ! -x "${BINARY_PATH}" ]; then
        echo "ERROR: Binary is not executable"
        exit 1
    fi
fi

echo "  ✓ Binary exists"

# ========================================
# VALIDATION: No Premium Strings
# ========================================
echo ""
echo "Checking for premium strings..."

FORBIDDEN_STRINGS=(
    "ProEngine"
    "pro_engine"
    "pro-engine"
)

VIOLATIONS=0
for string in "${FORBIDDEN_STRINGS[@]}"; do
    if strings "${BINARY_PATH}" 2>/dev/null | grep -qi "${string}"; then
        echo "  ✗ Found forbidden string: ${string}"
        VIOLATIONS=$((VIOLATIONS + 1))
    fi
done

if [ ${VIOLATIONS} -gt 0 ]; then
    echo "ERROR: Binary contains ${VIOLATIONS} forbidden strings"
    exit 1
fi

echo "  ✓ No premium strings found"

# ========================================
# VALIDATION: No WASM or Encrypted Files
# ========================================
echo ""
echo "Checking for wasm/encrypted files..."

if find "${BUNDLE_DIR}" -name "*.wasm" -o -name "*.enc" | grep -q .; then
    echo "ERROR: Found .wasm or .enc files in bundle"
    find "${BUNDLE_DIR}" -name "*.wasm" -o -name "*.enc"
    exit 1
fi

echo "  ✓ No wasm/encrypted files"

# ========================================
# VALIDATION: Schemas Present
# ========================================
echo ""
echo "Checking schemas..."

# Schemas are optional but if schemas/ exists, validate structure
if [ -d "${BUNDLE_DIR}/schemas" ]; then
    SCHEMA_FILES=(
        "baselines.schema.json"
        "policy.schema.json"
        "slo.schema.json"
        "project.schema.json"
    )
    
    for schema in "${SCHEMA_FILES[@]}"; do
        if [ ! -f "${BUNDLE_DIR}/schemas/${schema}" ]; then
            echo "  ⚠ Schema missing: ${schema} (optional)"
        else
            echo "  ✓ ${schema}"
        fi
    done
else
    echo "  ⚠ schemas/ directory not present (optional)"
fi

# ========================================
# VALIDATION: Edition Enforcement
# ========================================
echo ""
echo "Checking edition..."

# For non-Windows, test binary execution
if [[ "${PLATFORM}" != *"windows"* ]]; then
    VERSION_OUTPUT=$(cd "${BUNDLE_DIR}" && ./${BINARY_NAME} --version 2>&1 || true)
    
    if ! echo "${VERSION_OUTPUT}" | grep -q "(Free)"; then
        echo "ERROR: Binary does not report Free edition"
        echo "Output: ${VERSION_OUTPUT}"
        exit 1
    fi
    
    if echo "${VERSION_OUTPUT}" | grep -qi "(Premium)"; then
        echo "ERROR: Binary reports Premium edition (should be Free)"
        echo "Output: ${VERSION_OUTPUT}"
        exit 1
    fi
    
    echo "  ✓ Edition: Free"
else
    echo "  ⚠ Edition check skipped (Windows binary)"
fi

# ========================================
# VALIDATION: Deterministic Timestamps
# ========================================
echo ""
echo "Checking deterministic timestamps..."

if [ -n "${SOURCE_DATE_EPOCH:-}" ]; then
    EXPECTED_TIME="${SOURCE_DATE_EPOCH}"
    
    # Check all files have correct mtime
    TIMESTAMP_VIOLATIONS=0
    while IFS= read -r -d '' file; do
        FILE_MTIME=$(stat -c %Y "${file}" 2>/dev/null || stat -f %m "${file}" 2>/dev/null || echo "0")
        
        if [ "${FILE_MTIME}" != "${EXPECTED_TIME}" ]; then
            echo "  ✗ Timestamp mismatch: ${file}"
            TIMESTAMP_VIOLATIONS=$((TIMESTAMP_VIOLATIONS + 1))
        fi
    done < <(find "${BUNDLE_DIR}" -type f -print0)
    
    if [ ${TIMESTAMP_VIOLATIONS} -gt 0 ]; then
        echo "ERROR: ${TIMESTAMP_VIOLATIONS} files have incorrect timestamps"
        exit 1
    fi
    
    echo "  ✓ All timestamps match SOURCE_DATE_EPOCH"
else
    echo "  ⚠ SOURCE_DATE_EPOCH not set, skipping timestamp validation"
fi

# ========================================
# VALIDATION: Artifact Size
# ========================================
echo ""
echo "Checking artifact size..."

ARTIFACT_SIZE=$(stat -c%s "${ARTIFACT_PATH}" 2>/dev/null || stat -f%z "${ARTIFACT_PATH}" 2>/dev/null)
SIZE_LIMIT=0

case "${PLATFORM}" in
    linux-x64)
        SIZE_LIMIT=$((15 * 1024 * 1024))  # 15MB
        ;;
    macos-x64|macos-arm64)
        SIZE_LIMIT=$((18 * 1024 * 1024))  # 18MB
        ;;
    windows-x64)
        SIZE_LIMIT=$((20 * 1024 * 1024))  # 20MB
        ;;
    *)
        SIZE_LIMIT=$((20 * 1024 * 1024))  # Default 20MB
        ;;
esac

SIZE_MB=$(echo "scale=2; ${ARTIFACT_SIZE} / 1024 / 1024" | bc)
LIMIT_MB=$(echo "scale=0; ${SIZE_LIMIT} / 1024 / 1024" | bc)

if [ ${ARTIFACT_SIZE} -gt ${SIZE_LIMIT} ]; then
    echo "ERROR: Artifact size ${SIZE_MB}MB exceeds limit of ${LIMIT_MB}MB"
    exit 1
fi

echo "  ✓ Size: ${SIZE_MB}MB (< ${LIMIT_MB}MB)"

# ========================================
# VALIDATION: SHA-256 Signature
# ========================================
echo ""
echo "Checking SHA-256 signature..."

SIGNATURE_FILE="${ARTIFACT_PATH}.sha256"

if [ -f "${SIGNATURE_FILE}" ]; then
    # Determine hash command
    HASH_CMD=""
    if command -v sha256sum &> /dev/null; then
        HASH_CMD="sha256sum"
    elif command -v shasum &> /dev/null; then
        HASH_CMD="shasum -a 256"
    else
        echo "ERROR: Neither sha256sum nor shasum found"
        exit 1
    fi
    
    # Get artifact directory and filename
    ARTIFACT_DIR=$(dirname "${ARTIFACT_PATH}")
    ARTIFACT_FILE=$(basename "${ARTIFACT_PATH}")
    
    # Read stored hash
    STORED_HASH=$(cut -d' ' -f1 "${SIGNATURE_FILE}")
    
    # Calculate current hash
    cd "${ARTIFACT_DIR}"
    CURRENT_HASH=$(${HASH_CMD} "${ARTIFACT_FILE}" | cut -d' ' -f1)
    
    if [ "${STORED_HASH}" != "${CURRENT_HASH}" ]; then
        echo "ERROR: SHA-256 signature mismatch"
        echo "  Expected: ${STORED_HASH}"
        echo "  Actual:   ${CURRENT_HASH}"
        exit 1
    fi
    
    echo "  ✓ SHA-256: ${STORED_HASH}"
else
    echo "  ⚠ No signature file found (expected: ${SIGNATURE_FILE})"
fi

# ========================================
# SUCCESS
# ========================================
echo ""
echo "=========================================="
echo "VERIFIED: ${ARTIFACT_PATH}"
echo "=========================================="
