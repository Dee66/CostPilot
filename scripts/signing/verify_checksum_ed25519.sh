#!/usr/bin/env bash
set -euo pipefail

# Usage: verify_checksum_ed25519.sh <sha256sum.txt> <sig> <pub_key.pem>
# Verifies Ed25519 signature against checksum file

if [[ $# -ne 3 ]]; then
  echo "ERROR: Usage: verify_checksum_ed25519.sh <sha256sum.txt> <sig> <pub_key.pem>" >&2
  exit 1
fi

CHECKSUM_FILE="$1"
SIG_FILE="$2"
PUB_KEY="$3"

if [[ ! -f "$CHECKSUM_FILE" ]]; then
  echo "ERROR: missing file: $CHECKSUM_FILE" >&2
  exit 2
fi

if [[ ! -f "$SIG_FILE" ]]; then
  echo "ERROR: missing file: $SIG_FILE" >&2
  exit 2
fi

if [[ ! -f "$PUB_KEY" ]]; then
  echo "ERROR: missing file: $PUB_KEY" >&2
  exit 2
fi

# Verify signature using pkeyutl -rawin
if ! openssl pkeyutl -verify -pubin -inkey "$PUB_KEY" -rawin -in "$CHECKSUM_FILE" -sigfile "$SIG_FILE" 2>/dev/null; then
  echo "VERIFY FAIL" >&2
  exit 3
fi

echo "VERIFY: ${CHECKSUM_FILE} OK"
