#!/usr/bin/env bash
set -euo pipefail

# Usage: sign_checksum_ed25519.sh <sha256sum.txt> <priv_key.pem> [out_sig]
# Signs checksum file with Ed25519 private key

if [[ $# -lt 2 || $# -gt 3 ]]; then
  echo "ERROR: Usage: sign_checksum_ed25519.sh <sha256sum.txt> <priv_key.pem> [out_sig]" >&2
  exit 1
fi

CHECKSUM_FILE="$1"
PRIV_KEY="$2"
OUT_SIG="${3:-${1}.sig}"

if [[ ! -f "$CHECKSUM_FILE" ]]; then
  echo "ERROR: missing file: $CHECKSUM_FILE" >&2
  exit 2
fi

if [[ ! -f "$PRIV_KEY" ]]; then
  echo "ERROR: missing file: $PRIV_KEY" >&2
  exit 2
fi

# Sign with Ed25519 using pkeyutl -rawin
openssl pkeyutl -sign -inkey "$PRIV_KEY" -rawin -in "$CHECKSUM_FILE" -out "$OUT_SIG" 2>/dev/null

echo "SIGN: ${OUT_SIG}"
