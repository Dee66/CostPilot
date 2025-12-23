#!/usr/bin/env bash
set -euo pipefail

# Generate Ed25519-signed license for CostPilot Pro
# Usage: generate_ed25519_license.sh <email> <license_key> <expires_iso> <private_key_path> [output_path]

if [[ $# -lt 4 ]]; then
  echo "ERROR: Usage: generate_ed25519_license.sh <email> <license_key> <expires_iso> <private_key_path> [output_path]" >&2
  exit 1
fi

EMAIL="$1"
LICENSE_KEY="$2"
EXPIRES="$3"
PRIVATE_KEY_PATH="$4"
OUTPUT_PATH="${5:-license.json}"

# Validate inputs
if [[ ! -f "$PRIVATE_KEY_PATH" ]]; then
  echo "ERROR: Private key file not found: $PRIVATE_KEY_PATH" >&2
  exit 1
fi

# Validate date format (ISO 8601)
if ! date -d "$EXPIRES" >/dev/null 2>&1; then
  echo "ERROR: Invalid expires date format. Use ISO 8601 (e.g., 2025-12-31T23:59:59Z)" >&2
  exit 1
fi

# Generate canonical message
CANONICAL_MESSAGE="${EMAIL}|${LICENSE_KEY}|${EXPIRES}"

# Generate timestamp
ISSUED_AT=$(date -u +%Y-%m-%dT%H:%M:%SZ)

# Sign with Ed25519 using openssl (if available) or fallback to external tool
if command -v openssl >/dev/null 2>&1 && openssl ecparam -list_curves 2>/dev/null | grep -q Ed25519; then
  # Use openssl if Ed25519 is supported
  SIGNATURE=$(echo -n "$CANONICAL_MESSAGE" | openssl dgst -sha256 -sign "$PRIVATE_KEY_PATH" -binary | xxd -p -c 256 | tr -d '\n')
else
  # Fallback: require external Ed25519 signing tool
  if ! command -v ed25519-sign >/dev/null 2>&1; then
    echo "ERROR: Neither openssl with Ed25519 support nor ed25519-sign tool found" >&2
    echo "Install ed25519-sign or use openssl with Ed25519 support" >&2
    exit 1
  fi
  SIGNATURE=$(echo -n "$CANONICAL_MESSAGE" | ed25519-sign "$PRIVATE_KEY_PATH" | xxd -p -c 256 | tr -d '\n')
fi

# Create license JSON
LICENSE_JSON=$(jq -nc \
  --arg email "$EMAIL" \
  --arg license_key "$LICENSE_KEY" \
  --arg expires "$EXPIRES" \
  --arg issued_at "$ISSUED_AT" \
  --arg signature "$SIGNATURE" \
  '{
    email: $email,
    license_key: $license_key,
    expires: $expires,
    issued_at: $issued_at,
    signature: $signature,
    version: "1.0"
  }')

# Atomic write
TMP_FILE="${OUTPUT_PATH}.tmp.$$"
echo "$LICENSE_JSON" > "$TMP_FILE"
mv "$TMP_FILE" "$OUTPUT_PATH"

echo "License generated: $OUTPUT_PATH"
echo "Key fingerprint: $(openssl dgst -sha256 "$PRIVATE_KEY_PATH" 2>/dev/null | cut -d' ' -f2 || echo 'N/A')" >&2
