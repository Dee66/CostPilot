#!/usr/bin/env bash
set -euo pipefail

# Smoke test for packaging validation
# Creates test bundle, signs it, and verifies

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
TEST_DIR=$(mktemp -d)
trap "rm -rf '$TEST_DIR'" EXIT

echo "→ Running packaging validation smoke test..."

# Create fake binary for testing
mkdir -p "${PROJECT_ROOT}/target/release"
echo '#!/bin/bash' > "${PROJECT_ROOT}/target/release/costpilot"
echo 'echo "costpilot test"' >> "${PROJECT_ROOT}/target/release/costpilot"
chmod +x "${PROJECT_ROOT}/target/release/costpilot"

# Test bundle creation
export SOURCE_DATE_EPOCH=0
export DEV_FAST=1

bash "${PROJECT_ROOT}/packaging/make_release_bundle.sh" "1.0.0-test" "test-platform" "$TEST_DIR" >/dev/null

if [[ ! -f "${TEST_DIR}/costpilot-1.0.0-test-test-platform.tar.gz" ]]; then
  echo "ERROR: TAR.GZ bundle not created" >&2
  exit 1
fi

if [[ ! -f "${TEST_DIR}/costpilot-1.0.0-test-test-platform.zip" ]]; then
  echo "ERROR: ZIP bundle not created" >&2
  exit 1
fi

if [[ ! -f "${TEST_DIR}/sha256sum.txt" ]]; then
  echo "ERROR: Checksums not created" >&2
  exit 1
fi

# Test signing
unset GPG_PRIVATE_KEY GPG_PASSPHRASE ED25519_PRIV
export SIGNING_SECRET="test-secret-for-validation"
bash "${PROJECT_ROOT}/packaging/signing/sign_checksums.sh" "$TEST_DIR" "${TEST_DIR}/sha256sum.txt" "${TEST_DIR}/sha256sum.sig" >/dev/null

if [[ ! -f "${TEST_DIR}/sha256sum.sig" ]]; then
  echo "ERROR: Signature not created" >&2
  exit 1
fi

# Test verification (with SIGNING_SECRET in env)
export SIGNING_SECRET
bash "${PROJECT_ROOT}/packaging/verify_release_bundle.sh" \
  "${TEST_DIR}/costpilot-1.0.0-test-test-platform.tar.gz" \
  "${TEST_DIR}/sha256sum.txt" \
  "${TEST_DIR}/sha256sum.sig" >/dev/null

bash "${PROJECT_ROOT}/packaging/verify_release_bundle.sh" \
  "${TEST_DIR}/costpilot-1.0.0-test-test-platform.zip" \
  "${TEST_DIR}/sha256sum.txt" \
  "${TEST_DIR}/sha256sum.sig" >/dev/null

echo "✓ Packaging validation smoke test passed"
