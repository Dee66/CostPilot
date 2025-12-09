#!/bin/bash
# Binary fetcher script for GitHub Action

set -e

REPO="Dee66/CostPilot"
VERSION="${COSTPILOT_VERSION:-latest}"

# Determine platform
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

# Map architecture names
case $ARCH in
  x86_64) ARCH="x86_64" ;;
  aarch64|arm64) ARCH="aarch64" ;;
  *)
    echo "Error: Unsupported architecture: $ARCH"
    echo "Supported: x86_64, aarch64/arm64"
    exit 1
    ;;
esac

# Map OS names
case $OS in
  linux) OS="linux" ;;
  darwin) OS="macos" ;;
  *)
    echo "Error: Unsupported OS: $OS"
    echo "Supported: linux, darwin/macos"
    exit 1
    ;;
esac

echo "🔍 Detecting platform: ${OS}-${ARCH}"

# Get latest version if not specified
if [ "$VERSION" = "latest" ]; then
  echo "📡 Fetching latest release version..."
  VERSION=$(curl -s "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name"' | sed -E 's/.*"([^"]+)".*/\1/')
  
  if [ -z "$VERSION" ]; then
    echo "Error: Failed to fetch latest version"
    exit 1
  fi
  
  echo "✓ Latest version: $VERSION"
fi

# Construct download URL
BINARY_NAME="costpilot-${OS}-${ARCH}"
DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${VERSION}/${BINARY_NAME}"

echo "⬇️  Downloading CostPilot ${VERSION}..."
echo "   URL: ${DOWNLOAD_URL}"

# Download binary
if command -v curl &> /dev/null; then
  curl -fsSL -o costpilot "${DOWNLOAD_URL}"
elif command -v wget &> /dev/null; then
  wget -q -O costpilot "${DOWNLOAD_URL}"
else
  echo "Error: Neither curl nor wget found. Please install one of them."
  exit 1
fi

# Verify download
if [ ! -f "costpilot" ]; then
  echo "Error: Download failed - binary not found"
  exit 1
fi

# Make executable
chmod +x costpilot

# Verify binary works
echo "✓ Binary downloaded successfully"
echo "📦 Verifying installation..."

if ./costpilot --version; then
  echo "✅ CostPilot is ready to use!"
else
  echo "Error: Binary verification failed"
  exit 1
fi

# Optionally move to PATH
if [ -n "$INSTALL_PATH" ]; then
  echo "📌 Installing to ${INSTALL_PATH}..."
  mv costpilot "${INSTALL_PATH}/costpilot"
  echo "✓ Installed to PATH"
fi
