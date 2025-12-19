#!/bin/bash
set -euo pipefail

# Update PyPI package for CostPilot
# Usage: ./scripts/update_pypi.sh <version>

VERSION="${1:-}"
if [ -z "$VERSION" ]; then
    echo "Usage: $0 <version>"
    echo "Example: $0 v1.0.0"
    exit 1
fi

# Remove 'v' prefix if present
VERSION="${VERSION#v}"

echo "Updating PyPI package for CostPilot ${VERSION}"

# Update setup.py
sed -i.bak "s/version=\".*\"/version=\"$VERSION\"/" setup.py

# Update pyproject.toml
sed -i.bak "s/version = \".*\"/version = \"$VERSION\"/" pyproject.toml

echo "Python package files updated successfully!"
echo "To publish to PyPI:"
echo "  python -m build"
echo "  python -m twine upload dist/*"