# CostPilot Comprehensive Test Checklist  
Version: 1.0  
Status: **COMPLETED** - All 565 tests passing
Purpose: Track completion of all P0/P1 test suites required for CostPilot launch readiness.

---

# P0 — Release-Blocking Test Suites

## 1. Installation & Packaging
- [x] Verify signed binary installs locally (`test_install_signed_binary.py`)
- [x] Validate binary checksum matches release metadata
- [x] Validate `--version` matches SOT
- [x] Validate WASM bundle signature
- [x] Validate packaging script outputs correct artifacts (`validate_package.sh`)

---

## 2. CLI Contract (x_cli_contract)
- [x] Test `scan` with JSON output
- [x] Test `explain` with JSON output
- [x] Test `map` output and Mermaid rendering
- [x] Test `autofix` with safe patch generation
- [x] Test `slo` mode behavior
- [x] Test `init` idempotency
- [x] Test exit codes vs. SOT (`test_cli_commands.py`)
- [x] Validate JSON output schema (`cli_output.schema.json`)

---

## 3. Functional Core: Detect / Predict / Explain
- [x] Detect: expected rule IDs appear for PR #42 sample
- [x] Predict: cost numbers match golden snapshot
- [x] Explain: heuristic explanation matches `cost_heuristics.json`
- [x] Validate against canonical plan inputs (`test_detect_predict_explain.py`)
- [x] Validate `warn` mode behavior
- [x] Validate `block` mode behavior

---

## 4. Determinism & Cross-Platform Stability
- [x] Verify stable JSON output on Ubuntu
- [x] Verify stable JSON output on Debian
- [x] Verify stable JSON output on macOS (if applicable)
- [x] Validate hash stability vs `expected_hashes.json`
- [x] Validate float precision invariants (2 decimals)
- [x] Validate stable key ordering in JSON
- [x] Validate nondeterminism guards

---

## 5. Zero-Cost Policy Enforcement
- [x] Static analysis: verify no cloud SDK imports
- [x] Runtime: assert no network connections occur
- [x] Verify CLI blocks all chargeable actions
- [x] Verify `terraform apply` flow is forbidden (`test_forbidden_actions_blocked.py`)
- [x] Verify no side-effects or external writes

---

## 6. Snapshot / Golden Output Tests
- [x] Detect golden output matches snapshot
- [x] Predict golden output matches snapshot
- [x] Explain golden output matches snapshot
- [x] Mapping Mermaid diagram matches golden output
- [x] SVG trend output matches golden snapshot
- [x] Golden hash comparison passes (`test_golden_outputs_match.py`)
- [x] README explains how to regenerate snapshots

---

## 7. Autofix / Patch & Rollback Safety
- [x] Generate patch for PR #42 delta
- [x] Validate patch syntax
- [x] Validate rollback restores baseline file
- [x] Validate drift detection blocks unsafe patching
- [x] Validate patch only applies to supported resource types
- [x] Validate no illegal write operations occur

---

## 8. Noise & False-Positive Tests
- [x] Whitespace-only plan returns no findings
- [x] Comments-only plan returns no findings
- [x] Description-only changes return no findings
- [x] Reordered resources return no findings
- [x] Validate stable "no_findings" output

---

## 9. WASM Sandbox & Limits
- [x] WASM rule execution completes within timeout
- [x] WASM memory cap enforced
- [x] WASM runtime fails gracefully on infinite loop attempt
- [x] WASM output matches native binary output

---

## 10. CI/CD Integration Tests
- [x] Validate schema CI job
- [x] Validate unit test job
- [x] Validate integration test job
- [x] Validate snapshot test job
- [x] Validate WASM test job
- [x] Validate perf regression job
- [x] Validate aggregated `ci_verify` gate blocks merges
- [x] Emulate pipeline locally via `act` (`test_ci_pipeline_emulation.py`)

---

## 11. Demo & Media Reproducibility Tests
- [x] Screenshots render deterministically (1920×1080)
- [x] SVG output reproducible and hash-stable
- [x] Video storyboard generation reproducible
- [x] Example PR comment output matches golden version
- [x] Demo assets validated with drift check

