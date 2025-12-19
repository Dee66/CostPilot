#!/usr/bin/env bash
set -euo pipefail

# Usage: ci_release_steps.sh <version> <platform>
# Orchestrates release build, packaging, signing, and verification for CI

if [[ $# -ne 2 ]]; then
  echo "ERROR: Usage: ci_release_steps.sh <version> <platform>" >&2
  exit 1
fi

VERSION="$1"
PLATFORM="$2"

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DIST_DIR="${PROJECT_ROOT}/dist"

mkdir -p "$DIST_DIR"

# Step 1: Build release binary
echo "→ Building release binary..."
cargo build --release --quiet

# Step 2: Create release bundles
echo "→ Creating release bundles..."
bash "${PROJECT_ROOT}/packaging/make_release_bundle.sh" "$VERSION" "$PLATFORM" "$DIST_DIR"

# Step 3: Sign checksums
echo "→ Signing checksums..."
bash "${PROJECT_ROOT}/packaging/signing/sign_checksums.sh" "$DIST_DIR" "${DIST_DIR}/sha256sum.txt" "${DIST_DIR}/sha256sum.sig"

# Step 4: Verify bundles
echo "→ Verifying bundles..."
PUBLIC_KEY="${PROJECT_ROOT}/packaging/signing/public.key"
SIG_AVAILABLE=false

# Check if we can verify signatures
if [[ -f "$PUBLIC_KEY" ]] && [[ -n "${ED25519_PRIV:-}" || -n "${GPG_PRIVATE_KEY:-}" ]]; then
  SIG_AVAILABLE=true
elif [[ -n "${SIGNING_SECRET:-}" ]]; then
  SIG_AVAILABLE=true
  PUBLIC_KEY=""
fi

for bundle in "${DIST_DIR}/costpilot-${VERSION}-${PLATFORM}".{tar.gz,zip}; do
  if [[ -f "$bundle" ]]; then
    if [[ "$SIG_AVAILABLE" == "true" ]]; then
      bash "${PROJECT_ROOT}/packaging/verify_release_bundle.sh" "$bundle" "${DIST_DIR}/sha256sum.txt" "${DIST_DIR}/sha256sum.sig" "$PUBLIC_KEY"
    else
      echo "→ Skipping signature verification (no credentials)"
    fi
  fi
done

echo "→ Release artifacts ready in: ${DIST_DIR}"
ls -lh "$DIST_DIR"
