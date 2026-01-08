# CostPilot Operational Reality Investigation Report

## Scope & Method

This report determines the true operational reality of CostPilot as it exists today, without modification. Investigation traces actual execution paths, build artifacts, and runtime behavior through direct code inspection and evidence collection. No assumptions made; all findings supported by file paths and line numbers.

## Runtime Execution Truth

### Runtime Execution Graph
- **Entry**: src/bin/costpilot.rs:571 (fn main() calls main_inner())
- **Edition Detection**: src/bin/costpilot.rs:592 (costpilot::edition::detect_edition() - loads ProEngine conditionally)
- **Command Dispatch**: src/bin/costpilot.rs:651-766 (match on Commands enum)
- **Executed Modules**:
  - costpilot::cli::* (all command modules called via execute() methods)
  - costpilot::engines::* (detection, prediction, explain, policy, etc.)
  - costpilot::edition::* (always loaded)
  - costpilot::pro_engine::* (conditionally loaded via detect_edition())
- **Never-Called Modules**: None identified - all CLI commands have handlers
- **Conditionally-Loaded Modules**:
  - ProEngine/WASM: Only if license file present and valid (src/edition/mod.rs:18-25)
  - WASM runtime: Only for ProEngine features (src/pro_engine/wasm_runtime.rs)

### Command Implementation Status
- **Real Logic**: Scan, Baseline, Diff, Init, Map, Policy, Audit, Heuristics, Explain, Validate, Version, SloBurn, SloCheck, Slo, Trend, Anomaly, AutofixDriftSafe
- **Stubs/Placeholders**: Escrow (src/bin/costpilot.rs:1590-1603), Usage (src/bin/costpilot.rs:1693-1725), Performance (src/bin/costpilot.rs:1604-1692), AutofixPatch (src/bin/costpilot.rs:1428-1449), AutofixSnippet (src/bin/costpilot.rs:1450-1471)

### License Issuer Binary
- **Entry**: src/bin/license_issuer.rs: (main function generates keypairs and licenses)
- **Execution**: Standalone binary for license management, not part of runtime CostPilot

## Build & Distribution Truth

### Compiled Files (Cargo.toml)
- **Binaries**: costpilot (src/bin/costpilot.rs), license-issuer (src/bin/license_issuer.rs)
- **Library**: costpilot (src/lib.rs) as cdylib and rlib
- **Conditional Compilation**: WASM targets exclude ring/wasmtime/hex; native includes crypto extensions

### Build Process (build.rs)
- **Compile-time Actions**: Generates Ed25519 keypairs for license/WASM signing (build.rs:6-7)
- **Embedded Keys**: LICENSE_PUBLIC_KEY, WASM_PUBLIC_KEY included in binary (build.rs:25-35)
- **No Runtime Network**: Key generation is offline, deterministic

