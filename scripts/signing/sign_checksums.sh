#!/usr/bin/env bash
set -euo pipefail

# Usage: sign_checksums.sh <dist_dir> <sha256sum.txt> <out_sig>
# Signs checksums with GPG, Ed25519, or HMAC fallback

if [[ $# -ne 3 ]]; then
  echo "ERROR: Usage: sign_checksums.sh <dist_dir> <sha256sum.txt> <out_sig>" >&2
  exit 1
fi

DIST_DIR="$1"
CHECKSUM_FILE="$2"
OUT_SIG="$3"

if [[ ! -f "$CHECKSUM_FILE" ]]; then
  echo "ERROR: Checksum file not found: $CHECKSUM_FILE" >&2
  exit 1
fi

# Strategy: GPG > Ed25519 > HMAC fallback

if [[ -n "${GPG_PRIVATE_KEY:-}" && -n "${GPG_PASSPHRASE:-}" ]]; then
  # GPG signing with ephemeral home
  GNUPGHOME=$(mktemp -d)
  export GNUPGHOME
  trap "rm -rf '$GNUPGHOME'" EXIT
  
  # Import key
  echo "$GPG_PRIVATE_KEY" | gpg --batch --import 2>/dev/null
  
  # Sign with passphrase
  echo "$GPG_PASSPHRASE" | gpg --batch --yes --pinentry-mode loopback \
    --passphrase-fd 0 --output "$OUT_SIG" --detach-sign --armor "$CHECKSUM_FILE" 2>/dev/null
  
  echo "SIGNED: ${OUT_SIG}"

elif [[ -n "${ED25519_PRIV:-}" ]]; then
  # Ed25519 signing via openssl
  ED25519_KEY_FILE=$(mktemp)
  trap "rm -f '$ED25519_KEY_FILE'" EXIT
  
  echo "$ED25519_PRIV" > "$ED25519_KEY_FILE"
  chmod 600 "$ED25519_KEY_FILE"
  
  # Sign with Ed25519
  if openssl pkeyutl -sign -inkey "$ED25519_KEY_FILE" -rawin -in "$CHECKSUM_FILE" 2>/dev/null | base64 -w0 > "$OUT_SIG"; then
    echo "SIGNED: ${OUT_SIG}"
  else
    # Fallback to digest signing
    openssl dgst -sha256 -sign "$ED25519_KEY_FILE" -out - "$CHECKSUM_FILE" | base64 -w0 > "$OUT_SIG"
    echo "SIGNED: ${OUT_SIG}"
  fi

elif [[ -n "${SIGNING_SECRET:-}" ]]; then
  # HMAC-SHA256 fallback
  KEY=$(echo -n "$SIGNING_SECRET" | sha256sum | cut -d' ' -f1)
  openssl dgst -sha256 -hmac "$KEY" -binary "$CHECKSUM_FILE" | base64 -w0 > "$OUT_SIG"
  echo "SIGNED: ${OUT_SIG}"

else
  echo "ERROR: No signing credentials available (GPG_PRIVATE_KEY, ED25519_PRIV, or SIGNING_SECRET)" >&2
  exit 1
fi
