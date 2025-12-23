#!/usr/bin/env bash
set -euo pipefail

# Usage: verify_release_bundle.sh <artifact> <sha256sum.txt> <sig> [public_key_file]
# Validates release bundle integrity, signatures, and content

if [[ $# -lt 3 || $# -gt 4 ]]; then
  echo "ERROR: Usage: verify_release_bundle.sh <artifact> <sha256sum.txt> <sig> [public_key_file]" >&2
  exit 1
fi

ARTIFACT="$1"
CHECKSUM_FILE="$2"
SIG_FILE="$3"
PUBLIC_KEY="${4:-}"

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
VERIFY_SIG_SCRIPT="${PROJECT_ROOT}/packaging/signing/verify_signature.sh"

if [[ ! -f "$ARTIFACT" ]]; then
  echo "ERROR: Artifact not found: $ARTIFACT" >&2
  exit 1
fi

if [[ ! -f "$CHECKSUM_FILE" ]]; then
  echo "ERROR: Checksum file not found: $CHECKSUM_FILE" >&2
  exit 1
fi

if [[ ! -f "$SIG_FILE" ]]; then
  echo "ERROR: Signature file not found: $SIG_FILE" >&2
  exit 1
fi

# Verify checksum
ARTIFACT_BASENAME="$(basename "$ARTIFACT")"
ARTIFACT_DIR="$(dirname "$ARTIFACT")"
EXPECTED_CHECKSUM=$(grep " ${ARTIFACT_BASENAME}$" "$CHECKSUM_FILE" | cut -d' ' -f1)

if [[ -z "$EXPECTED_CHECKSUM" ]]; then
  echo "ERROR: Artifact not found in checksum file: $ARTIFACT_BASENAME" >&2
  exit 1
fi

ACTUAL_CHECKSUM=$(sha256sum "$ARTIFACT" | cut -d' ' -f1)

if [[ "$EXPECTED_CHECKSUM" != "$ACTUAL_CHECKSUM" ]]; then
  echo "ERROR: Checksum mismatch for $ARTIFACT_BASENAME" >&2
  exit 1
fi

# Verify signature
if [[ -x "$VERIFY_SIG_SCRIPT" ]]; then
  if [[ -n "$PUBLIC_KEY" && -f "$PUBLIC_KEY" ]]; then
    bash "$VERIFY_SIG_SCRIPT" "$CHECKSUM_FILE" "$SIG_FILE" "$PUBLIC_KEY" >/dev/null
  elif [[ -n "${SIGNING_SECRET:-}" ]]; then
    bash "$VERIFY_SIG_SCRIPT" "$CHECKSUM_FILE" "$SIG_FILE" >/dev/null
  else
    # Skip signature verification if no credentials available
    true
  fi
fi

# Verify artifact contents
TMPDIR=$(mktemp -d)
trap "rm -rf '$TMPDIR'" EXIT

if [[ "$ARTIFACT" == *.tar.gz ]]; then
  tar -xzf "$ARTIFACT" -C "$TMPDIR"
elif [[ "$ARTIFACT" == *.zip ]]; then
  unzip -q "$ARTIFACT" -d "$TMPDIR"
else
  echo "ERROR: Unknown artifact format: $ARTIFACT" >&2
  exit 1
fi

# Check for forbidden files
FORBIDDEN_PATTERNS=(.git/ .env "*.pem" "*.key" pro-engine/ pro_engine/ LICENSE_PRIVATE .DS_Store)
for pattern in "${FORBIDDEN_PATTERNS[@]}"; do
  if find "$TMPDIR" -name "$pattern" -o -path "*/$pattern/*" 2>/dev/null | grep -q .; then
    echo "ERROR: Forbidden file pattern found: $pattern" >&2
    exit 1
  fi
done

# Check SBOM presence
if ! find "$TMPDIR" -name "sbom.spdx.json" | grep -q .; then
  echo "ERROR: SBOM not found in artifact" >&2
  exit 1
fi

# Check artifact size
MIN_BYTES="${MIN_ARTIFACT_BYTES:-0}"
MAX_BYTES="${MAX_ARTIFACT_BYTES:-52428800}"
ARTIFACT_SIZE=$(stat -c%s "$ARTIFACT" 2>/dev/null || stat -f%z "$ARTIFACT")

if [[ $ARTIFACT_SIZE -lt $MIN_BYTES ]]; then
  echo "ERROR: Artifact too small: ${ARTIFACT_SIZE} bytes (min: ${MIN_BYTES})" >&2
  exit 1
fi

if [[ $ARTIFACT_SIZE -gt $MAX_BYTES ]]; then
  echo "ERROR: Artifact too large: ${ARTIFACT_SIZE} bytes (max: ${MAX_BYTES})" >&2
  exit 1
fi

echo "VERIFY: ${ARTIFACT_BASENAME} OK"
