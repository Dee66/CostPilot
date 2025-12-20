#!/usr/bin/env bash
set -euo pipefail

# Usage: verify_release_policy.sh
# Scans dist/ for unencrypted or unsigned pro bundles

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DIST_DIR="${PROJECT_ROOT}/dist"

if [[ ! -d "$DIST_DIR" ]]; then
  echo "RELEASE_BUNDLE_CHECK: OK (no dist/)"
  exit 0
fi

# Check for pro-engine related files
PRO_BUNDLES=$(find "$DIST_DIR" -type f \( -name "pro_engine.*" -o -name "pro-engine.*" -o -name "*-pro.wasm" \) 2>/dev/null || true)

if [[ -z "$PRO_BUNDLES" ]]; then
  echo "RELEASE_BUNDLE_CHECK: OK"
  exit 0
fi

# Check each pro bundle
while IFS= read -r bundle; do
  [[ -z "$bundle" ]] && continue

  # Check for signature file
  if [[ ! -f "${bundle}.sig" ]]; then
    echo "UNSIGNED_BUNDLE: ${bundle}" >&2
    exit 4
  fi

  # Check if file is suspiciously large and unencrypted
  SIZE=$(stat -c%s "$bundle" 2>/dev/null || stat -f%z "$bundle")
  if [[ $SIZE -gt 5242880 ]]; then
    # Check for unencrypted marker (WASM magic bytes at start)
    if head -c 4 "$bundle" | grep -q "$(printf '\x00\x61\x73\x6d')" 2>/dev/null; then
      echo "UNENCRYPTED_PRO_BUNDLE: ${bundle}" >&2
      exit 4
    fi
  fi
done <<< "$PRO_BUNDLES"

echo "RELEASE_BUNDLE_CHECK: OK"
