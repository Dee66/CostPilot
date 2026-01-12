#!/usr/bin/env bash
set -Eeuo pipefail

VERSION="v1.0.1"
RELEASE_BASE="https://github.com/Dee66/CostPilot/releases/download/${VERSION}"
ARCHIVE="costpilot-v1.0.1-linux-x86_64.tar.gz"
CHECKSUMS="SHA256SUMS.txt"

LICENSE_FILE="${1:-}"

if [[ -z "$LICENSE_FILE" || ! -f "$LICENSE_FILE" ]]; then
  echo "Usage: $0 /path/to/license.json"
  exit 1
fi

BASE_DIR="$(mktemp -d)"
BIN_DIR="$BASE_DIR/bin"
HOME_DIR="$BASE_DIR/home"
WORK_DIR="$BASE_DIR/work"

trap 'echo "Temp dir preserved at: $BASE_DIR"' EXIT

mkdir -p "$BIN_DIR" "$HOME_DIR" "$WORK_DIR"
cd "$BASE_DIR"

echo "==> Downloading release artifacts"
curl -fsSL "${RELEASE_BASE}/${ARCHIVE}" -o "$ARCHIVE"
curl -fsSL "${RELEASE_BASE}/${CHECKSUMS}" -o "$CHECKSUMS"

echo "==> Verifying checksum"
grep " ${ARCHIVE}$" "$CHECKSUMS" | sha256sum -c -

echo "==> Extracting binary"
tar -xzf "$ARCHIVE"
mv costpilot "$BIN_DIR/costpilot"
chmod +x "$BIN_DIR/costpilot"

echo "==> Installing license into isolated HOME"
export HOME="$HOME_DIR"
mkdir -p "$HOME/.costpilot"
cp "$LICENSE_FILE" "$HOME/.costpilot/license.json"

cd "$WORK_DIR"

echo "==> Verifying licensed mode"
"$BIN_DIR/costpilot" version
"$BIN_DIR/costpilot" scan --help
"$BIN_DIR/costpilot" policy --help

echo
echo "✅ LICENSED RELEASE BINARY VERIFIED SUCCESSFULLY"
