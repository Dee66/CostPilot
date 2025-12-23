#!/usr/bin/env bash
set -euo pipefail

# Packaging script for CostPilot
# Creates ZIP and TAR.GZ archives with binary, README, LICENSE, and checksums
# Does NOT include Pro Engine WASM or premium content

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DIST_DIR="${PROJECT_ROOT}/dist"
TMP_DIR="${PROJECT_ROOT}/build/package/tmp"

# Detect target platform if not set
if [ -z "${TARGET:-}" ]; then
    TARGET="$(bash "${PROJECT_ROOT}/scripts/print_target.sh")"
fi

echo "Building CostPilot for target: ${TARGET}"

# Clean and recreate temporary directory
rm -rf "${TMP_DIR}"
mkdir -p "${TMP_DIR}/costpilot-${TARGET}"
mkdir -p "${DIST_DIR}"

# Build in release mode
cd "${PROJECT_ROOT}"
cargo build --release

# Determine binary name and path
BINARY_NAME="costpilot"
BINARY_EXT=""
if [[ "${TARGET}" == *"windows"* ]]; then
    BINARY_EXT=".exe"
fi

BINARY_PATH="${PROJECT_ROOT}/target/release/${BINARY_NAME}${BINARY_EXT}"

# Verify binary exists
if [ ! -f "${BINARY_PATH}" ]; then
    echo "ERROR: Binary not found at ${BINARY_PATH}"
    exit 1
fi

# Copy files to staging directory
STAGE_DIR="${TMP_DIR}/costpilot-${TARGET}"
cp "${BINARY_PATH}" "${STAGE_DIR}/${BINARY_NAME}${BINARY_EXT}"
cp "${PROJECT_ROOT}/README.md" "${STAGE_DIR}/"
cp "${PROJECT_ROOT}/LICENSE" "${STAGE_DIR}/"

# Generate checksums for binary only
cd "${STAGE_DIR}"
sha256sum "${BINARY_NAME}${BINARY_EXT}" > checksums.txt

# Create archives
cd "${TMP_DIR}"
ZIP_FILE="${DIST_DIR}/costpilot-${TARGET}.zip"
TAR_FILE="${DIST_DIR}/costpilot-${TARGET}.tar.gz"

echo "Creating ZIP archive: ${ZIP_FILE}"
zip -r "${ZIP_FILE}" "costpilot-${TARGET}/"

echo "Creating TAR.GZ archive: ${TAR_FILE}"
tar czf "${TAR_FILE}" "costpilot-${TARGET}/"

# Verify archives were created
if [ ! -f "${ZIP_FILE}" ] || [ ! -f "${TAR_FILE}" ]; then
    echo "ERROR: Failed to create archives"
    exit 1
fi

# Show archive sizes
echo ""
echo "Package created successfully:"
ls -lh "${ZIP_FILE}" "${TAR_FILE}"

# Clean up
rm -rf "${TMP_DIR}"

echo ""
echo "Archives ready in ${DIST_DIR}/"