### Shipped Files (scripts/make_release_bundle.sh)
- **Binary**: target/release/costpilot (only)
- **Documentation**: README.md, LICENSE
- **Examples**: examples/*.json (if present)
- **Metadata**: sbom.spdx.json (generated)
- **Not Shipped**: Source code, tests, docs/, scripts/, all other files

### CI/Build Artifacts (.github/workflows/*)
- **Release Workflows**: Build binaries for multiple platforms, create bundles via make_release_bundle.sh
- **No Source Shipping**: Workflows build and ship only compiled binaries + minimal docs

## Offline / Zero-Network Verification

### Network Capability Assessment
- **Runtime Network Calls**: NONE
  - No HTTP clients imported (reqwest, hyper, axum, warp, rocket not in use statements)
  - No TCP/UDP sockets (socket mentions only in security validator detection patterns)
  - No telemetry hooks (telemetry mentions only in test data/docs)
  - Background threads: One timeout thread in ProEngine WASM runtime (src/pro_engine/wasm_runtime.rs:150), conditional on ProEngine loading

- **Tests Only**:
  - socket imports in tests/network/test_offline_mode_blocking.py
  - telemetry test data in tests/network/test_telemetry_blocking.py
  - HTTPS URLs in test fixtures (tests/test_data/*)

- **Documentation Only**:
  - reqwest examples in docs/WASM_BUILD.md
  - HTTP client lists in docs/ZERO_NETWORK.md

### Zero-Network Enforcement
- **Runtime Checks**: ZeroCostGuard enforces no network before scan command (src/bin/costpilot.rs:658-667)
- **Dependency Validation**: ZeroNetworkValidator blocks forbidden deps (src/engines/policy/zero_network.rs)

## Licensing Reality

### Client-Side Only
- **License Storage**: Local JSON file (~/.costpilot/license.json) (src/edition/mod.rs:67)
- **Validation Logic**: Client-side Ed25519 signature verification (src/pro_engine/crypto.rs:35-65)
- **No Server Communication**: All licensing offline, no network calls

### Renewal Not Automatic
- **Expiry Check**: Client checks expiry date against system time (src/pro_engine/license.rs:85-89)
- **No Renewal Logic**: No code for automatic renewal or server contact

### Subscription Not Enforced
- **Fallback Guaranteed**: detect_edition() returns free context if ProEngine fails (src/edition/mod.rs:18-25)
- **No Blocking**: Premium features gated but free mode always available

## Claims vs Reality Table

| Claim | Source | Reality | Evidence |
|-------|--------|---------|----------|
| No runtime agents or background collectors | README.md:32 | TRUE | No agent/collector code in src/; one conditional timeout thread |
| No cloud credentials or IAM access required | README.md:33 | TRUE | No AWS SDK deps; no credential handling in src/ |
| No noisy or speculative blocking | README.md:35 | TRUE | Blocking only for configured policies/SLOs |
| WASM-safe architecture (no network, no AWS SDK) | docs/IMPLEMENTATION_STATUS.md:7 | TRUE | No network imports; AWS SDK not in deps |
| Operates strictly at PR boundary | README.md:12 | TRUE | CLI takes plan files as input, no live infra access |
| Deterministic, reproducible outputs | README.md:16 | TRUE | No randomness in core logic (keys pre-generated) |
| Advisory and blocking modes | README.md:17 | TRUE | Policy/SLO enforcement configurable |

## Artifact Classification

### RUNTIME_CRITICAL
- src/lib.rs (core library)
- src/bin/costpilot.rs (main CLI)
- src/engines/detection/ (cost analysis core)
- src/engines/prediction/ (cost prediction core)
- src/engines/explain/ (causality explanation)
- src/engines/shared/ (common models/types)
- src/cli/ (command implementations)
- src/edition/ (edition management)
- src/pro_engine/ (premium features)
- Cargo.toml (build configuration)
- build.rs (key generation)

### BUILD_ONLY
- scripts/make_release_bundle.sh (packaging)
- scripts/create_*_installer.sh (platform installers)
- scripts/sign_release.sh (code signing)
- .github/workflows/release.yml (release automation)

### TEST_ONLY
- tests/ (all test files)
- benches/ (benchmarks)
- tarpaulin.toml (coverage config)
- cleanup/ (build artifacts)

### DOC_ONLY
- docs/ (all documentation files)
- README.md (user documentation)
- CHANGELOG.md (version history)
- LICENSE (legal)

### PARKED_FUTURE
- src/bin/license_issuer.rs (future license management)
- scripts/update_costpilot.sh (future auto-update)
- scripts/error_reporting.sh (future error reporting)
- scripts/logging_monitoring.sh (future logging)
- scripts/manage_config.sh (future config management)

### UNKNOWN
- examples/ (unclear if runtime examples or docs)
- completions/ (shell completions - may be runtime optional)
- configs/ (configuration templates - may be runtime)

## Unknowns / Insufficient Evidence

### CLI --help Output
- INSUFFICIENT EVIDENCE: Terminal unavailable for runtime verification

### Performance of Unimplemented Features
- INSUFFICIENT EVIDENCE: Stub implementations return placeholder data

### Full WASM Execution Paths
- INSUFFICIENT EVIDENCE: ProEngine conditionally loaded, full trace requires license
