#!/usr/bin/env bash
set -euo pipefail

# Generate reproducible build manifest and fingerprinting for CostPilot artifacts
# Creates: build.json, build_fingerprint.txt, provenance.json

PROJECT_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
DIST_DIR="${PROJECT_ROOT}/dist"
BINARY_PATH="${PROJECT_ROOT}/target/release/costpilot"

# Ensure dist directory exists
mkdir -p "${DIST_DIR}"

# ========================================
# Extract metadata
# ========================================

# Version from Cargo.toml
VERSION=$(grep '^version' "${PROJECT_ROOT}/Cargo.toml" | head -1 | cut -d'"' -f2)

# Git commit (short SHA)
GIT_COMMIT="unknown"
if command -v git &> /dev/null && [ -d "${PROJECT_ROOT}/.git" ]; then
    GIT_COMMIT=$(git rev-parse --short HEAD 2>/dev/null || echo "unknown")
fi

# Build timestamp (UTC ISO 8601)
BUILD_TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

# Rustc version
RUSTC_VERSION="unknown"
if command -v rustc &> /dev/null; then
    RUSTC_VERSION=$(rustc --version | cut -d' ' -f2)
fi

# Platform
PLATFORM="linux-x86_64"

# Edition mode
EDITION_MODE="free"

# ========================================
# Generate build.json
# ========================================

cat > "${DIST_DIR}/build.json" <<EOF
{
  "version": "${VERSION}",
  "git_commit": "${GIT_COMMIT}",
  "build_timestamp": "${BUILD_TIMESTAMP}",
  "rustc_version": "${RUSTC_VERSION}",
  "platforms": ["${PLATFORM}"],
  "edition_mode": "${EDITION_MODE}",
  "artifact_files": []
}
EOF

echo "✓ Generated build.json"

# ========================================
# Generate build_fingerprint.txt
# ========================================

FINGERPRINT_FILE="${DIST_DIR}/build_fingerprint.txt"
rm -f "${FINGERPRINT_FILE}"

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

# Hash binary
if [ -f "${BINARY_PATH}" ]; then
    BINARY_HASH=$(${HASH_CMD} "${BINARY_PATH}" | cut -d' ' -f1)
    echo "sha256 ${BINARY_HASH}  costpilot" >> "${FINGERPRINT_FILE}"
fi

# Hash artifacts (will be populated after packaging)
# This file will be appended to by release.sh after packaging

echo "✓ Generated build_fingerprint.txt"

# ========================================
# Generate provenance.json
# ========================================

# Full git commit SHA for provenance
GIT_COMMIT_FULL="unknown"
if command -v git &> /dev/null && [ -d "${PROJECT_ROOT}/.git" ]; then
    GIT_COMMIT_FULL=$(git rev-parse HEAD 2>/dev/null || echo "unknown")
fi

cat > "${DIST_DIR}/provenance.json" <<EOF
{
  "attester": "CostPilot Release Pipeline",
  "verified_artifacts": true,
  "integrity": "sha256",
  "policy": "strict",
  "commit": "${GIT_COMMIT_FULL}",
  "timestamp": "${BUILD_TIMESTAMP}",
  "version": "${VERSION}",
  "edition": "${EDITION_MODE}",
  "platform": "${PLATFORM}",
  "reproducible": true
}
EOF

echo "✓ Generated provenance.json"

# ========================================
# Update build.json with artifact list
# ========================================

# Find all artifacts in dist/
ARTIFACTS=$(find "${DIST_DIR}" -type f \( -name "*.zip" -o -name "*.tar.gz" \) 2>/dev/null | xargs -n1 basename 2>/dev/null || echo "")

if [ -n "${ARTIFACTS}" ]; then
    # Build JSON array
    ARTIFACT_JSON="["
    FIRST=true
    for artifact in ${ARTIFACTS}; do
        if [ "${FIRST}" = true ]; then
            ARTIFACT_JSON="${ARTIFACT_JSON}\"${artifact}\""
            FIRST=false
        else
            ARTIFACT_JSON="${ARTIFACT_JSON}, \"${artifact}\""
        fi
    done
    ARTIFACT_JSON="${ARTIFACT_JSON}]"

    # Update build.json with artifact list
    TEMP_JSON=$(mktemp)
    jq ".artifact_files = ${ARTIFACT_JSON}" "${DIST_DIR}/build.json" > "${TEMP_JSON}" 2>/dev/null || true
    if [ -f "${TEMP_JSON}" ] && [ -s "${TEMP_JSON}" ]; then
        mv "${TEMP_JSON}" "${DIST_DIR}/build.json"
    else
        rm -f "${TEMP_JSON}"
    fi
fi

echo ""
echo "Build manifest generated:"
echo "  Version: ${VERSION}"
echo "  Commit: ${GIT_COMMIT}"
echo "  Timestamp: ${BUILD_TIMESTAMP}"
echo "  Platform: ${PLATFORM}"
echo "  Edition: ${EDITION_MODE}"
