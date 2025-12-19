#!/usr/bin/env bash
set -euo pipefail

# Usage: generate_keypair.sh <out_dir> <key_name_prefix>
# Generates Ed25519 keypair for release signing

if [[ $# -ne 2 ]]; then
  echo "ERROR: Usage: generate_keypair.sh <out_dir> <key_name_prefix>" >&2
  exit 1
fi

OUT_DIR="$1"
KEY_PREFIX="$2"

if ! command -v openssl >/dev/null 2>&1; then
  echo "ERROR: openssl required" >&2
  exit 2
fi

mkdir -p "$OUT_DIR"

PRIV_KEY="${OUT_DIR}/${KEY_PREFIX}.pem"
PUB_KEY="${OUT_DIR}/${KEY_PREFIX}.pub.pem"

# Generate Ed25519 private key
openssl genpkey -algorithm ED25519 -out "$PRIV_KEY" 2>/dev/null

# Extract public key
openssl pkey -in "$PRIV_KEY" -pubout -out "$PUB_KEY" 2>/dev/null

# Set permissions
chmod 600 "$PRIV_KEY"
chmod 644 "$PUB_KEY"

echo "KEYPAIR: ${PRIV_KEY},${PUB_KEY}"
