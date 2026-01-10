# Fork Build Guide for CostPilot

**Purpose**: Instructions for building CostPilot Windows binary via repository fork
**Target Audience**: Contributors with Windows machines building for v1.0.0 release
**Zero CI Cost**: All builds are local, no GitHub Actions triggered

---

## Overview

CostPilot v1.0.0 requires a Windows x86_64 binary that cannot be cross-compiled from Linux. This guide explains how to fork the repository, build the Windows .exe locally, and contribute it back for manual GitHub release.

### Why Fork?

- **No Windows CI**: GitHub Actions costs money; we use manual builds
- **Isolation**: Fork provides clean build environment
- **Safety**: No risk of triggering CI workflows during build process

---

## Prerequisites

### System Requirements

- **OS**: Windows 10 or Windows 11
- **Rust**: 1.91.1 or later
- **Build Tools**: MSVC Build Tools (Visual Studio 2019+)

### Install Rust (if not already installed)

```powershell
# Download and run rustup-init.exe from:
https://rustup.rs

# Verify installation
rustc --version
cargo --version
```

### Install MSVC Build Tools

1. Download Visual Studio Build Tools: https://visualstudio.microsoft.com/downloads/
2. Select "Desktop development with C++"
3. Install

---

## Step 1: Fork Repository

### Via GitHub UI

1. Navigate to: https://github.com/Dee66/CostPilot
2. Click "Fork" button (top-right)
3. Select your GitHub account
4. Wait for fork to complete

**Result**: You now have `YourUsername/CostPilot`

---

## Step 2: Clone Fork Locally

```powershell
# Clone your fork
git clone https://github.com/YourUsername/CostPilot.git
cd CostPilot

# Add upstream remote (optional, for syncing)
git remote add upstream https://github.com/Dee66/CostPilot.git

# Verify remotes
git remote -v
```

**Expected Output**:
```
origin    https://github.com/YourUsername/CostPilot.git (fetch)
origin    https://github.com/YourUsername/CostPilot.git (push)
upstream  https://github.com/Dee66/CostPilot.git (fetch)
upstream  https://github.com/Dee66/CostPilot.git (push)
```

---

## Step 3: Checkout Exact Commit

**Critical**: Build from exact commit documented in `WINDOWS_BUILD_HANDOFF.md`

```powershell
# Verify you're on the correct commit
git log --oneline -1

# Should show:
# 639e5475 (HEAD -> main, origin/main) chore: GTM preparation - audit reports, contract protection, and CI hardening

# If not, checkout the exact commit
git checkout 639e5475
```

**Verification**:
```powershell
git show HEAD --no-patch
```

**Expected**:
```
commit 639e5475
Author: Dee <dee@example.com>
Date:   Sat Jan 10 15:21:14 2026 +0200

    chore: GTM preparation - audit reports, contract protection, and CI hardening
```

---

## Step 4: Build Windows Binary

### Build Command

```powershell
cargo build --release --target x86_64-pc-windows-msvc
```

### Expected Output

```
   Compiling costpilot v1.0.0 (C:\path\to\CostPilot)
    Finished release [optimized] target(s) in 3m 42s
```

### Build Time

- **First build**: 3-5 minutes (compiles all dependencies)
- **Subsequent builds**: 30-60 seconds

### Build Artifact Location

```
target\x86_64-pc-windows-msvc\release\costpilot.exe
```

---

## Step 5: Validate Binary

### Version Check

```powershell
.\target\x86_64-pc-windows-msvc\release\costpilot.exe --version
```

**Expected Output**:
```
costpilot 1.0.0
```

### Binary Size Check

```powershell
Get-Item .\target\x86_64-pc-windows-msvc\release\costpilot.exe | Select-Object Name, Length
```

**Expected**:
- **Size**: 9-12 MB
- **Stop if**: Size > 20 MB (indicates debug build or issue)

### Smoke Test

