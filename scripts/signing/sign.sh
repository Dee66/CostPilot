#!/usr/bin/env bash
set -euo pipefail

# Sign artifact using Ed25519 private key
# Usage: sign.sh <artifact_file> <private_key>
# Output: <artifact_file>.sig

ARTIFACT="${1:-}"
PRIVATE_KEY="${2:-}"

if [ -z "${ARTIFACT}" ] || [ -z "${PRIVATE_KEY}" ]; then
    echo "ERROR: Usage: sign.sh <artifact_file> <private_key>"
    exit 1
fi

if [ ! -f "${ARTIFACT}" ]; then
    echo "ERROR: Artifact not found: ${ARTIFACT}"
    exit 1
fi

if [ ! -f "${PRIVATE_KEY}" ]; then
    echo "ERROR: Private key not found: ${PRIVATE_KEY}"
    exit 1
fi

SIGNATURE_FILE="${ARTIFACT}.sig"

# Sign using openssl
if command -v openssl &> /dev/null; then
    # For Ed25519, we need to create a digest file first
    # Ed25519 signs raw messages, not pre-hashed digests
    
    # Sign the file directly with Ed25519 (raw signing)
    if openssl pkeyutl -sign -inkey "${PRIVATE_KEY}" -rawin -in "${ARTIFACT}" -out "${SIGNATURE_FILE}" 2>/dev/null; then
        # Convert to base64 for readability
        base64 "${SIGNATURE_FILE}" > "${SIGNATURE_FILE}.b64"
        mv "${SIGNATURE_FILE}.b64" "${SIGNATURE_FILE}"
        
        echo "✓ Signed: $(basename "${ARTIFACT}")"
        exit 0
    else
        # Fallback: create digest and sign
        DIGEST_FILE=$(mktemp)
        trap "rm -f ${DIGEST_FILE}" EXIT
        
        openssl dgst -sha256 -binary "${ARTIFACT}" > "${DIGEST_FILE}"
        
        if openssl pkeyutl -sign -inkey "${PRIVATE_KEY}" -in "${DIGEST_FILE}" -out "${SIGNATURE_FILE}" 2>/dev/null; then
            # Convert to base64 for readability
            base64 "${SIGNATURE_FILE}" > "${SIGNATURE_FILE}.b64"
            mv "${SIGNATURE_FILE}.b64" "${SIGNATURE_FILE}"
            
            echo "✓ Signed: $(basename "${ARTIFACT}")"
            exit 0
        fi
    fi
else
    echo "ERROR: openssl not found"
    exit 1
fi

echo "ERROR: Failed to sign artifact"
exit 1
