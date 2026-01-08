# GitHub Release Checklist

## Prerequisites
- [ ] All tests passing (1826/1826)
- [ ] Version tag created (`git tag -a v1.0.0 -m "message"`)
- [ ] CHANGELOG.md updated
- [ ] Release notes prepared (RELEASE_NOTES_v1.0.0.md)

## Build Artifacts
- [x] Linux x86_64 (native build)
- [ ] Linux ARM64 (requires cross-compilation or native ARM64 host)
- [ ] Windows x86_64 (requires Windows host or MSVC cross-toolchain)
- [ ] Windows ARM64 (requires Windows ARM64 host)
- [ ] macOS x86_64 (requires macOS host with Xcode)
- [ ] macOS ARM64 (requires macOS ARM64 host with Xcode)

## Release Process

### 1. Push Tag to GitHub
```bash
git push origin v1.0.0
```

### 2. Create GitHub Release
1. Go to https://github.com/Dee66/CostPilot/releases/new
2. Select tag: `v1.0.0`
3. Release title: `CostPilot v1.0.0 - Production Release`
4. Copy content from RELEASE_NOTES_v1.0.0.md
5. Check "Set as the latest release"

### 3. Upload Artifacts
Upload from `dist/` directory:
- [ ] costpilot-1.0.0-linux-amd64.tar.gz
- [ ] costpilot-1.0.0-linux-amd64.zip
- [ ] sha256sum.txt

### 4. Publish Release
- Review all information
- Click "Publish release"

## Post-Release
- [ ] Verify download links work
- [ ] Test installation from release artifacts
- [ ] Announce on relevant channels
- [ ] Update documentation links if needed

## Multi-Platform Build Strategy

### Option A: GitHub Actions (Recommended)
The `.github/workflows/release.yml` workflow handles multi-platform builds automatically:
```bash
# Trigger via GitHub UI: Actions → Release → Run workflow
```

### Option B: Manual Cross-Compilation
**Linux ARM64** (from Linux x86_64):
```bash
# Requires: sudo apt-get install gcc-aarch64-linux-gnu
rustup target add aarch64-unknown-linux-gnu
cargo build --release --target aarch64-unknown-linux-gnu
```

**Windows** (from Linux):
```bash
# Requires: mingw-w64 or xwin for MSVC
rustup target add x86_64-pc-windows-gnu
cargo build --release --target x86_64-pc-windows-gnu
```

**macOS**: Requires macOS host with Xcode installed.

### Option C: Multi-Boot Strategy
1. Build Linux on Linux Mint (current) ✓
2. Reboot to Windows, build Windows binaries
3. Use macOS VM or separate machine for macOS builds

## Current Status
- **Ready for Linux release**: Yes ✓
- **Artifacts location**: `dist/`
- **Checksums verified**: Yes ✓
- **GitHub tag created**: Yes ✓
- **Next step**: Push tag and create GitHub release