```powershell
# Test help command
.\target\x86_64-pc-windows-msvc\release\costpilot.exe --help

# Test scan command (should show "File not found" error, which is expected)
.\target\x86_64-pc-windows-msvc\release\costpilot.exe scan nonexistent.json
```

**Expected**: Commands execute without crashing

---

## Step 6: Create Windows Release Archive

### Create Zip Archive

```powershell
# Create release directory
New-Item -ItemType Directory -Force -Path dist

# Copy binary
Copy-Item target\x86_64-pc-windows-msvc\release\costpilot.exe dist\

# Create zip archive (requires PowerShell 5.0+)
Compress-Archive -Path dist\costpilot.exe -DestinationPath dist\costpilot-1.0.0-windows-amd64.zip

# Verify archive
Get-Item dist\costpilot-1.0.0-windows-amd64.zip | Select-Object Name, Length
```

**Expected Archive Size**: 3-4 MB (compressed)

---

## Step 7: Generate SHA256 Checksum

```powershell
# Generate SHA256 checksum
Get-FileHash dist\costpilot-1.0.0-windows-amd64.zip -Algorithm SHA256 | Format-List

# Output to file
Get-FileHash dist\costpilot-1.0.0-windows-amd64.zip -Algorithm SHA256 |
    Select-Object -ExpandProperty Hash |
    Out-File -FilePath dist\sha256sum-windows.txt -Encoding ASCII
```

**Expected Output**:
```
Algorithm : SHA256
Hash      : <64-character hex string>
Path      : C:\path\to\CostPilot\dist\costpilot-1.0.0-windows-amd64.zip
```

---

## Step 8: Verify Embedded Keys (Security Check)

**Purpose**: Confirm binary contains correct public keys

### Extract Strings (Optional Verification)

```powershell
# Install strings.exe (Sysinternals) if not available
# https://docs.microsoft.com/en-us/sysinternals/downloads/strings

# Extract strings from binary
strings64.exe -n 64 .\target\x86_64-pc-windows-msvc\release\costpilot.exe | Select-String "db52fc95"
```

**Expected**: Should find `db52fc95fe7ccbd5...` (LICENSE public key)

### Key Fingerprints (From WINDOWS_BUILD_HANDOFF.md)

- **LICENSE**: `db52fc95fe7ccbd5e55ecfd357d8271d1b2d4a9f608e68db3e7f869d54dba5df`
- **WASM**: `8db250f6bf7cdf016fcc1564b2309897a701c4e4fa1946ca0eb9084f1c557994`

**Note**: Keys are embedded at compile time, cannot be changed post-build

---

## Step 9: Upload to Original Repository

### Manual Upload to GitHub Release

**Prerequisites**:
- Dee66 account access (or send artifacts to Dee66 maintainer)
- GitHub release for v1.0.0 must already exist

**Upload Steps**:

1. Navigate to: https://github.com/Dee66/CostPilot/releases/tag/v1.0.0
2. Click "Edit release"
3. Scroll to "Attach binaries by dropping them here or selecting them"
4. Upload:
   - `dist\costpilot-1.0.0-windows-amd64.zip`
   - `dist\sha256sum-windows.txt`
5. Click "Update release"

**Alternative**: Email artifacts to Dee66 maintainer for upload

---

## Troubleshooting

### Build Fails: "linker `link.exe` not found"

**Cause**: MSVC Build Tools not installed or not in PATH

**Fix**:
```powershell
# Find MSVC installation
& "C:\Program Files (x86)\Microsoft Visual Studio\Installer\vs_installer.exe"

# Ensure "Desktop development with C++" workload is installed
```

### Build Fails: "error: could not compile `costpilot`"

**Cause**: Missing Rust dependencies or outdated Rust version

**Fix**:
```powershell
# Update Rust
rustup update

# Clean and rebuild
cargo clean
cargo build --release --target x86_64-pc-windows-msvc
```

### Binary Size > 20 MB

**Cause**: Debug symbols or unoptimized build

