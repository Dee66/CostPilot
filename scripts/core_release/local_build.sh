#!/usr/bin/env bash
set -euo pipefail

# Local build wrapper for core release packages

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

# Extract version
VERSION=$(grep '^version' "${PROJECT_ROOT}/Cargo.toml" | head -1 | cut -d'"' -f2)

echo "Building CostPilot ${VERSION} core release packages..."
echo ""

# Run build script
bash "${PROJECT_ROOT}/packaging/core_release/build_core_package.sh"

echo ""
echo "Output artifacts:"
echo "  dist/costpilot-${VERSION}-linux-x64.zip"
echo "  dist/costpilot-${VERSION}-linux-x64.tar.gz"
