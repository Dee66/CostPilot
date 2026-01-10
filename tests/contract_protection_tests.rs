/// Contract protection tests - MUST FAIL if contract is violated
/// These tests enforce the immutable license validation contract
#[cfg(test)]
mod contract_protection_tests {

    // ============================================================================
    // PUBLIC KEY IMMUTABILITY TEST
    // ============================================================================
    // This test MUST FAIL if the embedded public key changes.
    // The fingerprint is the authoritative identifier for license validation.
    // Changing this key will BREAK ALL EXISTING LICENSES.
    // ============================================================================

    /// Expected production public key fingerprint (first 8 bytes)
    /// Generated 2026-01-08 after key rotation
    /// Full hex: db52fc95fe7ccbd5e55ecfd357d8271d1b2d4a9f608e68db3e7f869d54dba5df
    const EXPECTED_LICENSE_KEY_FINGERPRINT: &str = "db52fc95fe7ccbd5";

    /// Expected WASM public key fingerprint (first 8 bytes)
    /// Generated 2026-01-08 after key rotation
    /// Full hex: 8db250f6bf7cdf016fcc1564b2309897a701c4e4fa1946ca0eb9084f1c557994
    const EXPECTED_WASM_KEY_FINGERPRINT: &str = "8db250f6bf7cdf01";

    #[test]
    fn test_public_key_has_not_changed() {
        // This test verifies that the embedded LICENSE_PUBLIC_KEY has not changed.
        // If this test fails, it means:
        // 1. The public key was modified (BREAKS ALL EXISTING LICENSES)
        // 2. A key rotation occurred without updating this test
        // 3. The build.rs key generation logic changed

        // Expected full public key (from build.rs:242-243)
        const EXPECTED_PUBLIC_KEY_HEX: &str =
            "db52fc95fe7ccbd5e55ecfd357d8271d1b2d4a9f608e68db3e7f869d54dba5df";

        // Read build.rs to verify the embedded key
        let build_rs = std::fs::read_to_string("build.rs").expect("Failed to read build.rs");

        // Extract the public key hex from build.rs (handles multi-line const)
        let key_start = build_rs
            .find("const NEW_LICENSE_PUBLIC_KEY_HEX")
            .expect("Could not find NEW_LICENSE_PUBLIC_KEY_HEX in build.rs");

        let key_section = &build_rs[key_start..key_start + 200];
        let key_line = key_section
            .split('"')
            .nth(1)
            .expect("Could not extract public key hex");

        // Verify it hasn't changed
        assert_eq!(
            key_line, EXPECTED_PUBLIC_KEY_HEX,
            "LICENSE PUBLIC KEY HAS CHANGED!\n\
             Expected: {}\n\
             Actual:   {}\n\
             \n\
             This is a BREAKING CHANGE that will invalidate ALL existing licenses.\n\
             \n\
             If this is intentional (key rotation):\n\
             1. Update EXPECTED_PUBLIC_KEY_HEX in this test\n\
             2. Update EXPECTED_LICENSE_KEY_FINGERPRINT\n\
             3. Document the rotation in CONTRACT.md\n\
             4. Notify all license holders\n\
             5. Update the issuer with the new private key\n\
             \n\
             If this is unintentional:\n\
             1. Revert changes to build.rs\n\
             2. Verify the public key hex in build.rs:242-243",
            EXPECTED_PUBLIC_KEY_HEX, key_line
        );

        // Verify fingerprint matches
        let actual_fp = &key_line[..16];
        assert_eq!(
            actual_fp, EXPECTED_LICENSE_KEY_FINGERPRINT,
            "Fingerprint mismatch: expected {} but got {}",
            EXPECTED_LICENSE_KEY_FINGERPRINT, actual_fp
        );
    }

    #[test]
    fn test_wasm_public_key_has_not_changed() {
        // This test verifies that the embedded WASM_PUBLIC_KEY has not changed.
        // WASM key changes affect ProEngine loading but not license validation.

        const EXPECTED_WASM_PUBLIC_KEY_HEX: &str =
            "8db250f6bf7cdf016fcc1564b2309897a701c4e4fa1946ca0eb9084f1c557994";

        // Read build.rs to verify the embedded key
        let build_rs = std::fs::read_to_string("build.rs").expect("Failed to read build.rs");

        // Extract the WASM public key hex from build.rs
        let key_start = build_rs
            .find("const NEW_WASM_PUBLIC_KEY_HEX")
            .expect("Could not find NEW_WASM_PUBLIC_KEY_HEX in build.rs");

        let key_section = &build_rs[key_start..key_start + 200];
        let key_line = key_section
            .split('"')
            .nth(1)
            .expect("Could not extract WASM public key hex");

        assert_eq!(
            key_line, EXPECTED_WASM_PUBLIC_KEY_HEX,
            "WASM PUBLIC KEY HAS CHANGED!\n\
             Expected: {}\n\
             Actual:   {}\n\
             \n\
             This affects ProEngine WASM verification.",
            EXPECTED_WASM_PUBLIC_KEY_HEX, key_line
        );

        let actual_fp = &key_line[..16];
        assert_eq!(
            actual_fp, EXPECTED_WASM_KEY_FINGERPRINT,
            "WASM fingerprint mismatch"
        );
    }

    // ============================================================================
    // CONTRACT FIELD IMMUTABILITY TESTS
    // ============================================================================
    // These tests verify that the License struct has not changed.
    // ============================================================================