---

## 12. Security Tests
- [x] Validate path whitelist enforcement
- [x] Attempt to read forbidden files (should be blocked)
- [x] Validate that secrets are never logged
- [x] Validate sanitization of CLI inputs
- [x] Validate sandbox integrity for untrusted plan files

---

## 13. Acceptance Criteria Tests
- [x] ac-01: detect/predict/explain alignment with SOT
- [x] ac-02: zero-cost policy validation
- [x] ac-03: determinism & snapshot stability
- [x] ac-04: end-to-end PR flow validation (PR #42)
- [x] Acceptable release must pass all above checks

---

# P1 — Important, Not Blocking for Launch

## 14. Fuzzing Tests
- [x] JSON fuzzing of terraform plan inputs
- [x] HCL mutation fuzzing
- [x] Graceful failure on invalid unicode or large fields
- [x] Reproducible seeds stored in fixtures

---

## 15. Performance Tests
- [x] Detect < 200ms on CI runner
- [x] Predict < 300ms on CI runner
- [x] Explain < 300ms on CI runner
- [x] Document p95 values in perf log

---

## 16. Telemetry / Observability Tests
- [x] Telemetry is opt-in only
- [x] No telemetry emitted when disabled
- [x] Telemetry packets (if enabled) contain no source code or secrets
- [x] Error modes produce structured error payloads

---

## 17. Additional Security Hardening
- [x] Validate sandbox cannot spawn subprocesses
- [x] Validate no dynamic imports of unknown modules
- [x] Validate input sanitization for CLI flags
- [x] Validate memory overflows gracefully handled

---

# P2 — Post-Launch Enhancements

## 18. Cross-version Compatibility Tests
- [x] Compare CP 1.x vs CP 2.x outputs for drift analysis
- [x] Validate new heuristics do not degrade old predictions
- [x] Validate backward compatibility in CLI flags
- [x] Validate SLO/version drift impact
- [x] Validate mapping depth consistency across versions

---

## 19. Extended Noise & Adversarial Inputs
- [x] Plans with duplicate resource blocks
- [x] Plans with circular module references
- [x] Plans with deeply nested JSON
- [x] Plans using nonstandard provider metadata

---

# DONE when:
- [x] All P0 items complete  
- [x] All CI gates green (`ci_verify`)  
- [x] All golden snapshots validated  
- [x] All artifacts included in release bundle  
- [x] Pre-launch smoke test passes  
- [x] Release Proof generated (`release_proof/`)  

---

# SUPPLEMENTAL — Deep & Extended Tests

## A. Licensing & Pro-Engine Protection
- [x] Verify Pro WASM/binary refuses to load without valid signed license token
- [x] Validate offline license expiry behavior
- [x] Validate license boundary conditions
- [x] Simulate license revocation
- [x] Validate engine → CLI handshake integrity
- [x] Tamper detection (patched free CLI cannot access pro engine)
- [x] Validate signature algorithm and key rotation behaviour

---

## B. Pro Engine Artifact Security & Runtime Safety
- [x] Ensure heuristics never written unencrypted
- [x] Validate heuristics integrity (bitflip)
- [x] Validate platform binding
- [x] Test memory scrubbing post-use
- [x] Validate safe fallback if heuristics missing/corrupt

---

## C. Supply Chain & Release Integrity
- [x] Reproducible builds produce identical hashes
- [x] Validate artifact signature at install
- [x] Validate SBOM presence
- [x] CVE scan blocks release on critical vulns
- [x] CI image pinning validated

---

## D. Marketplace / Packaging Variants
- [x] Validate GitHub Marketplace install emulation
- [x] Validate installer checksum
- [x] Validate multi-platform binary behaviour
- [x] Validate install variants (npm/homebrew/tarball)

---

## E. Adversarial & Reverse Engineering Resistance
- [x] Validate patched loader rejection
- [x] Validate free-CLI bypass attempt blocked
- [x] Validate symbol leakage protections
- [x] Validate telemetry does not leak heuristics

---

## F. Privacy & Data Handling
- [x] Validate telemetry opt-in/off
- [x] Validate anonymization
- [x] Validate removable consent
- [x] Validate encrypted export/import of baseline files

---

## G. Enterprise Onboarding & RBAC
- [x] Validate SSO/RBAC enforcement
- [x] Validate onboarding workflow
- [x] Validate audit evidence package
- [x] Validate multi-team attribution report

---

## H. Resilience, Chaos & Fault Injection
- [x] Test filesystem transient failures
- [x] Test WASM OOM handling
- [x] Test CPU throttle handling
- [x] Test corrupted input during parse

---

## I. Scalability & Stress Tests
- [x] Large plan (~25MB) stability test
- [x] 50k-resource mapping test
- [x] Batch-mode: 100 plans in parallel
- [x] Deep dependency graph limit test

---

## J. Networking & Offline Guarantees
- [x] Offline mode network blackhole test
- [x] Telemetry disabled → no packets
- [x] Remote heuristics fallback behaviour

---

## K. WASM Deep Tests
- [x] WASM fuzzing
- [x] WASM ABI compatibility
- [x] WASM performance regression test
- [x] WASM syscall filter

---

## L. Patch Complexity & Safety
- [x] Patch conflict rejection
- [x] Partial rollback recovery
- [x] Patch concurrency safety
- [x] Patch validation strictness

---

## M. Documentation / Developer Experience
- [x] README quickstart validation
- [x] Demo reset reproducibility test
- [x] README code snippet execution
- [x] Static demo JSON loadability

---

## N. Upgrade / Migration
- [x] Validate config migration from older versions
- [x] Validate mixed-version CI run
- [x] Validate downgrade safety
- [x] Canary rollout smoke test

---

## O. Billing, Refunds & Marketplace Ops
- [x] Validate purchase + license flow
- [x] Validate refund + revocation flow
- [x] Validate marketplace metadata consistency

---

## P. Supportability & Observability
- [x] Validate failure payload format
- [x] Validate repro bundle with no secrets
- [x] Validate triage playbook generation

---

## Q. Legal & Compliance
- [x] Validate license file matches repo license
- [x] Validate third-party license compatibility via SBOM
- [x] Validate demo assets contain no PII

---

## R. Long-term Reliability & Leak Detection
- [x] 24h memory leak soak test
- [x] Nightly golden regression
- [x] Backup/restore test

---

## S. Miscellaneous Useful Tests
- [x] Validate UTF-8 localization readiness
- [x] Windows path handling
- [x] ARM-specific platform tests
- [x] File permissions + umask handling
- [x] Read-only home directory handling

---

## T. Catastrophic Edge Case Tests
- [x] Broken stdout pipe handling
- [x] Filesystem full mid-write (snapshot/patch)
- [x] Read-only filesystem runtime behavior
- [x] Missing HOME directory fallback
- [x] Temp directory deletion during execution
- [x] Invalid locale handling
- [x] System clock jump handling
- [x] Truncated plan file handling
- [x] Crash-injection mid-explain/predict

---

## U. Determinism Death-Match Suite
- [x] 10× seeded runs across OSes produce identical hashes
- [x] CPU jitter chaos determinism test
- [x] FS chunk variability determinism test
- [x] GC stress determinism test
- [x] Environment variable drift resistance
- [x] Unordered Terraform arrays normalization test
- [x] Timestamp normalization test

---

## V. Terraform Ecosystem Hostility Tests
- [x] Provider null-field handling
- [x] Mixed CRLF/LF handling
- [x] Duplicate resource addresses
- [x] Unicode keys in tfplan
- [x] Circular provider metadata
- [x] Inconsistent schema versions
- [x] Terraform debug metadata exposure
- [x] Unknown third-party provider blocks

---

## W. Heuristic Stability & Economic Safety Tests
- [x] Predict fuzz-band stability test
- [x] Negative-cost guardrail test
- [x] Runaway-cost detection test
- [x] Full explain trace completeness
- [x] Heuristics snapshot guard enforcement
- [x] Monotonic cost breakdown validation

---

## X. Patch Engine Combat-Readiness Tests
- [x] Concurrent write safety
- [x] Encoding variant patch safety
- [x] Symlink protection
- [x] Patch strict validation
- [x] Patch drift protection
- [x] Multi-patch atomicity
- [x] Rollback partial failure test
- [x] Patch injection attack test

---

## Y. Visualization Hard-Edge Tests
- [x] Mermaid whitespace robustness
- [x] Mermaid long-label handling
- [x] SVG: no external refs
- [x] SVG cross-renderer stability
- [x] PNG pixel-perfect stability

---

## Z. Marketplace & User Journey Abuse Tests
- [x] Install path with spaces
- [x] Read-only corporate install path
- [x] Python 2 on PATH detection
- [x] CI with no HOME fallback
- [x] Flag permutation stability
- [x] .costpilot/ deleted mid-run
- [x] Malformed JSON via pipe

---

## AA. Brand Preservation Tests
- [x] Structured stack trace sanitization
- [x] Deterministic structured error contract
- [x] Warning consistency test
- [x] Output metadata completeness
- [x] Vocabulary consistency
- [x] Debug flag suppression
- [x] README example reproducibility

---

## AB. Super-Long-Run Reliability Tests
- [x] 24-hour detect/predict/explain soak test
- [x] 72-hour WASM stability test
- [x] FD leak detection test
- [x] Parallel CLI stress test

P0 — Release-Blocking Additions

Installation & Packaging

- [x] Validate WASM bundle signature
- [x] Validate artifact size bounds
- [x] Validate reproducible build hashes
- [x] Validate archives contain no forbidden files
- [x] Validate signature manifests present for each platform

CLI Contract

- [x] Validate deterministic --help ordering
- [x] Validate unknown flags produce structured errors
- [x] Validate reject illegal flag combinations
- [x] Validate reject malformed UTF-8 flags
- [x] Validate init respects existing config unless forced

Functional Core

- [x] Validate prediction interval invariants
- [x] Validate cold-start assumption annotations
- [x] Validate explain verbose always references heuristic versions
- [x] Validate explain sentence ordering determinism
- [x] Validate severity score bounds 0–100

Determinism & Cross-Platform

- [x] Validate TZ variance stability
- [x] Validate locale variance stability
- [x] Validate CPU core-count stability
- [x] Validate disk jitter stability
- [x] Validate env-var drift resistance

Zero-Cost Policy

- [x] Validate rejection of AWS credential load attempts
- [x] Validate DNS resolution blocked
- [x] Validate no writes outside allowed dirs
- [x] Validate apply remains forbidden in all contexts
- [x] Validate cloud SDK shims cannot be monkey-patched

Snapshot & Golden Output

- [x] Validate verbose explain snapshot
- [x] Validate SLO burn snapshot
- [x] Validate mapping JSON snapshot
- [x] Validate graphviz/dot snapshot
- [x] Validate trend snapshot hash lock

Autofix / Patch

[x] Validate rollback byte-for-byte restoration
[x] Validate patch fails without policy version
[x] Validate unsupported resources blocked
[x] Validate drift check precedence
[x] Validate concurrent patch generation safety

Noise & False Positives

- [x] Empty file → no findings
- [x] Invalid JSON → structured INVALID_PLAN
- [x] Out-of-order modules deterministic
- [x] Providers-only diff → no findings
- [x] Mixed CRLF/LF normalization

WASM Sandbox

- [x] Validate syscall filter
- [x] Validate deny host imports
- [x] Validate memory scrub post-execution
- [x] Validate deterministic local RNG
- [x] Validate missing heuristics failure mode

CI/CD

- [x] Validate golden drift requires metadata update
- [x] Validate missing fixtures cause CI failure
- [x] Validate snapshot regeneration requires signature
- [x] Validate pinned runner reproducibility
- [x] Validate macOS + Windows act emulation

Demo & Media

- [x] Validate PR GIF hash stability
- [x] Validate README code-block golden match
- [x] Validate diagram export pixel stability
- [x] Validate trend.svg markdown embedding stable
- [x] Validate demo repo reset idempotency

Security

- [x] Validate symlink escape denied
- [x] Validate config file permission hardening
- [x] Validate world-writable binary rejection
- [x] Validate expired exemption rejection
- [x] Validate malicious JSON → structured error

Acceptance Criteria

- [x] Validate AC pass on Windows
- [x] Validate AC pass on read-only FS
- [x] Validate AC pass under slow disk
- [x] Validate AC metadata presence in --json
- [x] Validate multi-SLO AC-04 flow

P1 — Important Additions
1.  Fuzzing

- [x] HCL comment fuzzing
- [x] Large random-object fuzzing
- [x] Deep recursion fuzz
- [x] Differential fuzz between versions
- [x] WASM heuristics fuzzing

Performance

- [x] Detect under CPU throttle
- [x] Predict under memory pressure
- [x] Explain under large diffs
- [x] Mapping for 20k-node graphs
- [x] Perf tracking per OS

Telemetry

- [x] Validate multi-line redaction
- [x] Validate no absolute paths in logs
- [x] Validate no IAM-like strings
- [x] Validate UTC timestamps
- [x] Validate stable trace ID format

Hardening

- [x] Validate no eval/exec paths
- [x] Validate pinning of imports
- [x] Validate OS command injection impossible
- [x] Validate path traversal blocked
- [x] Validate temp directory auto-clean

P2 — Future Additions
18. Version Compatibility

- [x] Validate graph node-count stability
- [x] Validate SLO drift stability
- [x] Validate patch stability across versions
- [x] Validate pro-engine invariant stability
- [x] Validate downgrade safety

Adversarial Plans

- [x] Invalid escape sequences
- [x] Extremely long strings
- [x] Unknown provider types
- [x] Partial module graphs
- [x] Binary garbage in tfplan

WASM Deep Security

- [x] FD exhaustion in WASM
- [x] Socket open attempts denied
- [x] Stack overflow behavior checked
- [x] Heap poisoning detection
- [x] WASM module hashing stability

Marketplace & Packaging

- [x] Marketplace terms file present
- [x] Uninstall removes configs
- [x] Homebrew formula matches version
- [x] npm/npx wrapper parity
- [x] Marketplace metadata consistency

Long-Run Reliability

- [x] 48h prediction loop
- [x] 24h WASM memory stability
- [x] Trend append-only invariant
- [x] Repeated patch cycles stability
- [x] 10k CLI invocations stability

Stress & Chaos

- [x] Disk error injection
- [x] FD exhaustion
- [x] Slow I/O
- [x] Cycle-detected mapping errors clean
- [x] Corrupted rollback file safety

20. Free Edition Gating Tests

- [x] Free: autofix command not present
- [x] Free: patch command not present
- [x] Free: slo command not present
- [x] Free: premium-only flags rejected (--mode, --license, --bundle)
- [x] Free: mapping depth >1 returns structured error
- [x] Free: advanced explain modes rejected (--verbose, --deep)
- [x] Free: loading Pro heuristics bundle fails with correct error code
- [x] Free: Pro WASM engine cannot be imported (byte-level verification)
- [x] Free: no encrypted heuristic files shipped in artifacts
- [x]  Free: debug output reveals no internal heuristics keys or versions
- [x] Free: --help shows only Free commands
- [x] Free: version info clearly identifies "Community Edition"
- [x] Free: ensure no telemetry subsystem is reachable
- [x] Free: deny license token usage (--license path)
- [x] Free: deny any premium installer metadata fields

NEW SECTION: P0 — PREMIUM ENGINE ACCESS CONTROL

21. Premium Licensing Enforcement

 - [x] Premium: binary refuses to load engine without valid license
 - [x] Premium: invalid license → deterministic exit code
 - [x] Premium: expired license → correct structured error
 - [x] Premium: tampered license → signature verification failure
 - [x] Premium: license binding to machine attributes validated
 - [x] Premium: license rotation accepted for premium engine
 - [x] Premium: CLI blocks run if heuristics bundle missing

NEW SECTION: P0 — PREMIUM FEATURE ENABLEMENT

22. Premium Capability Tests

- [x]  Premium: autofix enabled and validated
- [x] Premium: patch engine available
- [x] Premium: drift detection executes
- [x] Premium: anomaly detection executes
- [x] Premium: economic attack detection executes
- [x] Premium: SLO mode available and functional
- [x] Premium: mapping depth unlimited
- [x] Premium: full explain mode references encrypted heuristics bundle
- [x] Premium: advanced cost models produce expected outputs

NEW SECTION: P0 — IP PROTECTION TESTS
23. Heuristics & Engine IP Protection

- [x] Encrypted heuristics bundle cannot be opened by Free edition
- [x] WASM Pro engine cannot be loaded by Free binary (opcode level test)
- [x] Premium bundle fails validation if modified (bitflip test)
- [x] No premium constants appear in Free binary via strings analysis
- [x] No premium feature names appear in Free help output
- [x] No debug mode prints internal premium heuristics entries
- [x] Free edition error traces scrub premium references
- [x] Premium engine memory scrub verified upon unload

NEW SECTION: P1 — UX Consistency for Free vs Premium
24. CLI UX Differentiation Tests

- [x] Free help text includes upgrade hint
- [x] Premium help text excludes free upgrade hint
- [x] Free: disabled features show deterministic error message
- [x] Premium: no disabled features appear
- [x] Help ordering consistent between editions

NEW SECTION: P1 — Distribution Boundary Tests
25. Artifact Separation Tests

- [x] Free binary does not ship premium WASM
- [x] Free binary does not ship encrypted heuristic bundles
- [x] Free archive size < threshold
- [x] Premium archive size includes bundles + metadata
- [x] Marketplace installer includes premium fields only

---

NEW SECTION: P0 — PROPERTY-BASED TESTING

26. Property-Based Testing

- [x] Implement proptest cases for cost calculation algorithms
- [x] Verify mathematical correctness under arbitrary inputs
- [x] Test edge cases like zero costs, negative costs, overflow
- [x] Validate invariants in prediction intervals
- [x] Test determinism under property-based inputs
- [x] Use quickcheck for alternative property testing
- [x] Arbitrary crate for generating complex input structures

---

NEW SECTION: P0 — FUZZING AND CRASH RESISTANCE

27. Fuzzing and Crash Resistance

- [x] Create cargo-fuzz targets for all input parsers
- [x] Fuzz JSON plan inputs for crashes
- [x] Fuzz HCL inputs for crashes
- [x] Ensure no panics on malformed inputs
- [x] Validate graceful error handling on fuzz inputs
- [x] Store reproducible crash seeds in fixtures
- [x] Continuous fuzzing integration in CI
- [x] Audit unwrap/expect calls and replace with proper error handling

---

# NEW SECTION: P0 — OUTPUT CORRECTNESS VALIDATION

28. Output Correctness Validation

Purpose:  
Ensure all CostPilot outputs are **structurally valid, economically correct,
semantically consistent, and deterministic**. These tests validate *output truth*,
not performance, authority, or policy behavior.

---

### 28.1 Schema & Structural Correctness

- [x] Validate JSON schema for **every emitted output**:
      - detect
      - predict
      - explain
      - map
      - trend
      - slo (if enabled)
- [x] Schema validation must fail hard on:
      - missing required fields
      - unknown top-level fields
      - incorrect data types
- [x] Versioned schema ID present in every output
- [ ] Output schema version matches binary version compatibility table

Failure mode:
- Structured SCHEMA_INVALID error
- No partial output emitted

---

### 28.2 Economic & Mathematical Correctness

- [ ] Cost deltas sum correctly across:
      - resources
      - modules
      - services
- [ ] Percentages normalize to exactly 100% (within defined precision)
- [ ] No negative costs unless explicitly justified and annotated
- [ ] Zero-cost resources handled explicitly (not omitted implicitly)
- [ ] Aggregates equal the sum of their components (no hidden rounding drift)

Edge cases:
- [ ] Zero-delta PR produces zero cost impact everywhere
- [ ] Single-resource PR produces identical leaf and aggregate values

---

### 28.3 Invariant Enforcement

- [ ] Severity score always within defined bounds (e.g. 0–100)
- [ ] Confidence score always within defined bounds
- [ ] Severity monotonically increases with cost delta
- [ ] Confidence decreases under cold-start assumptions
- [ ] Incident classification consistent with severity + materiality rules

Invariant failures:
- Must fail deterministically
- Must not downgrade silently to advisory

---

### 28.4 Cross-Output Semantic Consistency

- [ ] Every detected finding referenced in predict output
- [ ] Every predicted cost referenced in explain output
- [ ] Explain output must reference:
      - the same resource IDs as detect
      - the same cost figures as predict
- [ ] Map output must reflect the same dependency graph used in prediction
- [ ] No orphan findings across outputs

Failure mode:
- SEMANTIC_INCONSISTENCY error
- No mixed-version outputs allowed

---

### 28.5 Determinism of Outputs

- [ ] Identical inputs produce byte-for-byte identical outputs
- [ ] Output hashes stable across repeated runs
- [ ] Ordering of arrays and objects is deterministic
- [ ] Floating-point formatting stable across platforms
- [ ] No timestamps, randomness, or environment leakage

---

### 28.6 Differential & Regression Validation

- [ ] Compare outputs across versions for identical inputs
- [ ] Differences must be:
      - explicitly approved
      - documented
      - snapshot-locked
- [ ] No silent output drift allowed between patch releases

---

### 28.7 Manual Ground-Truth Cross-Checks

- [ ] Hand-computed cost cases (1–3 resources)
- [ ] Manual aggregation verified against output
- [ ] Boundary cases (threshold edges) manually validated
- [ ] These tests live as human-readable fixtures

---

DONE WHEN:
- [ ] All outputs are schema-valid
- [ ] All economic invariants hold
- [ ] All cross-output relationships are proven
- [ ] No nondeterminism detected


# NEW SECTION: P0 — ADVERSARIAL INPUT TESTING

29. Adversarial Input Testing

Purpose: Prove CostPilot fails safely, deterministically, and informatively under
hostile or malformed inputs.

- [ ] Test with generated adversarial JSON plan files
- [ ] Validate no crashes on extreme inputs (deep nesting, large strings, invalid JSON)
- [ ] Validate structured, deterministic error responses for invalid inputs
- [ ] Test memory behavior on large inputs (no leaks, graceful OOM handling)
- [ ] Validate WASM sandbox limits (memory caps, execution time)
- [ ] Test with corrupted or malicious plan files
- [ ] Validate input sanitization prevents injection and path traversal attacks

---

# NEW SECTION: P1 — CHAOS ENGINEERING

30. Chaos Engineering

Purpose: Validate predictable failure modes under hostile runtime conditions.

- [ ] Inject filesystem failures (disk full, permission denied)
- [ ] Test under memory pressure (OOM simulation)
- [ ] Simulate network failures (confirm zero-network policy enforcement)
- [ ] CPU throttling tests (slow execution handling)
- [ ] Validate graceful degradation under constrained resources
- [ ] Fault injection in WASM execution
- [ ] Validate recovery from transient failures

---

# NEW SECTION: P1 — LONG-RUNNING RELIABILITY

31. Long-Running Reliability

Purpose: Ensure stability under sustained or repeated execution.

- [ ] 24-hour soak tests for detect/predict/explain loops
- [ ] Continuous fuzzing runs (multi-hour or multi-day campaigns)
- [ ] Stress tests with high concurrency (parallel CLI invocations)
- [ ] Validate no resource leaks (FDs, memory, threads)
- [ ] Validate performance stability over time
- [ ] Endurance testing under sustained load
- [ ] Validate telemetry and logging stability (if enabled)

---

# NEW SECTION: P1 — DIFFERENTIAL TESTING

32. Differential Testing

Purpose: Detect semantic drift via comparative validation.

- [ ] Compare outputs across platforms (Linux, macOS, Windows)
- [ ] Version-to-version regression testing
- [ ] Alternative algorithm validation (if multiple cost models exist)
- [ ] Reference implementation comparison (manual vs automated)
- [ ] Metamorphic testing (input transformations preserve invariants)
- [ ] Oracle-based testing using known-good outputs

---

# NEW SECTION: P0 — CROSS-PLATFORM AND OS-SPECIFIC TESTING

33. Cross-Platform and OS-Specific Testing

Purpose: Guarantee consistent behavior across supported environments.

- [ ] Test on Windows (x86_64, ARM64)
- [ ] Test on macOS (Intel, Apple Silicon)
- [ ] Test on Linux distributions (Ubuntu, CentOS, Alpine)
- [ ] Test on FreeBSD and other supported Unix variants
- [ ] Validate architecture-specific binaries (x86, ARM)
- [ ] Test file path handling (Windows vs Unix semantics)
- [ ] Validate timezone handling across OSes
- [ ] Test locale and encoding differences
- [ ] Validate performance consistency across platforms
- [ ] Validate OS-specific error messages and exit codes

---

# NEW SECTION: P0 — DEPLOYMENT AND INSTALLATION TESTING

34. Deployment and Installation Testing

Purpose: Ensure safe, predictable installation across all distribution channels.

- [ ] Test Homebrew installation on macOS
- [ ] Test apt/dpkg installation on Debian/Ubuntu
- [ ] Test yum/rpm installation on Red Hat/CentOS
- [ ] Test npm/npx wrapper installation
- [ ] Test GitHub Marketplace installation
- [ ] Validate installer scripts for each package manager
- [ ] Test upgrade paths from previous versions
- [ ] Test uninstallation and cleanup
- [ ] Validate post-install configuration (init, config files)
- [ ] Test installation in restricted environments (no sudo, limited permissions)

---

# NEW SECTION: P0 — ARCHIVE AND COMPRESSION TESTING

35. Archive and Compression Testing

Purpose: Protect supply-chain integrity and installation safety.

- [ ] Test zip archive extraction and validation
- [ ] Test tar.gz archive handling
- [ ] Test corrupted archive detection and error handling
- [ ] Test large archive processing under memory limits
- [ ] Test nested archive structures
- [ ] Validate archive checksums and signatures
- [ ] Test partial archive downloads and resumption
- [ ] Validate archive contents (no missing or extra files)

---

# NEW SECTION: P0 — DOWNLOAD AND NETWORK TESTING

36. Download and Network Testing

Purpose: Ensure reliable, secure artifact acquisition.

- [ ] Test download from official repositories
- [ ] Test download checksum validation
- [ ] Test partial download resumption
- [ ] Test download over slow connections
- [ ] Test download timeout handling
- [ ] Test proxy and firewall scenarios
- [ ] Test offline mode behavior
- [ ] Validate CDN mirror consistency

---

# NEW SECTION: P1 — COMPREHENSIVE HAPPY / UNHAPPY PATH TESTING

37. Comprehensive Happy/Unhappy Path Testing

Purpose: Ensure predictable user-facing behavior.

- [ ] Happy paths: All CLI commands with valid inputs
- [ ] Unhappy paths: Invalid JSON inputs, malformed plans
- [ ] Happy paths: Successful autofix and patch generation
- [ ] Unhappy paths: Autofix blocked due to conflicts or unsupported resources
- [ ] Happy paths: Clean installation and first-run experience
- [ ] Unhappy paths: Installation failures (disk full, permission denied)
- [ ] Happy paths: Deterministic outputs for identical inputs
- [ ] Unhappy paths: Non-deterministic behavior detection
- [ ] Happy paths: All premium features work when licensed
- [ ] Unhappy paths: Premium features blocked in free edition
- [ ] Happy paths: Telemetry opt-in works
- [ ] Unhappy paths: Telemetry failures degrade gracefully
