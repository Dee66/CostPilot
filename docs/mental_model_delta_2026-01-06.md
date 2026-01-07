# Mental Model Delta - 2026-01-06

## Source
Factual investigation conducted 2026-01-06 correcting launch readiness audit errors.

## Trigger
User identified 4 major factual errors in initial audit. Strict READ-ONLY investigation revealed architectural misunderstanding.

## Proposed Claims

### Cryptographic Architecture

CLAIM license_private_keys_not_in_binary = true
VERIFY = mechanical
EVIDENCE = build.rs:267 comment "Public keys only. Private keys never shipped."
EVIDENCE = strings target/release/costpilot | grep -i "private\|BEGIN.*KEY" returns empty
EVIDENCE = src/pro_engine/crypto.rs:186-195 uses embedded public keys for verification only
RATIONALE = Ed25519 keypairs generated at build time via generate_crypto_keys(), public keys extracted to OUT_DIR/keys.rs, private keys discarded immediately. Runtime has no signing capability.

CLAIM license_issuer_key_management = external_file_required
VERIFY = mechanical
EVIDENCE = src/bin/license_issuer.rs:1-100 separate binary with --private-key flag
EVIDENCE = generate-license subcommand requires file path to 32-byte raw private key
EVIDENCE = NO dependency on build.rs generated keys
RATIONALE = License generation is separate operational tool, not part of runtime. Private keys managed externally via file system, never compiled into binary.

### Premium Feature Architecture

CLAIM premium_features_wasm_dependency = optional_enhancement
VERIFY = mechanical
EVIDENCE = src/pro_engine/pro_loader.rs:13 "if !wasm_enc.exists() { return Ok(()); }"
EVIDENCE = src/engines/prediction/prediction_engine.rs:189-199 Premium mode uses ProEngine "IF AVAILABLE"
EVIDENCE = find . -name "*wasm.enc" returns empty (no WASM bundle in repository)
EVIDENCE = .github/workflows/release.yml does not build WASM bundle by default
RATIONALE = Premium features (advanced prediction algorithms) implemented in native Rust. WASM provides optional acceleration but is not required. Absence of WASM is handled gracefully, not an error.

### Wasmtime Vulnerability Risk Assessment

CLAIM wasmtime_vulnerability_reachable = false
VERIFY = mechanical
EVIDENCE = RUSTSEC-2025-0046 affects wasmtime 27.0.0 (Severity 3.3 LOW)
EVIDENCE = src/pro_engine/instantiate.rs:14-21 only called from pro_loader.rs:33
EVIDENCE = pro_loader.rs:11-14 requires ~/.costpilot/pro-engine.wasm.enc to exist
EVIDENCE = find . -name "*wasm.enc" returns empty (file not distributed)
EVIDENCE = cargo tree confirms wasmtime 27.0.0 statically linked
EVIDENCE = nm target/release/costpilot shows no exposed wasmtime symbols (stripped/LTO)
RATIONALE = Wasmtime code path only reached if WASM bundle exists at ~/.costpilot/pro-engine.wasm.enc. File is not distributed in repository or release artifacts. Vulnerability present in dependency tree but code path unreachable without user-supplied WASM bundle.

### Product Scope and Limitations

CLAIM optimization_detection_scope = static_rule_based
VERIFY = human
EVIDENCE = src/engines/explain/anti_patterns.rs:1-353 defines 5 MVP patterns
EVIDENCE = Patterns detect: NAT_GATEWAY_OVERUSE, OVERPROVISIONED_EC2 (generic), S3_MISSING_LIFECYCLE, UNBOUNDED_LAMBDA_CONCURRENCY, DYNAMODB_PAY_PER_REQUEST_DEFAULT
EVIDENCE = detect_overprovisioned_ec2() (lines 90-134) checks for "large"/"xlarge" strings in instance_type only
EVIDENCE = NO tag analysis (e.g., "light-processing" vs "high-throughput")
EVIDENCE = NO consolidation detection (multiple small instances with same Service tag)
EVIDENCE = NO storage optimization intelligence (gp2 vs gp3, IOPS analysis)
EVIDENCE = NO reserved instance utilization analysis
EVIDENCE = README.md:47-48 "No live infrastructure optimization\nNo historical billing analysis"
RATIONALE = CostPilot operates at PR boundary with static analysis. Anti-pattern detection uses simple heuristics (resource type + basic config checks). Does not perform workload intelligence, tag-based optimization recommendations, or runtime utilization analysis. Product scope is cost governance at review time, not live infrastructure optimization.

CLAIM optimization_detection_tag_awareness = none
VERIFY = mechanical
EVIDENCE = grep "tags" src/engines/explain/anti_patterns.rs returns 0 matches
EVIDENCE = detect_overprovisioned_ec2() (lines 90-134) reads instance_type only, ignores tags field
RATIONALE = Current anti-pattern detection does not inspect resource tags. Cannot correlate instance sizing with workload tags (e.g., "light-processing" on c5.9xlarge).

CLAIM optimization_detection_consolidation = none
VERIFY = mechanical
EVIDENCE = grep "consolidat" src/engines/explain/anti_patterns.rs returns 1 match in suggested_fix text only
EVIDENCE = detect_anti_patterns() (lines 19-50) processes one ResourceChange at a time
EVIDENCE = NO cross-resource analysis for multiple instances with same Service tag
RATIONALE = Anti-pattern detection is per-resource. No multi-resource pattern analysis to detect consolidation opportunities (e.g., 3x t3.micro → 1x t3.medium).

## Status
PROPOSED - Awaiting review

## Verification Required
- [ ] Run `python3 scripts/detect_mental_model_contradictions.py` after merge
- [ ] Verify no test failures with `cargo test --all-features -- --maxfail=1`
- [ ] Confirm binary inspection commands reproducible on other systems

## Impact Assessment
**Functional**: No change - claims document existing behavior
**Security**: Clarifies correct cryptographic architecture (secure by design)
**Commercial Launch**: Removes 4 false blockers from audit
**Product Scope**: Documents intentional limitations (PR-time governance, not live optimization)
