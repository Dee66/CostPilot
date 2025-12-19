#!/bin/bash
set -euo pipefail

# Update Homebrew formula for CostPilot
# Usage: ./scripts/update_homebrew.sh <version>

VERSION="${1:-}"
if [ -z "$VERSION" ]; then
    echo "Usage: $0 <version>"
    echo "Example: $0 v1.0.0"
    exit 1
fi

# Remove 'v' prefix if present
VERSION="${VERSION#v}"

echo "Updating Homebrew formula for CostPilot ${VERSION}"

# Calculate SHA256 hashes for different platforms
echo "Fetching SHA256 hashes..."

# macOS x86_64
X64_URL="https://github.com/Dee66/CostPilot/releases/download/v${VERSION}/costpilot-darwin-x86_64.tar.gz"
X64_SHA256=$(curl -sL "$X64_URL" | sha256sum | cut -d' ' -f1)
echo "macOS x86_64 SHA256: $X64_SHA256"

# macOS ARM64
ARM64_URL="https://github.com/Dee66/CostPilot/releases/download/v${VERSION}/costpilot-darwin-aarch64.tar.gz"
ARM64_SHA256=$(curl -sL "$ARM64_URL" | sha256sum | cut -d' ' -f1)
echo "macOS ARM64 SHA256: $ARM64_SHA256"

# Update the formula
FORMULA_FILE="Formula/costpilot.rb"

sed -i.bak \
    -e "s/version \".*\"/version \"$VERSION\"/" \
    -e "s|url \".*darwin-x86_64.tar.gz\"|url \"$X64_URL\"|" \
    -e "s/sha256 \".*\" # x64/sha256 \".*\" # x64/" \
    -e "s/REPLACE_WITH_ACTUAL_SHA256_X64/$X64_SHA256/" \
    -e "s/REPLACE_WITH_ACTUAL_SHA256_ARM/$ARM64_SHA256/" \
    "$FORMULA_FILE"

echo "Formula updated successfully!"
echo "Don't forget to commit and push the changes to the Homebrew tap."