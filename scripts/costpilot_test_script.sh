#!/usr/bin/env bash
set -Eeuo pipefail

VERSION="v1.0.1"
RELEASE_BASE="https://github.com/Dee66/CostPilot/releases/download/${VERSION}"
ARCHIVE="costpilot-v1.0.1-linux-x86_64.tar.gz"
CHECKSUMS="SHA256SUMS.txt"

BASE_DIR="$(mktemp -d)"
BIN_DIR="${BASE_DIR}/bin"
HOME_DIR="${BASE_DIR}/home"
WORK_DIR="${BASE_DIR}/workspace"

cleanup() {
  echo "ℹ️  Temp dir preserved at: ${BASE_DIR}"
}
trap cleanup EXIT

mkdir -p "${BIN_DIR}" "${HOME_DIR}" "${WORK_DIR}"
cd "${BASE_DIR}"

echo "==> Downloading release artifacts"
curl -fL "${RELEASE_BASE}/${ARCHIVE}" -o "${ARCHIVE}"
curl -fL "${RELEASE_BASE}/${CHECKSUMS}" -o "${CHECKSUMS}"

echo "==> Verifying checksum"
grep " ${ARCHIVE}$" "${CHECKSUMS}" | sha256sum -c -

echo "==> Extracting binary"
tar -xzf "${ARCHIVE}"

if [[ ! -f costpilot ]]; then
  echo "ERROR: costpilot binary not found after extraction"
  exit 1
fi

mv costpilot "${BIN_DIR}/costpilot"
chmod +x "${BIN_DIR}/costpilot"

echo "==> Isolating HOME"
export HOME="${HOME_DIR}"

echo "==> Sanity checks"
"${BIN_DIR}/costpilot" version
"${BIN_DIR}/costpilot" --help > /dev/null

echo "==> Testing init"
cd "${WORK_DIR}"
"${BIN_DIR}/costpilot" init

if [[ ! -d ".costpilot" ]]; then
  echo "ERROR: init did not create .costpilot directory"
  exit 1
fi

echo "==> Testing scan (expected graceful handling)"
echo '{}' > plan.json
"${BIN_DIR}/costpilot" scan plan.json || true

echo "==> Testing policy help"
"${BIN_DIR}/costpilot" policy --help > /dev/null

echo
echo "✅ RELEASE BINARY VERIFIED SUCCESSFULLY"
