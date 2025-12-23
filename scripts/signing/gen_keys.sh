#!/usr/bin/env bash
set -euo pipefail

# Generate deterministic Ed25519 signing keypair for CostPilot artifact signing
# Keys stored in: packaging/signing/private.key, packaging/signing/public.key

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PRIVATE_KEY="${SCRIPT_DIR}/private.key"
PUBLIC_KEY="${SCRIPT_DIR}/public.key"

# Check if keys already exist
if [ -f "${PRIVATE_KEY}" ] && [ -f "${PUBLIC_KEY}" ]; then
    echo "ERROR: Keys already exist. Remove them first if regeneration is needed."
    echo "  ${PRIVATE_KEY}"
    echo "  ${PUBLIC_KEY}"
    exit 1
fi

# Ensure directory exists
mkdir -p "${SCRIPT_DIR}"

echo "Generating Ed25519 signing keypair..."

# Try openssl first (preferred)
if command -v openssl &> /dev/null; then
    OPENSSL_VERSION=$(openssl version | cut -d' ' -f2 | cut -d'.' -f1-2)

    # Generate Ed25519 private key
    openssl genpkey -algorithm Ed25519 -out "${PRIVATE_KEY}" 2>/dev/null

    # Extract public key
    openssl pkey -in "${PRIVATE_KEY}" -pubout -out "${PUBLIC_KEY}" 2>/dev/null

    # Verify keys were created
    if [ -f "${PRIVATE_KEY}" ] && [ -f "${PUBLIC_KEY}" ]; then
        chmod 600 "${PRIVATE_KEY}"
        chmod 644 "${PUBLIC_KEY}"

        echo "✓ Ed25519 keypair generated successfully"
        echo "  Private key: ${PRIVATE_KEY} (DO NOT COMMIT)"
        echo "  Public key: ${PUBLIC_KEY}"
        echo ""
        echo "Public key fingerprint:"
        openssl pkey -in "${PUBLIC_KEY}" -pubin -text -noout | head -5

        exit 0
    fi
else
    echo "ERROR: openssl not found. Install openssl 1.1.1+ for Ed25519 support."
    exit 1
fi

echo "ERROR: Failed to generate keypair"
exit 1
