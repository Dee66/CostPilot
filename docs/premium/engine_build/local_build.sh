#!/usr/bin/env bash
set -euo pipefail

# Local build wrapper for Pro Engine
# Auto-detects target and calls build_pro_engine.sh

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

# Detect target
TARGET="$(bash "${PROJECT_ROOT}/scripts/print_target.sh")"

# Verify encryption key
if [ -z "${PRO_ENGINE_ENC_KEY:-}" ]; then
    echo "ERROR: PRO_ENGINE_ENC_KEY not set"
    echo ""
    echo "Generate a key with:"
    echo "  export PRO_ENGINE_ENC_KEY=\$(openssl rand -hex 32)"
    exit 1
fi

echo "Building Pro Engine for ${TARGET}..."
bash "${PROJECT_ROOT}/premium/engine_build/build_pro_engine.sh"

echo ""
echo "Pro Engine build complete → dist/pro-engine-${TARGET}.wasm.enc"
