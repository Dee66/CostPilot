#!/usr/bin/env bash
set -euo pipefail

# Build and encrypt Pro Engine WASM bundle
# Produces: pro-engine-$TARGET.wasm.enc + metadata + checksum

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DIST_DIR="${PROJECT_ROOT}/dist"

# Detect target if not set
if [ -z "${TARGET:-}" ]; then
    TARGET="$(bash "${PROJECT_ROOT}/scripts/print_target.sh")"
fi

# Verify encryption key is set
if [ -z "${PRO_ENGINE_ENC_KEY:-}" ]; then
    echo "ERROR: PRO_ENGINE_ENC_KEY environment variable not set (32 bytes hex-encoded)"
    exit 1
fi

# Verify key length (64 hex chars = 32 bytes)
if [ ${#PRO_ENGINE_ENC_KEY} -ne 64 ]; then
    echo "ERROR: PRO_ENGINE_ENC_KEY must be 64 hex characters (32 bytes)"
    exit 1
fi

echo "Building Pro Engine WASM for target: ${TARGET}"

# Create dist directory
mkdir -p "${DIST_DIR}"

# Build WASM with size optimization
cd "${PROJECT_ROOT}"
echo "Compiling WASM with opt-level=z..."

# Build library only for WASM target (avoid bin/lib collision)
RUSTFLAGS="-C opt-level=z" \
cargo build --release --target wasm32-unknown-unknown \
  --lib --features wasm

# Source WASM path
WASM_SOURCE="${PROJECT_ROOT}/target/wasm32-unknown-unknown/release/costpilot.wasm"

# Verify WASM was built
if [ ! -f "${WASM_SOURCE}" ]; then
    echo "ERROR: WASM not found at ${WASM_SOURCE}"
    exit 1
fi

WASM_SIZE=$(stat -c%s "${WASM_SOURCE}" 2>/dev/null || stat -f%z "${WASM_SOURCE}" 2>/dev/null)
echo "WASM built: ${WASM_SIZE} bytes"

# Generate output filenames
ENC_OUTPUT="${DIST_DIR}/pro-engine-${TARGET}.wasm.enc"
META_OUTPUT="${DIST_DIR}/pro-engine-${TARGET}.meta.json"
SHA256_OUTPUT="${DIST_DIR}/pro-engine-${TARGET}.sha256"

# Extract version from Cargo.toml
VERSION=$(grep '^version' "${PROJECT_ROOT}/Cargo.toml" | head -1 | cut -d'"' -f2)
BUILD_TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

# Encrypt WASM using Python (OpenSSL available on all platforms)
echo "Encrypting WASM..."

python3 - <<EOF
import json
import os
import sys
from cryptography.hazmat.primitives.ciphers.aead import AESGCM
from cryptography.hazmat.backends import default_backend

# Read WASM
with open("${WASM_SOURCE}", "rb") as f:
    wasm_data = f.read()

# Parse key
key_hex = os.environ["PRO_ENGINE_ENC_KEY"]
key = bytes.fromhex(key_hex)

# Generate nonce (12 bytes for GCM)
nonce = os.urandom(12)

# Encrypt
aesgcm = AESGCM(key)
ciphertext = aesgcm.encrypt(nonce, wasm_data, None)

# Create envelope
envelope = {
    "nonce": nonce.hex(),
    "ciphertext": ciphertext.hex()
}

# Write encrypted output
with open("${ENC_OUTPUT}", "w") as f:
    json.dump(envelope, f, indent=2)

print(f"Encrypted WASM written to ${ENC_OUTPUT}")
EOF

if [ ! -f "${ENC_OUTPUT}" ]; then
    echo "ERROR: Encryption failed"
    exit 1
fi

ENC_SIZE=$(stat -c%s "${ENC_OUTPUT}" 2>/dev/null || stat -f%z "${ENC_OUTPUT}" 2>/dev/null)
echo "Encrypted WASM: ${ENC_SIZE} bytes"

# Verify encrypted size is under limit
MAX_SIZE=$((10 * 1024 * 1024))  # 10MB
if [ "${ENC_SIZE}" -gt "${MAX_SIZE}" ]; then
    echo "ERROR: Encrypted WASM size ${ENC_SIZE} exceeds limit of 10MB"
    exit 1
fi

# Generate metadata
cat > "${META_OUTPUT}" <<EOF_META
{
  "version": "${VERSION}",
  "target": "${TARGET}",
  "build_timestamp": "${BUILD_TIMESTAMP}",
  "wasm_size_bytes": ${WASM_SIZE},
  "encryption": "AES-256-GCM",
  "license_required": true
}
EOF_META

echo "Metadata written to ${META_OUTPUT}"

# Generate SHA256 checksum
sha256sum "${ENC_OUTPUT}" | cut -d' ' -f1 > "${SHA256_OUTPUT}"
echo "Checksum written to ${SHA256_OUTPUT}"

echo ""
echo "Pro Engine build complete:"
echo "  Encrypted: ${ENC_OUTPUT}"
echo "  Metadata:  ${META_OUTPUT}"
echo "  Checksum:  ${SHA256_OUTPUT}"
