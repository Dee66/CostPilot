#!/usr/bin/env bash
set -euo pipefail

# Print cargo target triple for current platform
# Used by packaging scripts to determine platform-specific archive names

# Detect OS
OS="$(uname -s)"
ARCH="$(uname -m)"

case "${OS}" in
    Linux*)
        OS_NAME="linux"
        ;;
    Darwin*)
        OS_NAME="macos"
        ;;
    MINGW*|MSYS*|CYGWIN*)
        OS_NAME="windows"
        ;;
    *)
        echo "unknown-os"
        exit 1
        ;;
esac

# Normalize architecture
case "${ARCH}" in
    x86_64|amd64)
        ARCH_NAME="amd64"
        ;;
    aarch64|arm64)
        ARCH_NAME="arm64"
        ;;
    *)
        echo "unknown-arch"
        exit 1
        ;;
esac

echo "${OS_NAME}-${ARCH_NAME}"
