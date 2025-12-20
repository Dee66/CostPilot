#!/usr/bin/env bash
set -euo pipefail

# Create deterministic release bundles (ZIP + TAR.GZ) with normalized timestamps and ordering
# DETERMINISM: --sort=name, --mtime, --owner/group=0, gzip -n, zip -X

DIST_DIR="${1:-}"
VERSION="${2:-}"
PLATFORM="${3:-linux-x64}"

if [ -z "${DIST_DIR}" ] || [ -z "${VERSION}" ]; then
    echo "ERROR: Usage: make_release_bundle.sh <dist_dir> <version> <platform>"
    exit 1
fi

if [ ! -d "${DIST_DIR}" ]; then
    echo "ERROR: dist directory not found: ${DIST_DIR}"
    exit 1
fi

PROJECT_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"

# Normalized timestamp (no fractional seconds)
if [ -n "${SOURCE_DATE_EPOCH}" ]; then
    BUILD_TIME=$(date -u -d "@${SOURCE_DATE_EPOCH}" +"%Y-%m-%dT%H:%M:%SZ")
    BUILD_TIME_TOUCH=$(date -u -d "@${SOURCE_DATE_EPOCH}" +"%Y%m%d%H%M.%S")
else
    BUILD_TIME=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
    BUILD_TIME_TOUCH=$(date -u +"%Y%m%d%H%M.%S")
fi

# Output paths
OUTPUT_TAR="${DIST_DIR}/costpilot-${VERSION}-${PLATFORM}.tar.gz"
OUTPUT_ZIP="${DIST_DIR}/costpilot-${VERSION}-${PLATFORM}.zip"

# Staging directory
STAGE=$(mktemp -d)
trap "rm -rf ${STAGE}" EXIT

# Create bundle directory structure
BUNDLE_DIR="${STAGE}/costpilot-${VERSION}-${PLATFORM}"
mkdir -p "${BUNDLE_DIR}"

# Copy binary if exists
if [ -f "${PROJECT_ROOT}/target/release/costpilot" ]; then
    cp "${PROJECT_ROOT}/target/release/costpilot" "${BUNDLE_DIR}/"
    chmod 755 "${BUNDLE_DIR}/costpilot"
elif [ -f "${PROJECT_ROOT}/target/release/costpilot.exe" ]; then
    cp "${PROJECT_ROOT}/target/release/costpilot.exe" "${BUNDLE_DIR}/"
    chmod 755 "${BUNDLE_DIR}/costpilot.exe"
fi

# Copy documentation
for doc in README.md LICENSE; do
    if [ -f "${PROJECT_ROOT}/${doc}" ]; then
        cp "${PROJECT_ROOT}/${doc}" "${BUNDLE_DIR}/"
        chmod 644 "${BUNDLE_DIR}/${doc}"
    fi
done

# Copy SBOM if exists
if [ -f "${DIST_DIR}/sbom.cyclonedx.json" ]; then
    cp "${DIST_DIR}/sbom.cyclonedx.json" "${BUNDLE_DIR}/"
    chmod 644 "${BUNDLE_DIR}/sbom.cyclonedx.json"
fi

# Copy provenance if exists
if [ -f "${DIST_DIR}/provenance.json" ]; then
    cp "${DIST_DIR}/provenance.json" "${BUNDLE_DIR}/"
    chmod 644 "${BUNDLE_DIR}/provenance.json"
fi

# Copy build metadata
if [ -f "${DIST_DIR}/build.json" ]; then
    cp "${DIST_DIR}/build.json" "${BUNDLE_DIR}/"
    chmod 644 "${BUNDLE_DIR}/build.json"
fi

# Copy examples if present
if [ -d "${PROJECT_ROOT}/examples" ]; then
    mkdir -p "${BUNDLE_DIR}/examples"
    cp -r "${PROJECT_ROOT}/examples"/* "${BUNDLE_DIR}/examples/" 2>/dev/null || true
    find "${BUNDLE_DIR}/examples" -type f -exec chmod 644 {} \;
    find "${BUNDLE_DIR}/examples" -type d -exec chmod 755 {} \;
fi

# Normalize file timestamps
find "${BUNDLE_DIR}" -exec touch -t "${BUILD_TIME_TOUCH}" {} \;

# Create TAR.GZ (deterministic)
cd "${STAGE}"
tar --sort=name --mtime="${BUILD_TIME}" --owner=0 --group=0 --numeric-owner -cf - "costpilot-${VERSION}-${PLATFORM}" | gzip -n -9 > "${OUTPUT_TAR}"

# Create ZIP (deterministic)
zip -q -X -r "${OUTPUT_ZIP}" "costpilot-${VERSION}-${PLATFORM}"

cd "${PROJECT_ROOT}"

# Verify outputs
if [ ! -f "${OUTPUT_TAR}" ] || [ ! -f "${OUTPUT_ZIP}" ]; then
    echo "ERROR: Bundle creation failed"
    exit 1
fi

# Compute checksums
cd "${DIST_DIR}"
CHECKSUM_FILE="sha256sum.txt"

# Determine hash command
HASH_CMD=""
if command -v sha256sum &> /dev/null; then
    HASH_CMD="sha256sum"
elif command -v shasum &> /dev/null; then
    HASH_CMD="shasum -a 256"
else
    echo "ERROR: No hash command available"
    exit 1
fi

# Compute and sort checksums
{
    ${HASH_CMD} "$(basename "${OUTPUT_TAR}")"
    ${HASH_CMD} "$(basename "${OUTPUT_ZIP}")"
} | sort -k2 > "${CHECKSUM_FILE}"

cd "${PROJECT_ROOT}"

echo "BUNDLES: ${OUTPUT_TAR} ${OUTPUT_ZIP}"
echo "CHECKSUMS: ${DIST_DIR}/${CHECKSUM_FILE}"
