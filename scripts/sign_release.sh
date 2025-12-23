#!/bin/bash
set -euo pipefail

# Sign CostPilot release bundles with GPG
# Usage: ./scripts/sign_release.sh <version> <key_id>

VERSION="${1:-}"
KEY_ID="${2:-}"

if [ -z "$VERSION" ]; then
    echo "Usage: $0 <version> [key_id]"
    echo "Example: $0 1.0.0"
    echo "Example: $0 1.0.0 ABC123DEF"
    exit 1
fi

echo "Signing CostPilot release bundles for version ${VERSION}"

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DIST_DIR="${PROJECT_ROOT}/dist"

# Check if dist directory exists
if [ ! -d "$DIST_DIR" ]; then
    echo "Error: dist directory not found. Run release build first."
    exit 1
fi

# Find GPG key if not provided
if [ -z "$KEY_ID" ]; then
    echo "Available GPG keys:"
    gpg --list-secret-keys --keyid-format SHORT
    echo
    read -p "Enter GPG key ID to use for signing: " KEY_ID
fi

# Verify GPG key exists
if ! gpg --list-secret-keys "$KEY_ID" >/dev/null 2>&1; then
    echo "Error: GPG key $KEY_ID not found"
    exit 1
fi

echo "Using GPG key: $KEY_ID"

# Sign each release bundle
for bundle in "$DIST_DIR"/*; do
    if [[ -f "$bundle" ]] && [[ "$bundle" == *.zip || "$bundle" == *.tar.gz ]]; then
        echo "Signing $(basename "$bundle")..."

        # Create detached signature
        gpg --detach-sign \
            --armor \
            --local-user "$KEY_ID" \
            --output "${bundle}.asc" \
            "$bundle"

        # Create SHA256 checksum
        sha256sum "$bundle" > "${bundle}.sha256"

        # Sign the checksum file
        gpg --detach-sign \
            --armor \
            --local-user "$KEY_ID" \
            --output "${bundle}.sha256.asc" \
            "${bundle}.sha256"

        echo "✅ Signed $(basename "$bundle")"
    fi
done

# Create a summary file with all checksums
echo "Creating checksum summary..."
CHECKSUM_FILE="$DIST_DIR/SHA256SUMS"
> "$CHECKSUM_FILE"

for bundle in "$DIST_DIR"/*.sha256; do
    if [[ -f "$bundle" ]]; then
        cat "$bundle" >> "$CHECKSUM_FILE"
    fi
done

# Sign the checksum summary
gpg --detach-sign \
    --armor \
    --local-user "$KEY_ID" \
    --output "${CHECKSUM_FILE}.asc" \
    "$CHECKSUM_FILE"

echo
echo "🎉 Release signing complete!"
echo
echo "Signed files in $DIST_DIR:"
ls -la "$DIST_DIR"/*.asc "$DIST_DIR"/*.sha256

echo
echo "To verify signatures:"
echo "  gpg --verify <file>.asc <file>"
echo "  gpg --verify SHA256SUMS.asc SHA256SUMS"
echo
echo "To verify checksums:"
echo "  sha256sum -c SHA256SUMS"</content>
<parameter name="filePath">/home/dee/workspace/AI/GuardSuite/CostPilot/scripts/sign_release.sh
