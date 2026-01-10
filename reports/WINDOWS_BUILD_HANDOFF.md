# Windows Build Handoff - Phase 5

**Date**: 2026-01-10
**Commit**: 02c35f75
**Purpose**: Provide exact Windows build instructions - no further Linux work required

---

## Summary

**Build Status**: ✅ **READY FOR WINDOWS BUILD**

**All Linux work complete**:
- ✅ Security sweep (1 old key removed)
- ✅ License E2E proof (47 tests passing)
- ✅ Contract frozen (10 protection tests, CI enforcement)
- ✅ CI cost safe ($0/month expected)

**Windows Task**: Build single `.exe` from commit 02c35f75

---

## Build Specification

### Exact Commit
```
Commit: 02c35f75
Branch: main
Date: 2025-01-09
Message: fix(build): Add cross-compilation config for ARM64 Linux
```

**Verification**:
```bash
git log --oneline -1
# Expected: 02c35f75 (HEAD -> main, origin/main) fix(build): Add cross-compilation config for ARM64 Linux
```

---

### Rust Toolchain

**Version**: 1.91.1 (or later stable)

**Linux Reference**:
```
rustc 1.91.1 (ed61e7d7e 2025-11-07)
cargo 1.91.1 (ea2d97820 2025-10-10)
```

**Windows Setup**:
```powershell
# Install Rust
# Download from: https://rustup.rs/
# Or use: winget install Rustlang.Rustup

# Verify installation
rustc --version  # Should be 1.91.1 or later
cargo --version
```

---

### Build Command

**Target**: `x86_64-pc-windows-msvc`

**Command**:
```powershell
# Clone repository
git clone https://github.com/yourusername/CostPilot.git
cd CostPilot
git checkout 02c35f75

# Build release binary
cargo build --release --target x86_64-pc-windows-msvc

# Binary location
# target\x86_64-pc-windows-msvc\release\costpilot.exe
```

**Alternative** (if target not installed):
```powershell
rustup target add x86_64-pc-windows-msvc
cargo build --release --target x86_64-pc-windows-msvc
```

---

### Expected Output

**Binary**: `target\x86_64-pc-windows-msvc\release\costpilot.exe`

**Expected Size**: ~9-12 MB (based on Linux binary: 9.4 MB)

**Expected Build Time**: 5-10 minutes (first build), 30 seconds (incremental)

---

### Build Verification

**Command**:
```powershell
.\target\x86_64-pc-windows-msvc\release\costpilot.exe --version
```

**Expected Output**:
```
costpilot 1.0.0
```

**Smoke Test**:
```powershell
# Test Free edition (no license)
.\target\x86_64-pc-windows-msvc\release\costpilot.exe scan --dry-run examples\basic\simple.yml

# Expected: No errors, edition=Free
```

---

## Dependencies

### System Requirements

**Windows Version**: Windows 10 or later

**MSVC Build Tools**: Required for x86_64-pc-windows-msvc target

**Installation** (if not present):
```powershell
# Option 1: Visual Studio Build Tools
# Download: https://visualstudio.microsoft.com/downloads/
# Select: "Desktop development with C++"

# Option 2: Windows SDK
# Included with Visual Studio or standalone
```

**Verification**:
```powershell
# Check if MSVC is available
cl.exe
# Should output version info
```

---

### Rust Dependencies

**All dependencies vendored via Cargo.toml**:
- ed25519-dalek (cryptography)
- chrono (timestamps)
- serde/serde_json (serialization)
- clap (CLI parsing)
- tokio (async runtime)
- No external system libraries required

**OpenSSL**: Not required (vendored via openssl-src crate)

---

## License Key Configuration

### Production Public Keys

**Embedded at compile time** (build.rs):
- LICENSE: `db52fc95fe7ccbd5e55ecfd357d8271d1b2d4a9f608e68db3e7f869d54dba5df`
- WASM: `8db250f6bf7cdf016fcc1564b2309897a701c4e4fa1946ca0eb9084f1c557994`

**Fingerprints**:
```
License key fingerprint: db52fc95
WASM key fingerprint: 8db250f6
```

**Windows binary will have same keys** (compiled from same build.rs)

---

### License File Location (Windows)

**Path**: `%USERPROFILE%\.costpilot\license.json`

**Example**: `C:\Users\YourName\.costpilot\license.json`

**Format** (same as Linux):
```json
{
  "email": "customer@example.com",
  "license_key": "PREMIUM-1234-5678",
  "expires": "2027-01-10T00:00:00Z",
  "signature": "a87221177e36270b83e3364ede0a59a8...",
  "issuer": "costpilot-v1"
}
```

---

## Known Windows-Specific Considerations

### Path Separators
- Code uses `std::path::Path` (automatically handles Windows backslashes)
- No changes needed

### Line Endings
- Git autocrlf may convert LF → CRLF
- Rust source handles both
- No build impact

### File Permissions
- Windows uses ACLs (not Unix permissions)
- License file security handled by NTFS permissions
- No changes needed

