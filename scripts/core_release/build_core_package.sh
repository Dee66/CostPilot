#!/usr/bin/env bash
set -euo pipefail

# Build reproducible multi-format release packages for CostPilot CLI (Free Edition)
# Produces: costpilot-$VERSION-linux-x64.zip and costpilot-$VERSION-linux-x64.tar.gz

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DIST_DIR="${PROJECT_ROOT}/dist"
STAGING_BASE="${PROJECT_ROOT}/staging"
STAGING_DIR="${STAGING_BASE}/core"

echo "Building CostPilot Core Release Package..."

# Extract version from Cargo.toml
VERSION=$(grep '^version' "${PROJECT_ROOT}/Cargo.toml" | head -1 | cut -d'"' -f2)
echo "Version: ${VERSION}"

# Set deterministic build environment
export TZ=UTC
export SOURCE_DATE_EPOCH=$(git -C "${PROJECT_ROOT}" log -1 --format=%ct 2>/dev/null || date +%s)
echo "SOURCE_DATE_EPOCH: ${SOURCE_DATE_EPOCH}"

# Build Linux binary
cd "${PROJECT_ROOT}"
echo "Building release binary..."
cargo build --release --bin costpilot

# Strip binary for determinism
echo "Stripping binary..."
strip target/release/costpilot

# Verify binary exists
if [ ! -f target/release/costpilot ]; then
    echo "ERROR: Binary not found at target/release/costpilot"
    exit 1
fi

# Create clean staging directory
rm -rf "${STAGING_DIR}"
mkdir -p "${STAGING_DIR}"

# Copy required files
echo "Staging files..."
cp target/release/costpilot "${STAGING_DIR}/"
cp LICENSE "${STAGING_DIR}/"
cp README.md "${STAGING_DIR}/"

# Create VERSION file
echo "${VERSION}" > "${STAGING_DIR}/VERSION"

# Copy schemas if they exist
if [ -d "${PROJECT_ROOT}/schemas" ]; then
    cp -r "${PROJECT_ROOT}/schemas" "${STAGING_DIR}/"
fi

# Copy examples if they exist
if [ -d "${PROJECT_ROOT}/examples" ]; then
    cp -r "${PROJECT_ROOT}/examples" "${STAGING_DIR}/"
fi

# Remove premium/pro-engine artifacts
echo "Removing premium content..."
find "${STAGING_DIR}" -type d -name "premium" -exec rm -rf {} + 2>/dev/null || true
find "${STAGING_DIR}" -type f -name "*pro_engine*" -delete 2>/dev/null || true
find "${STAGING_DIR}" -type f -name "*.wasm" -delete 2>/dev/null || true
find "${STAGING_DIR}" -type f -name "*.wasm.enc" -delete 2>/dev/null || true
find "${STAGING_DIR}" -type f -name "pro-engine*" -delete 2>/dev/null || true

# Remove test and dev content
rm -rf "${STAGING_DIR}/tests" 2>/dev/null || true
find "${STAGING_DIR}" -type f -name "make_video.sh" -delete 2>/dev/null || true

# Create dist directory
mkdir -p "${DIST_DIR}"

# Archive names
ZIP_NAME="costpilot-${VERSION}-linux-x64.zip"
TAR_NAME="costpilot-${VERSION}-linux-x64.tar.gz"
ZIP_PATH="${DIST_DIR}/${ZIP_NAME}"
TAR_PATH="${DIST_DIR}/${TAR_NAME}"

# Produce ZIP with deterministic settings
echo "Creating ZIP archive..."
cd "${STAGING_BASE}"
zip -X -r "${ZIP_PATH}" core/

# Produce TAR.GZ with deterministic settings
echo "Creating TAR.GZ archive..."
tar --mtime="@${SOURCE_DATE_EPOCH}" --owner=0 --group=0 --numeric-owner \
    -czf "${TAR_PATH}" core/

# Verify archives
if [ ! -f "${ZIP_PATH}" ] || [ ! -f "${TAR_PATH}" ]; then
    echo "ERROR: Failed to create archives"
    exit 1
fi

# Show results
echo ""
echo "Core release packages created:"
ls -lh "${ZIP_PATH}"
ls -lh "${TAR_PATH}"

# Cleanup staging
rm -rf "${STAGING_DIR}"

echo ""
echo "✓ Build complete: ${VERSION}"

echo ""
echo "Build complete. Artifacts in ${DIST_DIR}/"
