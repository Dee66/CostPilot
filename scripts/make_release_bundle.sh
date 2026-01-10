#!/usr/bin/env bash
set -euo pipefail

# Usage: make_release_bundle.sh [<version> [<platform> [<out_dir>]]]
# Creates deterministic ZIP and TAR.GZ release bundles with SBOM
# If arguments not provided, uses environment variables or defaults

if [[ $# -eq 0 ]]; then
  VERSION="${COSTPILOT_VERSION:-$(git describe --tags --abbrev=0 2>/dev/null || echo "dev")}"
  PLATFORM="${TARGET:-$(uname -m)}"
  OUT_DIR="${OUT_DIR:-dist}"
elif [[ $# -eq 3 ]]; then
  VERSION="$1"
  PLATFORM="$2"
  OUT_DIR="$3"
else
  echo "ERROR: Usage: make_release_bundle.sh [<version> [<platform> [<out_dir>]]]" >&2
  echo "       Or set COSTPILOT_VERSION, TARGET, and OUT_DIR environment variables" >&2
  exit 1
fi

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

# Detect platform for binary naming
PLATFORM_TYPE=$(uname -s)
BINARY_NAME="costpilot"
if [[ "$PLATFORM_TYPE" == "MINGW"* ]] || [[ "$PLATFORM_TYPE" == "MSYS"* ]] || [[ "$PLATFORM_TYPE" == "CYGWIN"* ]]; then
  BINARY_NAME="costpilot.exe"
fi

BINARY_PATH="${PROJECT_ROOT}/target/release/${BINARY_NAME}"

if [[ ! -f "$BINARY_PATH" ]]; then
  echo "ERROR: Binary not found: $BINARY_PATH" >&2
  exit 1
fi

mkdir -p "$OUT_DIR"

# Create temporary staging directory
TMPDIR=$(mktemp -d)
trap "rm -rf '$TMPDIR'" EXIT

BUNDLE_NAME="costpilot-${VERSION}-${PLATFORM}"
STAGE_DIR="${TMPDIR}/${BUNDLE_NAME}"
mkdir -p "$STAGE_DIR/bin"

# Copy binary
cp "$BINARY_PATH" "$STAGE_DIR/bin/$BINARY_NAME"
chmod 755 "$STAGE_DIR/bin/$BINARY_NAME"

# Copy documentation
[[ -f "${PROJECT_ROOT}/README.md" ]] && cp "${PROJECT_ROOT}/README.md" "$STAGE_DIR/"
[[ -f "${PROJECT_ROOT}/LICENSE" ]] && cp "${PROJECT_ROOT}/LICENSE" "$STAGE_DIR/"

# Generate SBOM
SBOM_SCRIPT="${PROJECT_ROOT}/packaging/sbom/generate_sbom.sh"
if [[ -x "$SBOM_SCRIPT" ]]; then
  bash "$SBOM_SCRIPT" "$STAGE_DIR" "$STAGE_DIR/sbom.spdx.json" >/dev/null
else
  echo '{"spdxVersion":"SPDX-2.3","dataLicense":"CC0-1.0","SPDXID":"SPDXRef-DOCUMENT","name":"CostPilot"}' > "$STAGE_DIR/sbom.spdx.json"
fi

# Deterministic timestamp
if [[ -n "${DEV_FAST:-}" ]]; then
  MTIME_ARG=""
else
  BUILD_TIME="${SOURCE_DATE_EPOCH:-0}"
  MTIME_ARG="--mtime=@${BUILD_TIME}"
fi

# Create deterministic TAR.GZ
TAR_OUTPUT="${OUT_DIR}/${BUNDLE_NAME}.tar.gz"
if [[ -n "${DEV_FAST:-}" ]]; then
  tar -czf "$TAR_OUTPUT" -C "$TMPDIR" "$BUNDLE_NAME"
else
  # Check if tar supports --sort (GNU tar), otherwise use basic options
  if tar --sort=name --help >/dev/null 2>&1; then
    tar --sort=name $MTIME_ARG --owner=0 --group=0 --numeric-owner \
      -czf "$TAR_OUTPUT" -C "$TMPDIR" "$BUNDLE_NAME"
  else
    # macOS tar doesn't support --sort, use basic deterministic options
    tar $MTIME_ARG --owner=0 --group=0 --numeric-owner \
      -czf "$TAR_OUTPUT" -C "$TMPDIR" "$BUNDLE_NAME"
  fi
fi

# Create deterministic ZIP
ZIP_OUTPUT="${OUT_DIR}/${BUNDLE_NAME}.zip"
ZIP_OUTPUT_ABS="$(cd "$OUT_DIR" && pwd)/${BUNDLE_NAME}.zip"

if [[ -n "${DEV_FAST:-}" ]]; then
  (cd "$TMPDIR" && zip -q -r "$ZIP_OUTPUT_ABS" "$BUNDLE_NAME")
else
  # Use Python for deterministic ZIP if available
  if command -v python3 >/dev/null 2>&1; then
    python3 - "$TMPDIR" "$BUNDLE_NAME" "$ZIP_OUTPUT_ABS" <<'EOF'
import sys, os, zipfile
from pathlib import Path

tmpdir, bundle_name, output = sys.argv[1:4]
bundle_path = Path(tmpdir) / bundle_name

with zipfile.ZipFile(output, 'w', zipfile.ZIP_DEFLATED) as zf:
    for root, dirs, files in sorted(os.walk(bundle_path)):
        dirs.sort()
        for file in sorted(files):
            file_path = Path(root) / file
            arcname = str(file_path.relative_to(tmpdir))
            zinfo = zipfile.ZipInfo(arcname)
            zinfo.date_time = (1980, 1, 1, 0, 0, 0)
            zinfo.external_attr = 0o644 << 16
            if 'costpilot' in file_path.name:
                zinfo.external_attr = 0o755 << 16
            with open(file_path, 'rb') as f:
                zf.writestr(zinfo, f.read(), compress_type=zipfile.ZIP_DEFLATED)
EOF
  else
    # Fallback to zip command
    (cd "$TMPDIR" && TZ=UTC zip -q -X -r "$ZIP_OUTPUT" "$BUNDLE_NAME")
  fi
fi

# Generate checksums
(
  cd "$OUT_DIR"
  sha256sum "$(basename "$TAR_OUTPUT")" "$(basename "$ZIP_OUTPUT")" | sort > sha256sum.txt
)

echo "BUNDLE: ${TAR_OUTPUT}"
