#!/bin/bash
# Generate changelog for a version
set -e

VERSION=$1

if [ -z "$VERSION" ]; then
  echo "Usage: ./scripts/generate_changelog.sh v1.0.0"
  exit 1
fi

VERSION_NUM="${VERSION#v}"
CHANGELOG_FILE="CHANGELOG.md"
TEMP_FILE="CHANGELOG.tmp"

# Get the date
DATE=$(date +%Y-%m-%d)

# Get previous tag
PREVIOUS_TAG=$(git describe --tags --abbrev=0 HEAD^ 2>/dev/null || echo "")

echo "📋 Generating changelog for $VERSION (since $PREVIOUS_TAG)"

# Create or update CHANGELOG.md
if [ ! -f "$CHANGELOG_FILE" ]; then
  echo "Creating new CHANGELOG.md"
  cat > "$CHANGELOG_FILE" << EOF
# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

EOF
fi

# Generate commit log
if [ -n "$PREVIOUS_TAG" ]; then
  COMMIT_RANGE="$PREVIOUS_TAG..HEAD"
else
  COMMIT_RANGE="HEAD"
fi

# Extract commits by type
ADDED=$(git log "$COMMIT_RANGE" --pretty=format:"- %s" --grep="^feat" 2>/dev/null || echo "")
CHANGED=$(git log "$COMMIT_RANGE" --pretty=format:"- %s" --grep="^refactor\|^perf" 2>/dev/null || echo "")
FIXED=$(git log "$COMMIT_RANGE" --pretty=format:"- %s" --grep="^fix" 2>/dev/null || echo "")
SECURITY=$(git log "$COMMIT_RANGE" --pretty=format:"- %s" --grep="^security" 2>/dev/null || echo "")

# Build new entry
NEW_ENTRY="## [$VERSION_NUM] - $DATE"

if [ -n "$ADDED" ]; then
  NEW_ENTRY="$NEW_ENTRY

### Added
$ADDED"
fi

if [ -n "$CHANGED" ]; then
  NEW_ENTRY="$NEW_ENTRY

### Changed
$CHANGED"
fi

if [ -n "$FIXED" ]; then
  NEW_ENTRY="$NEW_ENTRY

### Fixed
$FIXED"
fi

if [ -n "$SECURITY" ]; then
  NEW_ENTRY="$NEW_ENTRY

### Security
$SECURITY"
fi

# If no conventional commits found, add manual section
if [ -z "$ADDED" ] && [ -z "$CHANGED" ] && [ -z "$FIXED" ] && [ -z "$SECURITY" ]; then
  NEW_ENTRY="$NEW_ENTRY

### Added
- Initial release of CostPilot
- Cost estimation for Terraform plans
- Policy enforcement with custom DSL
- Drift detection with SHA256 checksums
- AI-powered cost predictions
- GitHub Actions integration
- PR comment automation"
fi

# Insert new entry after header
{
  # Keep the header
  sed -n '1,/^$/p' "$CHANGELOG_FILE"
  
  # Add new entry
  echo "$NEW_ENTRY"
  echo ""
  
  # Keep rest of changelog (skip header)
  sed '1,/^$/d' "$CHANGELOG_FILE"
} > "$TEMP_FILE"

mv "$TEMP_FILE" "$CHANGELOG_FILE"

echo "✅ Updated $CHANGELOG_FILE"
echo ""
echo "Please review and edit $CHANGELOG_FILE before committing"
echo "Pay special attention to:"
echo "  - Removing internal/non-user-facing changes"
echo "  - Grouping related changes"
echo "  - Adding context where needed"
echo ""
