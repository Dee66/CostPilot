#!/usr/bin/env bash
set -euo pipefail

# Verify release bundle checksums
# DETERMINISM: uses sha256sum -c for verification

DIST_DIR="${1:-}"
CHECKSUM_FILE="${2:-}"

if [ -z "${DIST_DIR}" ] || [ -z "${CHECKSUM_FILE}" ]; then
    echo "ERROR: Usage: verify_release_bundle.sh <dist_dir> <checksum_file>"
    exit 1
fi

if [ ! -d "${DIST_DIR}" ]; then
    echo "ERROR: dist directory not found: ${DIST_DIR}"
    exit 1
fi

if [ ! -f "${CHECKSUM_FILE}" ]; then
    echo "ERROR: checksum file not found: ${CHECKSUM_FILE}"
    exit 1
fi

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

# Change to dist directory for verification
cd "${DIST_DIR}"

# Verify checksums
if ${HASH_CMD} -c --status "$(basename "${CHECKSUM_FILE}")" 2>/dev/null; then
    echo "✓ All bundle checksums verified"
    exit 0
else
    echo "ERROR: Checksum verification failed"
    exit 1
fi
