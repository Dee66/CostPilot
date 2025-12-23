#!/usr/bin/env bash
set -euo pipefail

# Usage: generate_license_token.sh <license_id> <email> <expires_iso> <out_path>
# Generates signed license JSON with HMAC signature

if [[ $# -ne 4 ]]; then
  echo "ERROR: Usage: generate_license_token.sh <license_id> <email> <expires_iso> <out_path>" >&2
  exit 1
fi

LICENSE_ID="$1"
EMAIL="$2"
EXPIRES="$3"
OUT_PATH="$4"

# Use SOURCE_DATE_EPOCH for reproducibility, or current time
BUILD_TIME="${SOURCE_DATE_EPOCH:-$(date +%s)}"
ISSUED_AT=$(date -u -d "@${BUILD_TIME}" +%Y-%m-%dT%H:%M:%SZ 2>/dev/null || date -u -r "${BUILD_TIME}" +%Y-%m-%dT%H:%M:%SZ)

# Create license JSON skeleton
LICENSE_JSON=$(jq -nc \
  --arg lid "$LICENSE_ID" \
  --arg email "$EMAIL" \
  --arg exp "$EXPIRES" \
  --arg iat "$ISSUED_AT" \
  '{
    license_id: $lid,
    email: $email,
    expires: $exp,
    issued_at: $iat
  }')

# Derive signature
if [[ -n "${SIGNING_SECRET:-}" ]]; then
  # Production: HMAC-SHA256 signature
  KEY=$(echo -n "$SIGNING_SECRET" | sha256sum | cut -d' ' -f1)
  SIGNATURE=$(echo -n "$LICENSE_JSON" | openssl dgst -sha256 -hmac "$KEY" -binary | base64 -w0)
else
  # Dev stub
  SIGNATURE="dev-stub"
fi

# Add signature to JSON
FINAL_JSON=$(echo "$LICENSE_JSON" | jq --arg sig "$SIGNATURE" '. + {signature: $sig}')

# Atomic write
TMP_FILE="${OUT_PATH}.tmp.$$"
echo "$FINAL_JSON" > "$TMP_FILE"
mv "$TMP_FILE" "$OUT_PATH"

echo "LICENSE: ${OUT_PATH}"