---

## Build Artifacts

### Expected Files

After successful build:
```
target\
  x86_64-pc-windows-msvc\
    release\
      costpilot.exe          ← Main binary (~9-12 MB)
      costpilot.pdb          ← Debug symbols (optional)
```

### Distribution Package

**Minimum**:
- `costpilot.exe` (required)

**Optional**:
- `costpilot.pdb` (debug symbols for crash reports)
- `README.md` (user documentation)
- `LICENSE` (Apache 2.0)
- `CHANGELOG.md` (version history)

---

## Testing on Windows

### Basic Functionality Test

```powershell
# 1. No license (Free edition)
.\costpilot.exe scan --dry-run examples\basic\simple.yml

# 2. With valid license (Premium edition)
# Place license.json in %USERPROFILE%\.costpilot\
.\costpilot.exe scan --dry-run examples\basic\simple.yml

# 3. Version check
.\costpilot.exe --version
# Expected: costpilot 1.0.0

# 4. Help output
.\costpilot.exe --help
# Should display CLI help
```

### License Validation Test

```powershell
# Test expired license (should fallback to Free)
# Modify license.json: "expires": "2020-01-01T00:00:00Z"
.\costpilot.exe scan --dry-run examples\basic\simple.yml
# Expected: Runs in Free edition, no errors

# Test invalid signature (should fallback to Free)
# Modify license.json: "signature": "invalid"
.\costpilot.exe scan --dry-run examples\basic\simple.yml
# Expected: Runs in Free edition, no errors
```

---

## No Further Linux Work Required

**All non-Windows tasks complete**:
1. ✅ Security sweep (SECURITY_SWEEP_REPORT.md)
2. ✅ License E2E proof (LICENSE_E2E_PROOF.md)
3. ✅ Contract freeze (CONTRACT_FREEZE_CONFIRMATION.md)
4. ✅ CI cost safety (CI_COST_STATUS.md)
5. ✅ Windows handoff (this document)

**Linux work**: DONE
**Windows work**: Build `.exe` from commit 02c35f75
**After Windows build**: LAUNCH READY

---

## Troubleshooting

### Build Error: "linker `link.exe` not found"

**Solution**: Install Visual Studio Build Tools with "Desktop development with C++"

**Verification**:
```powershell
where link.exe
# Should output: C:\Program Files\...\link.exe
```

---

### Build Error: "could not find target x86_64-pc-windows-msvc"

**Solution**:
```powershell
rustup target add x86_64-pc-windows-msvc
cargo build --release --target x86_64-pc-windows-msvc
```

---

### Build Warning: "fingerprint mismatch"

**Solution**: Clean build
```powershell
cargo clean
cargo build --release --target x86_64-pc-windows-msvc
```

---

### Runtime Error: "VCRUNTIME140.dll not found"

**Solution**: Install Visual C++ Redistributable
- Download: https://aka.ms/vs/17/release/vc_redist.x64.exe
- Or distribute with binary

---

## Distribution Checklist

- [ ] Build `costpilot.exe` from commit 02c35f75
- [ ] Verify `costpilot.exe --version` outputs 1.0.0
- [ ] Test Free edition (no license)
- [ ] Test Premium edition (with valid license)
- [ ] Test expired license (fallback to Free)
- [ ] Package binary (+ LICENSE, README.md)
- [ ] Sign binary (optional, recommended for Windows SmartScreen)
- [ ] Create installer (optional, MSI or NSIS)
- [ ] Publish to GitHub Releases

---

## Signing Recommendations (Optional)

**Windows SmartScreen**: Unsigned binaries may trigger warnings

**Options**:
1. **Code Signing Certificate**: Purchase from DigiCert, Sectigo, etc.
2. **GitHub Actions**: Sign via workflow (requires cert in secrets)
3. **Self-Signed**: For internal use only (not recommended for public)

**Without signing**: Users may see "Windows protected your PC" warning (can be bypassed with "More info" → "Run anyway")

---

## Contact for Build Issues

If Windows build fails:
1. Verify commit: `git log --oneline -1` (should be 02c35f75)
2. Verify Rust version: `rustc --version` (should be 1.91.1+)
3. Verify MSVC tools: `cl.exe` (should output version)
4. Try clean build: `cargo clean && cargo build --release`

**All Linux-side validation already done** - build should succeed without changes.

---

## Conclusion

**Windows build ready**:
- ✅ Commit identified: 02c35f75
- ✅ Dependencies vendored
- ✅ Build command documented
- ✅ License system proven working (47 tests passing)
- ✅ Contract frozen (cannot be broken accidentally)
- ✅ CI safe ($0/month cost)

**Action**: Build `costpilot.exe` using above instructions.

**After Windows build**: ALL GTM preparation complete.

---

**Build Platform**: Windows 10+ with MSVC
**Target**: x86_64-pc-windows-msvc
**Expected Size**: ~9-12 MB
**Expected Time**: 5-10 minutes
