#!/usr/bin/env bash
set -euo pipefail

# Smoke test for Ed25519 sign/verify roundtrip

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
TEST_DIR=$(mktemp -d)
trap "rm -rf '$TEST_DIR'" EXIT

echo "→ Testing Ed25519 sign/verify..."

# Generate test keypair
bash "${PROJECT_ROOT}/packaging/signing/generate_keypair.sh" "$TEST_DIR" test_key >/dev/null

if [[ ! -f "${TEST_DIR}/test_key.pem" ]]; then
  echo "ERROR: Keypair generation failed" >&2
  exit 1
fi

# Create sample checksum file
echo "abc123  testfile.tar.gz" > "${TEST_DIR}/sha256sum.txt"

# Sign checksums
bash "${PROJECT_ROOT}/packaging/signing/sign_checksum_ed25519.sh" \
  "${TEST_DIR}/sha256sum.txt" \
  "${TEST_DIR}/test_key.pem" \
  "${TEST_DIR}/sha256sum.txt.sig" >/dev/null

if [[ ! -f "${TEST_DIR}/sha256sum.txt.sig" ]]; then
  echo "ERROR: Signing failed" >&2
  exit 1
fi

# Verify signature
bash "${PROJECT_ROOT}/packaging/signing/verify_checksum_ed25519.sh" \
  "${TEST_DIR}/sha256sum.txt" \
  "${TEST_DIR}/sha256sum.txt.sig" \
  "${TEST_DIR}/test_key.pub.pem" >/dev/null

echo "✓ Ed25519 sign/verify test passed"
