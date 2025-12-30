# Mental Model Version History

This file tracks changes to the mental model over time.
Each entry represents a validated delta that was applied.

## Version 1.0.0 - 2025-12-30

**Initial Mental Model Established**

- Project Identity: CostPilot as local CLI binary
- Execution Scope: Pull-request boundary only
- Security: No runtime network access
- Processing: Offline, deterministic, local-only
- Blocking: Only for explicit governance or safety violations
- Licensing: Offline, signature-verified, client-side only

**Validation Status**: ✅ All claims verified against codebase
**Authority Source**: Tests and executable code
**Conflicts Detected**: 0 (11 resolved via deltas)

---

## Delta Proposal Template

When proposing mental model changes, use this format:

### Proposed Delta: [Brief Description]

**Section**: [Mental Model Section Number/Name]

**Change Type**: [ADD | MODIFY | REMOVE | CLARIFY]

**Proposed Content**:
[Exact text to add/modify/remove]

**Evidence**:
- [Source 1]: [Description]
- [Source 2]: [Description]

**Rationale**:
[Why this change is needed]

**Impact Assessment**:
- [Breaking change? Yes/No]
- [Affects authority precedence? Yes/No]
- [Requires re-sync? Yes/No]

**Validation**:
- [How to verify this change]
- [Test cases to update]

---

### Proposed Delta: Refine Network Access Claim (False Positive Resolution)

**Section**: 1. Project Identity (Factual)

**Change Type**: CLARIFY

**Proposed Content**:
Runtime network access: Not permitted (except pattern matching for violation detection)

**Evidence**:
- `src/security/validator.rs`: Uses regex patterns `reqwest::` and `hyper::` to DETECT network violations, not perform them
- `Cargo.toml`: Does not include reqwest or hyper as dependencies
- `tests/zero_network_tests.rs`: Validates zero-network enforcement

**Rationale**:
The contradiction detector flags security validator pattern matching as network access usage. The validator is actually ENFORCING the no-network claim, not violating it.

**Impact Assessment**:
- Breaking change? No
- Affects authority precedence? No
- Requires re-sync? No

**Validation**:
- Run contradiction detector - should no longer flag validator.rs patterns
- Verify validator still detects actual network violations

---

### Proposed Delta: Refine Determinism Contract (Allow Controlled Non-Determinism)

**Section**: 5. Determinism Contract

**Change Type**: MODIFY

**Proposed Content**:
Given identical inputs:

- Outputs must be byte-identical
- Ordering must be stable
- Hashes must match across runs and platforms

Non-deterministic behavior is a defect, except:

- Cryptographic key generation (license issuer)
- Unique identifier generation (UUIDs for escrow receipts, metering events)
- Timestamp recording (SystemTime for audit trails, not for core logic)
- Deterministic pseudo-random sequences (Monte Carlo with fixed seeds)

**Evidence**:
- `src/bin/license_issuer.rs`: Uses `rand::` for cryptographic key generation
- `src/engines/escrow/release.rs`: UUID generation for package/receipt IDs
- `src/engines/metering/usage_meter.rs`: UUID generation for event IDs
- `src/engines/prediction/monte_carlo.rs`: Deterministic pseudo-random (not true randomness)
- `src/engines/policy/zero_network.rs`: Explicitly validates against non-deterministic operations

**Rationale**:
Absolute determinism is impractical for real systems. Need controlled exceptions for legitimate use cases while maintaining core deterministic guarantees.

**Impact Assessment**:
- Breaking change? No (clarifies existing practice)
- Affects authority precedence? No
- Requires re-sync? No

**Validation**:
- Run contradiction detector - should accept controlled non-determinism
- Verify ZeroNetworkValidator still rejects inappropriate randomness
- Test deterministic outputs for core cost calculations

**Review Status**: APPROVED
**Applied Date**: 2025-12-30
**Applied By**: Mental Model System (Automated Resolution)

---

### Proposed Delta: Fix Seasonality Determinism Violation

**Section**: 5. Determinism Contract

**Change Type**: CODE_FIX

**Proposed Content**:
Modified `SeasonalityDetector::calculate_current_adjustment()` to use deterministic pattern-based adjustment instead of `SystemTime::now()`

**Evidence**:
- `src/engines/prediction/seasonality.rs`: Was using `SystemTime::now()` for adjustment calculation
- Violated determinism contract requiring identical inputs → identical outputs

**Rationale**:
Seasonality analysis should be deterministic based on historical patterns, not current time.

**Impact Assessment**:
- Breaking change? No (maintains same API, changes internal behavior)
- Affects authority precedence? No
- Requires re-sync? No

**Validation**:
- Run contradiction detector - should no longer flag seasonality.rs
- Verify seasonality analysis remains deterministic across runs

**Review Status**: APPROVED
**Applied Date**: 2025-12-30
**Applied By**: Mental Model System (Automated Resolution)
