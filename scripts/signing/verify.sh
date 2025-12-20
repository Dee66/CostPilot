#!/usr/bin/env bash
set -euo pipefail

# Verify Ed25519 signature for artifact
# Usage: verify.sh <artifact_file> <public_key> <signature_file>
# Exit 0 on success, 1 on failure

ARTIFACT="${1:-}"
PUBLIC_KEY="${2:-}"
SIGNATURE_FILE="${3:-}"

if [ -z "${ARTIFACT}" ] || [ -z "${PUBLIC_KEY}" ] || [ -z "${SIGNATURE_FILE}" ]; then
    echo "ERROR: Usage: verify.sh <artifact_file> <public_key> <signature_file>"
    exit 1
fi

if [ ! -f "${ARTIFACT}" ]; then
    echo "ERROR: Artifact not found: ${ARTIFACT}"
    exit 1
fi

if [ ! -f "${PUBLIC_KEY}" ]; then
    echo "ERROR: Public key not found: ${PUBLIC_KEY}"
    exit 1
fi

if [ ! -f "${SIGNATURE_FILE}" ]; then
    echo "ERROR: Signature not found: ${SIGNATURE_FILE}"
    exit 1
fi

# Verify using openssl
if command -v openssl &> /dev/null; then
    # Decode base64 signature
    TEMP_SIG=$(mktemp)
    trap "rm -f ${TEMP_SIG}" EXIT

    base64 -d "${SIGNATURE_FILE}" > "${TEMP_SIG}" 2>/dev/null

    # Try raw verification first
    if openssl pkeyutl -verify -pubin -inkey "${PUBLIC_KEY}" -rawin -in "${ARTIFACT}" -sigfile "${TEMP_SIG}" 2>/dev/null; then
        echo "✓ Verified: $(basename "${ARTIFACT}")"
        exit 0
    else
        # Fallback: verify digest
        DIGEST_FILE=$(mktemp)
        trap "rm -f ${DIGEST_FILE} ${TEMP_SIG}" EXIT

        openssl dgst -sha256 -binary "${ARTIFACT}" > "${DIGEST_FILE}"

        if openssl pkeyutl -verify -pubin -inkey "${PUBLIC_KEY}" -in "${DIGEST_FILE}" -sigfile "${TEMP_SIG}" 2>/dev/null; then
            echo "✓ Verified: $(basename "${ARTIFACT}")"
            exit 0
        else
            echo "✗ Verification failed: $(basename "${ARTIFACT}")"
            exit 1
        fi
    fi
else
    echo "ERROR: openssl not found"
    exit 1
fi

echo "ERROR: Failed to verify signature"
exit 1
