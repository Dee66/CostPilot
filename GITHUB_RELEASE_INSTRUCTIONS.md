# GitHub Release Instructions for v1.0.0

## 📋 Summary

**Status:** Ready for Linux x86_64 release
**Artifacts:** 2 archives + checksums (3.9 MB each)
**Tag:** v1.0.0 (created locally, not yet pushed)
**Tests:** 1826/1826 passing

## 🚀 Step-by-Step Release Process

#### Tag Provenance Verification

Before proceeding, verify that tag v1.0.0 points to a commit already on branch 'main' and the working tree is clean:

```bash
git show v1.0.0 --no-patch
git branch --show-current
git status
```

The tag must point to a commit on 'main', and `git status` must show a clean working tree.

### Step 1: Push the Tag to GitHub

```bash
cd /home/dee/workspace/AI/GuardSuite/CostPilot
git push origin v1.0.0
```

This makes the tag available on GitHub for creating the release.

### Step 2: Create GitHub Release

1. **Navigate to GitHub:**
   ```
   https://github.com/Dee66/CostPilot/releases/new
   ```

2. **Configure Release:**
   - **Tag:** Select `v1.0.0` from dropdown (or type if not visible)
   - **Release title:** `CostPilot v1.0.0 - Production Release`
   - **Description:** Copy entire content from `RELEASE_NOTES_v1.0.0.md`

3. **Upload Artifacts:**
   Click "Attach binaries" and upload these files from `dist/`:
   - `costpilot-1.0.0-linux-amd64.tar.gz`
   - `costpilot-1.0.0-linux-amd64.zip`
   - `sha256sum.txt`

4. **Final Checks:**
   - ✓ Check "Set as the latest release"
   - ✓ Leave "Create a discussion" unchecked (or check if you want)
   - ✓ Don't check "This is a pre-release"

5. **Publish:**
   Click green "Publish release" button

## 📦 Available Artifacts

```
dist/
├── costpilot-1.0.0-linux-amd64.tar.gz  (3.9 MB)
├── costpilot-1.0.0-linux-amd64.zip     (3.9 MB)
└── sha256sum.txt                        (checksums)
```

**Checksums (SHA256):**
```
bc1459220a856abcd33d179af780bc5712d770f6cd538c90526c644f620135c0  costpilot-1.0.0-linux-amd64.tar.gz
e4aa6cc969a15af5be8aba4b0928b4a18361a94fd8e4183579ad3b3d69fb8b14  costpilot-1.0.0-linux-amd64.zip
```

**Artifact Immutability Rule:** Contents of `dist/` must not change after checksums are generated. Artifacts must not be rebuilt between tagging and upload.

```bash
ls -lh dist/
sha256sum dist/*
# Verify no changes to existing files
```

## 🔍 Verification After Release

1. **Check release page:**
   ```
   https://github.com/Dee66/CostPilot/releases/tag/v1.0.0
   ```

2. **Test download link:**
   ```bash
   wget https://github.com/Dee66/CostPilot/releases/download/v1.0.0/costpilot-1.0.0-linux-amd64.tar.gz
   ```

3. **Verify checksum:**
   ```bash
   sha256sum costpilot-1.0.0-linux-amd64.tar.gz
   # Should match: bc1459220a856abcd33d179af780bc5712d770f6cd538c90526c644f620135c0
   ```

4. **Test installation:**
   ```bash
   tar -xzf costpilot-1.0.0-linux-amd64.tar.gz
   cd costpilot-1.0.0-linux-amd64/bin
   ./costpilot --version
   # Should output: costpilot 1.0.0
   ```

## 🪟 Windows Release

#### Windows Prerequisites

Required tools and setup before building:
- Visual Studio with MSVC toolchain (C++ build tools)
- rustup target x86_64-pc-windows-msvc
- Git Bash or equivalent for bash scripts

To add Windows binaries after rebooting to Windows:

1. **Build on Windows:**
   ```powershell
   cd C:\path\to\CostPilot
   git pull origin v1.0.0
   cargo build --release --target x86_64-pc-windows-msvc
   ```

2. **Package:**
   ```powershell
   $env:COSTPILOT_VERSION="1.0.0"
   $env:TARGET="windows-amd64"
   $env:OUT_DIR="dist"
   bash scripts/make_release_bundle.sh
   ```

   **STOP CONDITION:** If `make_release_bundle.sh` does not run successfully on Windows, stop the process. Do not improvise alternative build or packaging approaches.

3. **Upload to existing release:**
   - Go to release page
   - Click "Edit release"
   - Upload new artifacts:
     - `costpilot-1.0.0-windows-amd64.zip`
     - Update `sha256sum.txt` with new checksums
   - Save changes

   **CHECKSUM HANDLING:** After upload, generate SHA256 checksum for the Windows artifact using PowerShell:

   ```powershell
   Get-FileHash -Algorithm SHA256 costpilot-1.0.0-windows-amd64.zip | Format-List
   ```

   Add the checksum to the release notes under a "Checksums" section.

## 🍎 macOS Release

**SCOPE:** macOS releases are out of scope for this release cycle. macOS binaries will be added in a future release when macOS development environment is available.

## 📊 Project Status

### ✅ Completed
- Git tag v1.0.0 created
- Linux x86_64 artifacts built
- Release notes prepared
- Project root cleaned up
- Test suite passing (1826/1826)
- License system operational

### 🔜 Future Platforms
- Windows x86_64 (requires Windows host)
- Linux ARM64 (requires cross-toolchain or ARM host)
- macOS x86_64 & ARM64 (requires macOS host)

### 📁 Project Structure (Post-Cleanup)
- **Test data:** Moved to `.archive/test-data/`
- **Master keys:** Moved to `.archive/keys/` (gitignored)
- **Planning docs:** Moved to `docs/planning/`
- **Artifacts:** In `dist/`
- **Root:** Clean, professional structure

## 🎯 Next Actions

1. Push tag: `git push origin v1.0.0`
2. Create GitHub release (follow steps above)
3. Upload 3 files from `dist/`
4. Publish release
5. Test download and installation
6. (Optional) Reboot to Windows for Windows binaries
7. (Optional) Commit and push cleanup changes

## 💡 Tips

- **Don't forget** to copy the full content from `RELEASE_NOTES_v1.0.0.md`
- **Test the download** after publishing to ensure links work
- **Keep sha256sum.txt** with artifacts for user verification
- **Update checksums** if you rebuild or add more platforms
- **CLEANUP SAFETY:** Do not delete the v1.0.0 tag or release. These are immutable release artifacts. Only add new platforms to existing releases.

---

**Ready to release!** All Linux work complete. Windows binaries can be added later.