**Fix**:
```powershell
# Verify release mode
cargo build --release --target x86_64-pc-windows-msvc

# Check Cargo.toml profile settings (should show opt-level = "z")
Get-Content Cargo.toml | Select-String -Pattern "opt-level"
```

### Version Shows "0.1.0" Instead of "1.0.0"

**Cause**: Wrong commit checked out

**Fix**:
```powershell
# Re-checkout correct commit
git checkout 639e5475

# Verify version in Cargo.toml
Get-Content Cargo.toml | Select-String -Pattern "version"

# Should show: version = "1.0.0"

# Clean and rebuild
cargo clean
cargo build --release --target x86_64-pc-windows-msvc
```

---

## Security Considerations

### Private Keys

**NEVER**:
- Commit private keys (`.pem` files) to fork
- Share private keys via email or Slack
- Push private keys to any GitHub repository

**Private keys are NOT required for building**. Only public keys are embedded in the binary at compile time.

### License Validation

**Test License** (Optional):
```json
{
  "email": "test@example.com",
  "license_key": "TEST-KEY-000",
  "expires": "2027-01-01T00:00:00Z",
  "signature": "<valid Ed25519 signature>",
  "issuer": "test-costpilot"
}
```

Save to `~\.costpilot\license.json` for local testing.

**Note**: Test licenses require test keypair (not included in repository).

---

## Verification Checklist

Before uploading Windows binary:

- [ ] Built from commit `639e5475`
- [ ] `costpilot.exe --version` shows `costpilot 1.0.0`
- [ ] Binary size is 9-12 MB
- [ ] Smoke tests pass (help, scan commands)
- [ ] Zip archive created: `costpilot-1.0.0-windows-amd64.zip`
- [ ] SHA256 checksum generated
- [ ] Embedded keys verified (optional)

---

## Fork Cleanup (Optional)

After successful build and upload:

```powershell
# Delete fork (via GitHub UI)
1. Go to: https://github.com/YourUsername/CostPilot/settings
2. Scroll to "Danger Zone"
3. Click "Delete this repository"
4. Confirm deletion

# Or keep fork for future contributions
```

---

## FAQ

### Q: Can I build on Linux with cross-compilation?

**A**: No. Rust cross-compilation to Windows requires complex setup and often fails for crates with native dependencies. Native Windows build is required.

### Q: Can I build on macOS with cross-compilation?

**A**: No. Same reason as Linux.

### Q: Do I need a CostPilot license to build?

**A**: No. License is only required for **running** Premium features, not for building.

### Q: Will this trigger GitHub Actions?

**A**: No. All workflows are manual (`workflow_dispatch`) or PR-gated. Local builds do not trigger CI.

### Q: Can I push changes to my fork?

**A**: Yes, but do NOT push build artifacts or private keys. The .gitignore already protects against this.

### Q: How do I sync my fork with upstream?

**A**:
```powershell
git fetch upstream
git merge upstream/main
git push origin main
```

---

## Support

### Issues

If you encounter build issues:

1. Check [Troubleshooting](#troubleshooting) section
2. Open an issue: https://github.com/Dee66/CostPilot/issues
3. Include:
   - Rust version (`rustc --version`)
   - Windows version
   - Full error output

### Contact

- **Maintainer**: Dee (Dee66 GitHub account)
- **Repository**: https://github.com/Dee66/CostPilot

---

## Summary

**Fork → Clone → Build → Verify → Upload**

1. Fork Dee66/CostPilot to your account
2. Clone fork locally on Windows
3. Checkout commit `639e5475`
4. Build with `cargo build --release --target x86_64-pc-windows-msvc`
5. Verify version and size
6. Create zip archive
7. Generate SHA256 checksum
8. Upload to Dee66/CostPilot release (or send to maintainer)

**Zero CI Cost**: All builds are local, no GitHub Actions triggered

---

**Guide Version**: 1.0
**Last Updated**: 2026-01-10
**Commit to Build**: 639e5475
