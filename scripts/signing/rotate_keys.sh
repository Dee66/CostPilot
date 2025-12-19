#!/usr/bin/env bash
set -euo pipefail

# Usage: rotate_keys.sh <keys_dir> <new_prefix> <rotation_out>
# Creates new keypair and produces rotation metadata

if [[ $# -ne 3 ]]; then
  echo "ERROR: Usage: rotate_keys.sh <keys_dir> <new_prefix> <rotation_out>" >&2
  exit 1
fi

KEYS_DIR="$1"
NEW_PREFIX="$2"
ROTATION_OUT="$3"

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
GENERATE_SCRIPT="${PROJECT_ROOT}/packaging/signing/generate_keypair.sh"

if [[ ! -x "$GENERATE_SCRIPT" ]]; then
  echo "ERROR: generate_keypair.sh not found or not executable" >&2
  exit 2
fi

# Find previous public key if exists
PREV_PUB_SHA256="none"
if compgen -G "${KEYS_DIR}/*.pub.pem" > /dev/null 2>&1; then
  PREV_PUB=$(ls -t "${KEYS_DIR}"/*.pub.pem 2>/dev/null | head -1)
  if [[ -f "$PREV_PUB" ]]; then
    PREV_PUB_SHA256=$(sha256sum "$PREV_PUB" | cut -d' ' -f1)
  fi
fi

# Generate new keypair
bash "$GENERATE_SCRIPT" "$KEYS_DIR" "$NEW_PREFIX" >/dev/null

NEW_PUB_KEY="${KEYS_DIR}/${NEW_PREFIX}.pub.pem"
ROTATED_AT=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

# Create rotation metadata
TMP_FILE="${ROTATION_OUT}.tmp.$$"
jq -nc \
  --arg rotated "$ROTATED_AT" \
  --arg newpub "$NEW_PUB_KEY" \
  --arg prevsha "$PREV_PUB_SHA256" \
  '{
    rotated_at: $rotated,
    new_public_key: $newpub,
    prev_public_key_sha256: $prevsha
  }' > "$TMP_FILE"

mv "$TMP_FILE" "$ROTATION_OUT"
echo "ROTATE: ${ROTATION_OUT}"
