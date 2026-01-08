# Scripts Directory Organization

## Core Categories

### Build & Compilation
- `build_wasm.sh` - Compile ProEngine to WebAssembly
- `rebuild_with_master_key.sh` - Build with cryptographic keys from env vars

### Release Management
- `release.sh` - Main release orchestrator
- `ci_release_steps.sh` - CI-specific release steps
- `make_release_bundle.sh` - Package release artifacts
- `sign_release.sh` - Cryptographically sign release bundles
- `verify_release_bundle.sh` - Validate signed release integrity
- `generate_changelog.sh` - Auto-generate CHANGELOG from commits

### Platform Packaging
- `create_debian_package.sh` - Build .deb for Debian/Ubuntu
- `create_rpm_package.sh` - Build .rpm for RHEL/Fedora
- `create_macos_installer.sh` - Build macOS .pkg
- `create_windows_installer.sh` - Build Windows .msi

### Licensing
- `issue_license.py` - Generate signed Pro/Enterprise licenses
- `test_license_flow.sh` - Verify license issuance and validation

### Testing & Validation
- `test_all.sh` - Run full test suite
- `test_installations.sh` - Validate package installations
- `validate_wasm.sh` - Verify WASM module integrity
- `pre_push_checks.sh` - Pre-commit validation
- `static_analysis.sh` - Lint and static analysis
- `run_security_scanning.sh` - Security vulnerability scanning

### Mental Model System
- `detect_mental_model_contradictions.py` - Find conflicts in docs/mental_model.md
- `detect_mental_model_conflicts.py` - Alias for contradictions detector
- `discover_mental_model.py` - Extract mental model from codebase
- `apply_mental_model_deltas.py` - Apply approved mental model changes

### Configuration & Utilities
- `manage_config.sh` - Configuration file management
- `extract_pubkey_hex.py` - Extract public key fingerprints
- `generate_test.py` - Test case generator
- `fix_python_tests.py` - Repair Python test infrastructure
- `update_progress.py` - Update progress tracking
- `print_target.sh` - Display build target info
- `error_reporting.sh` - Error diagnostic tooling
- `ai_failure_analysis.sh` - Analyze AI agent failures
- `peer_review_requirements.sh` - Enforce review policies

## Subdirectories

### signing/
Cryptographic signing infrastructure:
- PEM signing scripts
- GPG helpers
- Checksum verification
- Release policy validation

**Note**: Master keys stored externally (see `/tmp/costpilot_master_keys/`)

### test-data/
Test fixtures and ephemeral test keys (NOT production keys)

### tests/
Shell-based integration tests

### packaging_tools/
Platform-specific packaging helpers

### provenance/
SLSA provenance generation for supply chain security

### sbom/
Software Bill of Materials generation

### core_release/
Release-specific tooling

### cleanup/
Temporary build artifact cleanup (mostly empty after cleanup)

## Archived

### .archive/
Obsolete scripts replaced by new infrastructure:
- Old key management (pre-2026-01-08 rotation)
- Legacy key generation scripts
- `public.key.OLD` - Revoked public key

## Security Notes

1. **No private keys in git** - All private key material stored externally
2. **Test keys only in test-data/** - Never use for production
3. **Build requires env vars** - `COSTPILOT_LICENSE_PUBKEY`, `COSTPILOT_WASM_PUBKEY`
4. **Cryptographic rotation** - See `/tmp/costpilot_master_keys/README.md`

## Usage Examples

```bash
# Build with new keys
export COSTPILOT_LICENSE_PUBKEY=db52fc95fe7ccbd5e55ecfd357d8271d1b2d4a9f608e68db3e7f869d54dba5df
export COSTPILOT_WASM_PUBKEY=8db250f6bf7cdf016fcc1564b2309897a701c4e4fa1946ca0eb9084f1c557994
./scripts/rebuild_with_master_key.sh

# Issue license
python3 scripts/issue_license.py \
  --licensee "Company Name" \
  --tier Pro \
  --expires 2026-12-31

# Full release
./scripts/release.sh v1.0.1

# Run all tests
./scripts/test_all.sh
```
