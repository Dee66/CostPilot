#!/usr/bin/env bash
set -euo pipefail

# Generate SHA-256 signatures for packaged artifacts
# Creates .sha256 files with format: <hash>  <filename>

ARTIFACT_PATH="${1:-}"

if [ -z "${ARTIFACT_PATH}" ]; then
    echo "ERROR: Usage: sign_artifacts.sh <artifact_path>"
    exit 1
fi

if [ ! -f "${ARTIFACT_PATH}" ]; then
    echo "ERROR: Artifact not found: ${ARTIFACT_PATH}"
    exit 1
fi

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
SIGNATURE_FILE="${ARTIFACT_PATH}.sha256"

echo "Signing artifact: ${ARTIFACT_FILE}"

# Generate hash (cd to directory to get clean relative filename)
cd "${ARTIFACT_DIR}"
${HASH_CMD} "${ARTIFACT_FILE}" > "${ARTIFACT_FILE}.sha256"

# Verify signature file was created
if [ ! -f "${ARTIFACT_FILE}.sha256" ]; then
    echo "ERROR: Failed to create signature file"
    exit 1
fi

# Extract just the hash for display
HASH=$(cut -d' ' -f1 "${ARTIFACT_FILE}.sha256")

echo "  SHA-256: ${HASH}"
echo "  Signature: ${ARTIFACT_FILE}.sha256"
echo "✓ Artifact signed successfully"
