#!/usr/bin/env bash
set -euo pipefail

# Usage: verify_signature.sh <sha256sum.txt> <sig> [public_key_file]
# Verifies GPG, Ed25519, or HMAC signature

if [[ $# -lt 2 || $# -gt 3 ]]; then
  echo "ERROR: Usage: verify_signature.sh <sha256sum.txt> <sig> [public_key_file]" >&2
  exit 1
fi

CHECKSUM_FILE="$1"
SIG_FILE="$2"
PUBLIC_KEY="${3:-}"

if [[ ! -f "$CHECKSUM_FILE" ]]; then
  echo "ERROR: Checksum file not found: $CHECKSUM_FILE" >&2
  exit 1
fi

if [[ ! -f "$SIG_FILE" ]]; then
  echo "ERROR: Signature file not found: $SIG_FILE" >&2
  exit 1
fi

# Detect signature type
if [[ -n "$PUBLIC_KEY" ]]; then
  if [[ "$SIG_FILE" == *.asc ]] || grep -q "BEGIN PGP SIGNATURE" "$SIG_FILE" 2>/dev/null; then
    # GPG signature
    GNUPGHOME=$(mktemp -d)
    export GNUPGHOME
    trap "rm -rf '$GNUPGHOME'" EXIT

    gpg --batch --import "$PUBLIC_KEY" 2>/dev/null

    if gpg --batch --verify "$SIG_FILE" "$CHECKSUM_FILE" 2>/dev/null; then
      echo "VERIFIED: true"
    else
      echo "ERROR: GPG signature verification failed" >&2
      exit 1
    fi

  else
    # Ed25519 signature (base64)
    SIG_BINARY=$(mktemp)
    trap "rm -f '$SIG_BINARY'" EXIT

    base64 -d "$SIG_FILE" > "$SIG_BINARY"

    # Try raw verification first
    if openssl pkeyutl -verify -pubin -inkey "$PUBLIC_KEY" -rawin -in "$CHECKSUM_FILE" -sigfile "$SIG_BINARY" 2>/dev/null; then
      echo "VERIFIED: true"
    else
      # Try digest verification
      if openssl dgst -sha256 -verify "$PUBLIC_KEY" -signature "$SIG_BINARY" "$CHECKSUM_FILE" 2>/dev/null; then
        echo "VERIFIED: true"
      else
        echo "ERROR: Ed25519 signature verification failed" >&2
        exit 1
      fi
    fi
  fi

else
  # HMAC verification (requires SIGNING_SECRET)
  if [[ -z "${SIGNING_SECRET:-}" ]]; then
    echo "ERROR: SIGNING_SECRET required for HMAC verification" >&2
    exit 1
  fi

  KEY=$(echo -n "$SIGNING_SECRET" | sha256sum | cut -d' ' -f1)
  EXPECTED_SIG=$(openssl dgst -sha256 -hmac "$KEY" -binary "$CHECKSUM_FILE" | base64 -w0)
  ACTUAL_SIG=$(cat "$SIG_FILE")

  if [[ "$EXPECTED_SIG" == "$ACTUAL_SIG" ]]; then
    echo "VERIFIED: true"
  else
    echo "ERROR: HMAC signature verification failed" >&2
    exit 1
  fi
fi