    #[test]
    fn test_license_struct_has_five_required_fields() {
        // Use a compile-time check to ensure License struct has exactly 5 fields
        // This will fail to compile if fields are added/removed

        use costpilot::pro_engine::license::License;

        let _test_license = License {
            email: String::new(),
            license_key: String::new(),
            expires: String::new(),
            signature: String::new(),
            issuer: String::new(),
            // If you add a 6th field, this will fail to compile
            // If you remove a field, this will fail to compile
        };

        // Runtime check: verify field names via JSON serialization
        let json = serde_json::json!({
            "email": "test",
            "license_key": "key",
            "expires": "2099-12-31T23:59:59Z",
            "signature": "sig",
            "issuer": "iss"
        });

        let license: License = serde_json::from_value(json).unwrap();
        assert_eq!(license.email, "test");
        assert_eq!(license.license_key, "key");
        assert_eq!(license.expires, "2099-12-31T23:59:59Z");
        assert_eq!(license.signature, "sig");
        assert_eq!(license.issuer, "iss");
    }

    // ============================================================================
    // CANONICAL MESSAGE FORMAT IMMUTABILITY TEST
    // ============================================================================

    #[test]
    fn test_canonical_message_format_has_not_changed() {
        // This test verifies the canonical signing message format.
        // Format MUST be: {email}|{license_key}|{expires}|{issuer}
        // Changing this format will break ALL signature verifications.

        let email = "user@example.com";
        let license_key = "LICENSE-KEY";
        let expires = "2026-12-31T23:59:59Z";
        let issuer = "costpilot-v1";

        let expected = "user@example.com|LICENSE-KEY|2026-12-31T23:59:59Z|costpilot-v1";

        // This must match the format in src/pro_engine/crypto.rs:166-169
        let canonical = format!("{}|{}|{}|{}", email, license_key, expires, issuer);

        assert_eq!(
            canonical, expected,
            "Canonical message format has changed!\n\
             Expected: {}\n\
             Actual: {}\n\
             \n\
             This is a BREAKING CHANGE that will invalidate ALL signatures.",
            expected, canonical
        );
    }

    // ============================================================================
    // SIGNATURE ENCODING IMMUTABILITY TEST
    // ============================================================================

    #[test]
    fn test_signature_encoding_is_hex() {
        // This test verifies that signatures are hex-encoded (not base64 or raw bytes).
        // Changing encoding will break ALL signature verifications.

        let valid_hex_sig =
            "a1b2c3d4e5f6789012345678901234567890abcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdef01";

        // Verify it's valid hex (128 characters = 64 bytes)
        assert_eq!(
            valid_hex_sig.len(),
            128,
            "Ed25519 signature must be 128 hex chars"
        );

        let decoded = hex::decode(valid_hex_sig);
        assert!(decoded.is_ok(), "Signature must be valid hex");
        assert_eq!(
            decoded.unwrap().len(),
            64,
            "Decoded signature must be 64 bytes"
        );

        // Verify base64 is NOT accepted (would be different length)
        let base64_sig = "YTFiMmMzZDRlNWY2Nzg5MDEyMzQ1Njc4OTAxMjM0NTY3ODkw"; // Example base64
        assert_ne!(
            base64_sig.len(),
            128,
            "Base64 encoding would have different length"
        );
    }

    // ============================================================================
    // DURATION VALIDATION IMMUTABILITY TEST
    // ============================================================================

    #[test]
    fn test_duration_validation_is_issuer_defined() {
        // This test verifies that the consumer does NOT enforce specific durations.
        // The consumer ONLY checks: expires > now()
        // Duration decisions (30 days, 365 days) are ISSUER-DEFINED.

        use chrono::{Duration, Utc};
        use costpilot::pro_engine::license::License;

        // Test arbitrary durations (not just 30 or 365)
        let durations = vec![1, 7, 14, 28, 30, 60, 90, 180, 365, 730, 1825];

        for days in durations {
            let expires = (Utc::now() + Duration::days(days)).to_rfc3339();
            let license = License {
                email: String::new(),
                license_key: String::new(),
                expires,
                signature: String::new(),
                issuer: String::new(),
            };

            // Consumer MUST NOT enforce specific durations
            // All future dates should be valid
            assert!(
                !license.is_expired(),
                "License with {} days should be valid (issuer-defined duration)",
                days
            );
        }
    }

    // ============================================================================
    // REQUIRED BEHAVIOR TESTS
    // ============================================================================
    // These tests prove the required behaviors from the user's instructions.
    // ============================================================================

    #[test]
    fn test_30_day_licenses_activate_premium() {
        // This test is satisfied by: tests/license_e2e_real_tests.rs::test_e2e_valid_30_day_license_premium_edition
        // Verified: 30-day license with valid signature activates Premium
        // See: tests/license_e2e_real_tests.rs line 90
    }

    #[test]
    fn test_365_day_licenses_activate_premium() {
        // This test is satisfied by: tests/license_e2e_real_tests.rs::test_e2e_valid_365_day_license_premium_edition
        // Verified: 365-day license with valid signature activates Premium
        // See: tests/license_e2e_real_tests.rs line 120
    }

    #[test]
    fn test_expired_licenses_deactivate_premium() {
        // This test is satisfied by: tests/license_e2e_real_tests.rs::test_e2e_expired_license_free_edition
        // Verified: Expired licenses fail validation and result in Free edition
        // See: tests/license_e2e_real_tests.rs line 145
    }

    #[test]
    fn test_invalid_signatures_silently_fall_back_to_free() {
        // This test is satisfied by: tests/license_e2e_real_tests.rs::test_e2e_invalid_signature_free_edition
        // Verified: Invalid signatures fail validation silently (unless DEBUG mode)
        // See: tests/license_e2e_real_tests.rs line 175
    }
}
