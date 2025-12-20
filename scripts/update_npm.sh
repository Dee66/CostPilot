#!/bin/bash
set -euo pipefail

# Update npm package for CostPilot
# Usage: ./scripts/update_npm.sh <version>

VERSION="${1:-}"
if [ -z "$VERSION" ]; then
    echo "Usage: $0 <version>"
    echo "Example: $0 v1.0.0"
    exit 1
fi

# Remove 'v' prefix if present
VERSION="${VERSION#v}"

echo "Updating npm package for CostPilot ${VERSION}"

# Update package.json
sed -i.bak "s/\"version\": \".*\"/\"version\": \"$VERSION\"/" package.json

echo "npm package.json updated successfully!"
echo "To publish to npm:"
echo "  npm publish"
