#!/bin/bash
# Release script for CostPilot
set -e

VERSION=$1

if [ -z "$VERSION" ]; then
  echo "Usage: ./scripts/release.sh v1.0.0"
  exit 1
fi

# Validate version format
if ! [[ "$VERSION" =~ ^v[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9]+)?$ ]]; then
  echo "Error: Invalid version format. Use vX.Y.Z or vX.Y.Z-tag"
  exit 1
fi

VERSION_NUM="${VERSION#v}"

echo "🚀 Starting release process for $VERSION"

# Check for uncommitted changes
if [[ -n $(git status -s) ]]; then
  echo "Error: Working directory has uncommitted changes"
  git status -s
  exit 1
fi

# Ensure we're on main branch
CURRENT_BRANCH=$(git rev-parse --abbrev-ref HEAD)
if [ "$CURRENT_BRANCH" != "main" ]; then
  echo "Error: Must be on main branch (currently on $CURRENT_BRANCH)"
  exit 1
fi

# Pull latest changes
echo "📥 Pulling latest changes..."
git pull origin main

# Update version in Cargo.toml
echo "📝 Updating version in Cargo.toml..."
sed -i.bak "s/^version = \".*\"/version = \"$VERSION_NUM\"/" Cargo.toml
rm -f Cargo.toml.bak

# Update version in package.json if it exists
if [ -f "package.json" ]; then
  echo "📝 Updating version in package.json..."
  sed -i.bak "s/\"version\": \".*\"/\"version\": \"$VERSION_NUM\"/" package.json
  rm -f package.json.bak
fi

# Run tests
echo "🧪 Running tests..."
cargo test --all-features

# Run clippy
echo "🔍 Running clippy..."
cargo clippy --all-features -- -D warnings

# Check formatting
echo "✨ Checking formatting..."
cargo fmt -- --check

# Build release binaries
echo "🔨 Building release binary..."
cargo build --release

# Generate or update CHANGELOG.md
echo "📋 Generating changelog..."
./scripts/generate_changelog.sh "$VERSION"

# Commit version changes
echo "💾 Committing version changes..."
git add Cargo.toml package.json CHANGELOG.md 2>/dev/null || git add Cargo.toml CHANGELOG.md
git commit -m "chore: bump version to $VERSION"

# Push changes
echo "📤 Pushing changes..."
git push origin main

# Create and push tag
echo "🏷️  Creating tag $VERSION..."
git tag -a "$VERSION" -m "Release $VERSION"
git push origin "$VERSION"

# Update major version tag (e.g., v1 -> v1.2.3)
if [[ ! "$VERSION" =~ - ]]; then
  MAJOR_VERSION=$(echo "$VERSION" | cut -d. -f1)
  echo "🏷️  Updating major version tag $MAJOR_VERSION..."
  
  git tag -d "$MAJOR_VERSION" 2>/dev/null || true
  git push origin ":refs/tags/$MAJOR_VERSION" 2>/dev/null || true
  
  git tag "$MAJOR_VERSION"
  git push origin "$MAJOR_VERSION"
fi

echo ""
echo "✅ Release process started successfully!"
echo ""
echo "Next steps:"
echo "  1. Monitor GitHub Actions: https://github.com/Dee66/CostPilot/actions"
echo "  2. CI will build binaries and create GitHub release"
echo "  3. Verify release: https://github.com/Dee66/CostPilot/releases/tag/$VERSION"
echo ""
echo "To rollback if needed:"
echo "  git tag -d $VERSION"
echo "  git push origin :refs/tags/$VERSION"
echo ""
