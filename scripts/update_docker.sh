#!/bin/bash
set -euo pipefail

# Update Docker image for CostPilot
# Usage: ./scripts/update_docker.sh <version>

VERSION="${1:-}"
if [ -z "$VERSION" ]; then
    echo "Usage: $0 <version>"
    echo "Example: $0 v1.0.0"
    exit 1
fi

# Remove 'v' prefix if present
VERSION="${VERSION#v}"

echo "Building Docker image for CostPilot ${VERSION}"

# Build the Docker image
docker build -t costpilot:${VERSION} -t costpilot:latest .

echo "Docker image built successfully!"
echo "To push to registry:"
echo "  docker tag costpilot:${VERSION} your-registry/costpilot:${VERSION}"
echo "  docker push your-registry/costpilot:${VERSION}"