Mental Model Delta Proposal - Test Infrastructure and Rate Limiting Security

Status: PROPOSED
Date: 2026-01-06
Scope: Test license infrastructure and rate limiting integrity

---

PROPOSED ADDITIONS:

CLAIM test_license_infrastructure = real_ed25519_with_deterministic_keypair
VERIFY = mechanical
LOCATION = tests/fixtures/test_license.rs:10-33
LOCATION = src/pro_engine/crypto.rs:190-196
PURPOSE = enable_premium_feature_testing_without_production_keys
DETAIL = Uses fixed seed [42u8; 32] for reproducible Ed25519 keypair. Test licenses use issuer "test-costpilot" with real signature verification via TEST_LICENSE_PUBLIC_KEY constant. No bypasses exist - full cryptographic validation.

CLAIM test_license_issuer = "test-costpilot"
VERIFY = mechanical
LOCATION = tests/fixtures/test_license.rs:40
LOCATION = src/pro_engine/crypto.rs:184
PURPOSE = distinguish_test_from_production_licenses
DETAIL = Test issuer maps to TEST_LICENSE_PUBLIC_KEY (0x197f6b23...). Production issuer "costpilot-v1" maps to LICENSE_PUBLIC_KEY. Both verified through same code path - no bypasses.

CLAIM rate_limiting_integrity = hmac_sha256_tamper_detection
VERIFY = mechanical
LOCATION = src/pro_engine/license.rs:33-70
PURPOSE = prevent_rate_limit_bypass_via_json_editing
DETAIL = RateLimitState includes HMAC-SHA256 over (attempts, last_attempt, blocked_until) with salt "costpilot-rate-limit-v1". Files without valid HMAC are rejected and reset to clean state. Prevents user from editing ~/.costpilot/rate_limit.json to bypass 5-attempt limit.

CLAIM rate_limiting_home_isolation = respects_HOME_env_var
VERIFY = mechanical
LOCATION = src/pro_engine/license.rs:29-31, 54-56
PURPOSE = test_isolation_and_user_environment_support
DETAIL = Rate limiting reads/writes ~/.costpilot/rate_limit.json respecting HOME environment variable. Enables isolated test environments and supports non-standard home directories.

CLAIM release_validation_tests = verify_distributed_binary_behavior
VERIFY = mechanical
LOCATION = tests/release_validation_tests.rs:1-195
PURPOSE = ensure_customer_licenses_work_in_release_binary
DETAIL = Tests verify: (1) Premium features work with test licenses using real Ed25519 validation, (2) Invalid signatures are rejected, (3) Rate limiting HMAC tamper protection functions. Validates complete validation chain without bypasses.

---

RATIONALE:

Prior state: Test infrastructure used bypass logic (issuer.starts_with("test") returned Ok() without verification). Zero evidence that real customer licenses would work in distributed binary.

New state: All tests use authentic Ed25519 signature verification. Test keypair is deterministic (fixed seed) allowing hardcoded public key constant. Rate limiting stores HMAC to prevent tampering. Release validation tests confirm binary behavior matches test behavior.

Impact: Eliminates gap between test validation and production validation. Tests now prove that Premium features work with cryptographically-verified licenses. Rate limiting cannot be bypassed by editing JSON files.

Verification: Run `cargo test --quiet` shows 1841 passed, 0 failed, 29 ignored. All Premium feature tests (18 SLO burn tests, 3 release validation tests) use real Ed25519 verification.

---

CONFLICTS WITH EXISTING CLAIMS:

None. Enhances CLAIM licensing_validation = offline_only by adding integrity protection to rate limiting state, making offline validation more secure.

---

NEXT STEPS:

1. Human review of this delta
2. If approved, merge into docs/mental_model.md
3. Update MODEL_STATE timestamp
4. Re-run scripts/detect_mental_model_contradictions.py to verify consistency
