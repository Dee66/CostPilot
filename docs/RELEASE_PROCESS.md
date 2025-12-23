# Release Process Documentation

## Overview

This document describes the process for publishing versioned releases of CostPilot to GitHub, GitHub Actions Marketplace, and package registries.

## Release Versioning

CostPilot follows [Semantic Versioning](https://semver.org/):

**Format:** `MAJOR.MINOR.PATCH`

- **MAJOR:** Breaking changes (e.g., `1.0.0` → `2.0.0`)
- **MINOR:** New features, backward compatible (e.g., `1.0.0` → `1.1.0`)
- **PATCH:** Bug fixes, backward compatible (e.g., `1.0.0` → `1.0.1`)

**Examples:**
- `v1.0.0` - Initial stable release
- `v1.1.0` - Added Azure provider support
- `v1.1.1` - Fixed cost calculation bug
- `v2.0.0` - Changed policy DSL syntax (breaking)

## Pre-Release Checklist

Before creating a release:

- [ ] All tests pass (`cargo test`)
- [ ] Code coverage ≥ 80% (`cargo tarpaulin`)
- [ ] Benchmarks show no regressions (`cargo bench`)
- [ ] Documentation updated
- [ ] CHANGELOG.md updated with all changes
- [ ] Version bumped in `Cargo.toml`
- [ ] Version bumped in `package.json` (if applicable)
- [ ] GitHub Action `action.yml` updated with new version
- [ ] Examples tested with new version
- [ ] Security audit passed (`cargo audit`)
- [ ] No clippy warnings (`cargo clippy`)
- [ ] README badges updated (if version scheme changes)

## Release Process

### 1. Update Version Numbers

**Cargo.toml:**
```toml
[package]
name = "costpilot"
version = "1.1.0"  # Update this
```

**package.json (if applicable):**
```json
{
  "name": "costpilot",
  "version": "1.1.0"
}
```

**GitHub Action (action.yml):**
```yaml
# Add comment with version recommendation
# Users should pin: uses: Dee66/CostPilot@v1.1.0
```

### 2. Update CHANGELOG.md

Follow [Keep a Changelog](https://keepachangelog.com/) format:

```markdown
# Changelog

## [1.1.0] - 2025-12-15

### Added
- Azure provider support for cost estimation
- Cost anomaly detection with ML models
- Slack notification integration

### Changed
- Improved policy DSL error messages
- Faster Terraform plan parsing (30% speedup)

### Fixed
- Fixed incorrect RDS pricing for multi-AZ deployments
- Resolved drift detection false positives for tags

### Security
- Updated dependencies to address CVE-2025-XXXX

## [1.0.1] - 2025-12-01

### Fixed
- Fixed crash when parsing plans with null resource attributes
- Corrected cost calculation for spot instances

## [1.0.0] - 2025-11-15

### Added
- Initial stable release
- AWS provider support
- Policy engine with custom DSL
- Drift detection with SHA256 checksums
- GitHub Actions integration
```

### 3. Create Git Tag

```bash
# Ensure you're on main branch and up to date
git checkout main
git pull origin main

# Create annotated tag
git tag -a v1.1.0 -m "Release v1.1.0 - Azure support and ML anomaly detection"

# Push tag to remote
git push origin v1.1.0
```

### 4. Build Release Binaries

Use CI/CD to build cross-platform binaries:

**GitHub Actions Workflow (`.github/workflows/release.yml`):**

```yaml
name: Release

on:
  push:
    tags:
      - 'v*.*.*'

jobs:
  build:
    strategy:
      matrix:
        include:
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
            artifact: costpilot-linux-x86_64
          - os: ubuntu-latest
            target: aarch64-unknown-linux-gnu
            artifact: costpilot-linux-aarch64
          - os: macos-latest
            target: x86_64-apple-darwin
            artifact: costpilot-macos-x86_64
          - os: macos-latest
            target: aarch64-apple-darwin
            artifact: costpilot-macos-aarch64

    runs-on: ${{ matrix.os }}

    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          target: ${{ matrix.target }}

      - name: Build Release Binary
        run: cargo build --release --target ${{ matrix.target }}

      - name: Strip Binary (Linux/macOS)
        if: runner.os != 'Windows'
        run: strip target/${{ matrix.target }}/release/costpilot

      - name: Package Binary
        run: |
          mkdir -p artifacts
          cp target/${{ matrix.target }}/release/costpilot artifacts/${{ matrix.artifact }}
          cd artifacts
          tar -czf ${{ matrix.artifact }}.tar.gz ${{ matrix.artifact }}
          sha256sum ${{ matrix.artifact }}.tar.gz > ${{ matrix.artifact }}.tar.gz.sha256

      - name: Upload Artifacts
        uses: actions/upload-artifact@v3
        with:
          name: ${{ matrix.artifact }}
          path: artifacts/${{ matrix.artifact }}.tar.gz*

  release:
    needs: build
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v4

      - name: Download All Artifacts
        uses: actions/download-artifact@v3

      - name: Extract Version
        id: version
        run: echo "VERSION=${GITHUB_REF#refs/tags/}" >> $GITHUB_OUTPUT

      - name: Generate Release Notes
        run: |
          # Extract changelog for this version
          sed -n '/## \[${{ steps.version.outputs.VERSION }}\]/,/## \[/p' CHANGELOG.md | head -n -1 > release_notes.md

      - name: Create GitHub Release
        uses: softprops/action-gh-release@v1
        with:
          tag_name: ${{ steps.version.outputs.VERSION }}
          name: CostPilot ${{ steps.version.outputs.VERSION }}
          body_path: release_notes.md
          files: |
            costpilot-linux-x86_64/*
            costpilot-linux-aarch64/*
            costpilot-macos-x86_64/*
            costpilot-macos-aarch64/*
          draft: false
          prerelease: false
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
```

### 5. Create GitHub Release

**Manual Process (if not using CI/CD):**

1. Go to https://github.com/Dee66/CostPilot/releases/new
2. Choose tag: `v1.1.0`
3. Release title: `CostPilot v1.1.0`
4. Description: Copy from CHANGELOG.md
5. Upload binaries:
   - `costpilot-linux-x86_64.tar.gz`
   - `costpilot-linux-aarch64.tar.gz`
   - `costpilot-macos-x86_64.tar.gz`
   - `costpilot-macos-aarch64.tar.gz`
   - SHA256 checksums for each
6. Check "Set as the latest release"
7. Click "Publish release"

**Release Notes Template:**

```markdown
## 🎉 CostPilot v1.1.0

### What's New

- **Azure Support:** Full cost estimation for Azure resources
- **ML Anomaly Detection:** Automatically detect cost anomalies using machine learning
- **Slack Integration:** Get notified of cost changes in Slack channels

### Improvements

- 30% faster Terraform plan parsing
- Better error messages for policy DSL syntax errors
- Enhanced drift detection accuracy

### Bug Fixes

- Fixed incorrect RDS multi-AZ pricing
- Resolved false positives in drift detection for tag changes
- Corrected spot instance cost calculations

### Installation

**Linux (x86_64):**
```bash
curl -L https://github.com/Dee66/CostPilot/releases/download/v1.1.0/costpilot-linux-x86_64.tar.gz | tar xz
sudo mv costpilot /usr/local/bin/
```

**macOS (Apple Silicon):**
```bash
curl -L https://github.com/Dee66/CostPilot/releases/download/v1.1.0/costpilot-macos-aarch64.tar.gz | tar xz
sudo mv costpilot /usr/local/bin/
```

**GitHub Action:**
```yaml
- uses: Dee66/CostPilot@v1.1.0
```

### Checksums

Verify downloads with SHA256:
```
abc123... costpilot-linux-x86_64.tar.gz
def456... costpilot-linux-aarch64.tar.gz
ghi789... costpilot-macos-x86_64.tar.gz
jkl012... costpilot-macos-aarch64.tar.gz
```

### Full Changelog

https://github.com/Dee66/CostPilot/compare/v1.0.1...v1.1.0
```

### 6. Update Major Version Tag

For GitHub Actions, maintain a moving `v1` tag pointing to latest `v1.x.x`:

```bash
# Delete old v1 tag (locally and remotely)
git tag -d v1
git push origin :refs/tags/v1

# Create new v1 tag pointing to v1.1.0
git tag v1
git push origin v1
```

This allows users to use `Dee66/CostPilot@v1` and automatically get the latest v1.x.x.

### 7. Publish to GitHub Actions Marketplace

**First Time Setup:**

1. Go to https://github.com/Dee66/CostPilot/releases
2. Click "Draft a new release"
3. Enable "Publish this Action to the GitHub Marketplace"
4. Fill in marketplace details:
   - **Primary Category:** Deployment
   - **Icon:** dollar-sign
   - **Color:** green
   - **Keywords:** terraform, cost, finops, infrastructure-as-code, policy
5. Agree to GitHub Marketplace Terms

**Subsequent Releases:**

Marketplace automatically updates when you create a new release if "Publish to Marketplace" was enabled initially.

### 8. Update Documentation

Update version references in:

- `README.md` (installation instructions)
- `docs/CLI_QUICKSTART.md`
- `docs/cli_reference.md`
- `examples/` (if version-specific examples exist)
- Install script (`install.sh`)

**install.sh:**
```bash
#!/bin/bash
set -e

VERSION="${VERSION:-v1.1.0}"  # Update this
REPO="Dee66/CostPilot"

# Detection and download logic...
```

### 9. Announce Release

**GitHub Discussions:**
Create announcement post at https://github.com/Dee66/CostPilot/discussions

**Twitter/X:**
```
🚀 CostPilot v1.1.0 is out!

✨ Azure support
🤖 ML-powered anomaly detection
💬 Slack notifications

Install: curl -fsSL https://costpilot.dev/install.sh | bash

Full notes: https://github.com/Dee66/CostPilot/releases/tag/v1.1.0

#DevOps #FinOps #Terraform
```

**Reddit:**
Post to r/devops, r/terraform, r/aws

**Hacker News:**
Consider "Show HN: CostPilot v1.1.0" if major feature release

**Blog Post:**
Write detailed blog post covering new features and migration guide

## Release Cadence

**Recommended Schedule:**

- **Patch Releases (1.0.x):** As needed for critical bugs (within 24-48 hours)
- **Minor Releases (1.x.0):** Every 4-6 weeks for new features
- **Major Releases (x.0.0):** Every 6-12 months for breaking changes

**Examples:**

- Critical security fix → Patch release within 24 hours
- New cloud provider support → Minor release in planned sprint
- Policy DSL syntax change → Major release with 3-month advance notice

## Breaking Changes

When introducing breaking changes:

1. **Announce Early:** Blog post + GitHub discussion 3 months before release
2. **Provide Migration Guide:** Document all changes and upgrade steps
3. **Deprecation Period:** Mark old features as deprecated in previous minor release
4. **Backward Compatibility:** Support old behavior with warnings if possible
5. **Major Version Bump:** `v1.x.x` → `v2.0.0`

**Migration Guide Template:**

```markdown
# Migrating to CostPilot v2.0.0

## Breaking Changes

### 1. Policy DSL Syntax Changed

**Old (v1.x):**
```json
{
  "rule": "cost < 1000"
}
```

**New (v2.x):**
```json
{
  "rule": "monthly_cost <= 1000"
}
```

**Migration:**
Replace `cost` with `monthly_cost` in all policy files.

### 2. Removed Deprecated Flags

- `--baseline-path` (use `--baseline-file` instead)
- `--no-color` (use `--color=never` instead)

**Migration:**
Update CI/CD scripts with new flag names.

## New Features

- Multi-cloud support (AWS, Azure, GCP)
- Enhanced ML predictions
- Real-time cost tracking

## Upgrade Steps

1. Update binary: `curl -fsSL https://costpilot.dev/install.sh | bash`
2. Update policies: `costpilot migrate-policies --from v1 --to v2`
3. Test locally: `costpilot analyze --plan plan.json`
4. Update CI/CD: Change `Dee66/CostPilot@v1` to `@v2`
```

## Rollback Plan

If a release has critical issues:

### 1. Immediate Actions

```bash
# Mark release as pre-release (reduces visibility)
gh release edit v1.1.0 --prerelease

# Or delete release entirely
gh release delete v1.1.0 --yes
git push origin :refs/tags/v1.1.0
```

### 2. Revert Major Version Tag

```bash
# Point v1 back to previous stable version
git tag -d v1
git push origin :refs/tags/v1
git tag v1 v1.0.1
git push origin v1
```

### 3. Publish Hotfix

```bash
# Create hotfix branch from previous stable
git checkout -b hotfix/1.1.1 v1.0.1

# Apply fix
git cherry-pick <commit-hash>

# Release v1.1.1 with fix
git tag v1.1.1
git push origin v1.1.1
```

### 4. Communicate

Post to all channels:
- GitHub Discussions
- Twitter/X
- Reddit posts
- Email users (if mailing list exists)

**Template:**
```
⚠️ CostPilot v1.1.0 Rollback Notice

We've identified a critical issue in v1.1.0 and have rolled back to v1.0.1.

Issue: [Brief description]
Impact: [What's affected]
Fix: Use v1.0.1 or wait for v1.1.1 (ETA: 24 hours)

Apologies for the inconvenience. We're improving our testing to prevent this.

Details: https://github.com/Dee66/CostPilot/issues/XXX
```

## Release Automation

### GitHub Actions Workflow

Create `.github/workflows/release.yml`:

```yaml
name: Release

on:
  push:
    tags:
      - 'v*.*.*'

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Run Tests
        run: cargo test --all-features
      - name: Run Clippy
        run: cargo clippy -- -D warnings
      - name: Check Coverage
        run: cargo tarpaulin --out Xml --all-features

  build-and-release:
    needs: test
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Build Cross-Platform Binaries
        uses: rust-build/rust-build.action@v1.4.4
        with:
          RUSTTARGET: |
            x86_64-unknown-linux-gnu
            aarch64-unknown-linux-gnu
            x86_64-apple-darwin
            aarch64-apple-darwin
          UPLOAD_MODE: release
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}

      - name: Generate Changelog
        run: ./scripts/generate_changelog.sh ${{ github.ref_name }}

      - name: Update Homebrew Formula
        run: ./scripts/update_homebrew.sh ${{ github.ref_name }}
```

### Release Script

Create `scripts/release.sh`:

```bash
#!/bin/bash
set -e

VERSION=$1

if [ -z "$VERSION" ]; then
  echo "Usage: ./scripts/release.sh v1.1.0"
  exit 1
fi

echo "🚀 Starting release process for $VERSION"

# 1. Update version in Cargo.toml
sed -i "s/^version = .*/version = \"${VERSION#v}\"/" Cargo.toml

# 2. Update CHANGELOG.md
./scripts/generate_changelog.sh $VERSION

# 3. Commit changes
git add Cargo.toml CHANGELOG.md
git commit -m "chore: bump version to $VERSION"
git push origin main

# 4. Create and push tag
git tag -a $VERSION -m "Release $VERSION"
git push origin $VERSION

# 5. Wait for CI to build and publish
echo "✅ Release process started. CI will build and publish binaries."
echo "📦 Monitor progress: https://github.com/Dee66/CostPilot/actions"
```

## Post-Release Checklist

After publishing a release:

- [ ] Verify binaries download correctly from GitHub releases
- [ ] Test installation script on fresh Ubuntu VM
- [ ] Test GitHub Action with new version
- [ ] Update documentation site with new version
- [ ] Check GitHub Marketplace listing is updated
- [ ] Monitor GitHub issues for bug reports
- [ ] Respond to community feedback (GitHub Discussions, Reddit, HN)
- [ ] Update project roadmap based on feedback
- [ ] Schedule next release planning meeting

## Version Support Policy

**Active Support:**
- Latest major version (v2.x.x)
- Previous major version (v1.x.x) for 6 months after v2.0.0 release

**Security Patches:**
- Latest major version (v2.x.x)
- Previous major version (v1.x.x) for 12 months

**End of Life:**
- No support after 12 months from next major version

**Example Timeline:**

- **Nov 2025:** v1.0.0 released
- **May 2026:** v2.0.0 released
  - v1.x.x enters maintenance mode (security patches only)
- **Nov 2026:** v1.x.x reaches end of life
  - No further updates to v1.x.x

## Troubleshooting

### Issue: Tag already exists

```bash
# Delete local tag
git tag -d v1.1.0

# Delete remote tag
git push origin :refs/tags/v1.1.0

# Recreate tag
git tag -a v1.1.0 -m "Release v1.1.0"
git push origin v1.1.0
```

### Issue: CI build fails

1. Check GitHub Actions logs
2. Fix issue in code
3. Delete tag and release
4. Recreate tag after fix

### Issue: Binary doesn't work on target platform

1. Test locally with cross-compilation
2. Add platform to test matrix
3. Consider adding integration tests in CI

## References

- [Semantic Versioning](https://semver.org/)
- [Keep a Changelog](https://keepachangelog.com/)
- [GitHub Releases](https://docs.github.com/en/repositories/releasing-projects-on-github)
- [GitHub Actions Marketplace](https://docs.github.com/en/actions/creating-actions/publishing-actions-in-github-marketplace)

---

**Last Updated:** December 2025
**Version:** 1.0.0
