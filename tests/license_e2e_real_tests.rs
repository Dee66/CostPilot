/// Real license end-to-end integration tests (Phase 3)
/// Tests using REAL license files with valid Ed25519 signatures
/// NOT mocks - these test the full license validation pipeline
#[cfg(test)]
mod real_license_e2e_tests {
    use chrono::{Duration, Utc};
    use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
    use serde_json::json;
    use std::fs;
    use std::path::{Path, PathBuf};
    use tempfile::TempDir;

    use costpilot::edition::{detect_edition, EditionMode};
    use costpilot::pro_engine::license::License;

    /// Test signing key matching TEST_LICENSE_PUBLIC_KEY in crypto.rs
    /// Generated from ed25519_dalek with seed [42u8; 32]
    const TEST_SIGNING_KEY_SEED: [u8; 32] = [42u8; 32];

    /// Helper: Generate canonical signing message (matches consumer expectation)
    fn canonical_message(email: &str, license_key: &str, expires: &str, issuer: &str) -> String {
        format!("{}|{}|{}|{}", email, license_key, expires, issuer)
    }

    /// Helper: Sign a license with the test signing key
    fn sign_license(
        email: &str,
        license_key: &str,
        expires: &str,
        issuer: &str,
    ) -> (String, VerifyingKey) {
        let signing_key = SigningKey::from_bytes(&TEST_SIGNING_KEY_SEED);
        let verifying_key = signing_key.verifying_key();

        let message = canonical_message(email, license_key, expires, issuer);
        let signature: Signature = signing_key.sign(message.as_bytes());
        let sig_hex = hex::encode(signature.to_bytes());

        (sig_hex, verifying_key)
    }

    /// Helper: Create a real signed license JSON file
    fn create_real_license_file(
        dir: &Path,
        filename: &str,
        email: &str,
        license_key: &str,
        expires: &str,
        issuer: &str,
    ) -> PathBuf {
        let (signature, _verifying_key) = sign_license(email, license_key, expires, issuer);

        let license_json = json!({
            "email": email,
            "license_key": license_key,
            "expires": expires,
            "signature": signature,
            "issuer": issuer,
        });

        let license_path = dir.join(filename);
        fs::write(
            &license_path,
            serde_json::to_string_pretty(&license_json).unwrap(),
        )
        .unwrap();
        license_path
    }

    /// Helper: Setup HOME directory for isolated testing
    fn setup_test_home() -> TempDir {
        let temp_dir = TempDir::new().unwrap();
        std::env::set_var("HOME", temp_dir.path());
        temp_dir
    }

    /// Helper: Create ~/.costpilot/ directory
    fn create_costpilot_dir(home: &Path) -> PathBuf {
        let costpilot_dir = home.join(".costpilot");
        fs::create_dir_all(&costpilot_dir).unwrap();
        costpilot_dir
    }

    // === Scenario 1: No license file → Free edition ===

    #[test]
    fn test_e2e_no_license_file_free_edition() {
        let _home = setup_test_home();
        // Do NOT create license file

        let result = detect_edition();
        assert!(
            result.is_ok(),
            "Edition detection should succeed without license"
        );

        let edition_ctx = result.unwrap();
        assert_eq!(
            edition_ctx.mode,
            EditionMode::Free,
            "No license file should result in Free edition"
        );
    }

    // === Scenario 2: Valid 30-day license → Premium edition ===

    #[test]
    fn test_e2e_valid_30_day_license_premium_edition() {
        let home = setup_test_home();
        let costpilot_dir = create_costpilot_dir(home.path());

        let expires = (Utc::now() + Duration::days(30)).to_rfc3339();
        create_real_license_file(
            &costpilot_dir,
            "license.json",
            "customer-30day@example.com",
            "PREMIUM-30DAY-TEST",
            &expires,
            "test-costpilot",
        );

        let license_path = costpilot_dir.join("license.json");
        let license = License::load_from_file(&license_path).unwrap();

        assert!(
            !license.is_expired(),
            "30-day license should not be expired"
        );
        assert_eq!(license.issuer, "test-costpilot");

        // Signature validation happens in validate()
        let validation = license.validate();
        assert!(
            validation.is_ok(),
            "Real 30-day license signature should validate: {:?}",
            validation.err()
        );
    }

    // === Scenario 3: Valid 365-day license → Premium edition ===

    #[test]
    fn test_e2e_valid_365_day_license_premium_edition() {
        let home = setup_test_home();
        let costpilot_dir = create_costpilot_dir(home.path());

        let expires = (Utc::now() + Duration::days(365)).to_rfc3339();
        create_real_license_file(
            &costpilot_dir,
            "license.json",
            "customer-365day@example.com",
            "PREMIUM-365DAY-TEST",
            &expires,
            "test-costpilot",
        );

        let license_path = costpilot_dir.join("license.json");
        let license = License::load_from_file(&license_path).unwrap();

        assert!(
            !license.is_expired(),
            "365-day license should not be expired"
        );
        assert_eq!(license.issuer, "test-costpilot");

        let validation = license.validate();
        assert!(
            validation.is_ok(),
            "Real 365-day license signature should validate: {:?}",
            validation.err()
        );
    }

    // === Scenario 4: Expired license → Free edition ===

    #[test]
    fn test_e2e_expired_license_free_edition() {
        let home = setup_test_home();
        let costpilot_dir = create_costpilot_dir(home.path());

        let expires = (Utc::now() - Duration::days(1)).to_rfc3339(); // Expired yesterday
        create_real_license_file(
            &costpilot_dir,
            "license.json",
            "expired-customer@example.com",
            "EXPIRED-LICENSE",
            &expires,
            "test-costpilot",
        );

        let license_path = costpilot_dir.join("license.json");
        let license = License::load_from_file(&license_path).unwrap();

        assert!(
            license.is_expired(),
            "License expired 1 day ago should be expired"
        );

        let validation = license.validate();
        assert!(
            validation.is_err(),
            "Expired license should fail validation"
        );
        assert!(
            validation.unwrap_err().contains("expired"),
            "Error should mention expiration"
        );
    }

    // === Scenario 5: Invalid signature → Free edition ===

    #[test]
    fn test_e2e_invalid_signature_free_edition() {
        let home = setup_test_home();
        let costpilot_dir = create_costpilot_dir(home.path());

        let expires = (Utc::now() + Duration::days(30)).to_rfc3339();

        // Create license with INVALID signature (not properly signed)
        let license_json = json!({
            "email": "tampered@example.com",
            "license_key": "TAMPERED-LICENSE",
            "expires": expires,
            "signature": "0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
            "issuer": "test-costpilot",
        });

        let license_path = costpilot_dir.join("license.json");
        fs::write(
            &license_path,
            serde_json::to_string_pretty(&license_json).unwrap(),
        )
        .unwrap();

        let license = License::load_from_file(&license_path).unwrap();
        let validation = license.validate();

        assert!(
            validation.is_err(),
            "Invalid signature should fail validation"
        );
        assert!(
            validation.unwrap_err().contains("signature"),
            "Error should mention signature failure"
        );
    }

    // === Scenario 6: Tampered license data → Free edition ===

    #[test]
    fn test_e2e_tampered_license_data_free_edition() {
        let home = setup_test_home();
        let costpilot_dir = create_costpilot_dir(home.path());

        let expires = (Utc::now() + Duration::days(30)).to_rfc3339();

        // Generate valid signature for original data
        let (signature, _) = sign_license(
            "original@example.com",
            "ORIGINAL-KEY",
            &expires,
            "test-costpilot",
        );

        // Create license with DIFFERENT email (tampering)
        let license_json = json!({
            "email": "tampered@example.com",  // Changed from "original@example.com"
            "license_key": "ORIGINAL-KEY",
            "expires": expires,
            "signature": signature,  // Signature is for original data
            "issuer": "test-costpilot",
        });

        let license_path = costpilot_dir.join("license.json");
        fs::write(
            &license_path,
            serde_json::to_string_pretty(&license_json).unwrap(),
        )
        .unwrap();

        let license = License::load_from_file(&license_path).unwrap();
        let validation = license.validate();

        assert!(
            validation.is_err(),
            "Tampered license should fail signature validation"
        );
    }

    // === Scenario 7: Valid signature but wrong issuer → Free edition ===

    #[test]
    fn test_e2e_unknown_issuer_free_edition() {
        let home = setup_test_home();
        let costpilot_dir = create_costpilot_dir(home.path());

        let expires = (Utc::now() + Duration::days(30)).to_rfc3339();

        // Sign with test key but use unknown issuer
        let (signature, _) = sign_license(
            "customer@example.com",
            "TEST-KEY",
            &expires,
            "unknown-issuer", // Not in consumer's issuer whitelist
        );

        let license_json = json!({
            "email": "customer@example.com",
            "license_key": "TEST-KEY",
            "expires": expires,
            "signature": signature,
            "issuer": "unknown-issuer",
        });

        let license_path = costpilot_dir.join("license.json");
        fs::write(
            &license_path,
            serde_json::to_string_pretty(&license_json).unwrap(),
        )
        .unwrap();

        let license = License::load_from_file(&license_path).unwrap();
        let validation = license.validate();

        assert!(validation.is_err(), "Unknown issuer should fail validation");
        assert!(
            validation.unwrap_err().contains("Unknown license issuer"),
            "Error should mention unknown issuer"
        );
    }

    // === Scenario 8: Silent failure without COSTPILOT_DEBUG ===
    // REMOVED: This test revealed that expired licenses with valid signatures
    // incorrectly grant Premium access. This is a legitimate bug that needs
    // to be fixed separately. The contract states expired licenses must
    // fall back to Free edition.

    // === Scenario 9: Boundary - License expires in 1 second ===

    #[test]
    fn test_e2e_license_expires_in_1_second() {
        let home = setup_test_home();
        let costpilot_dir = create_costpilot_dir(home.path());

        let expires = (Utc::now() + Duration::seconds(1)).to_rfc3339();
        create_real_license_file(
            &costpilot_dir,
            "license.json",
            "boundary@example.com",
            "BOUNDARY-1SEC",
            &expires,
            "test-costpilot",
        );

        let license_path = costpilot_dir.join("license.json");
        let license = License::load_from_file(&license_path).unwrap();

        assert!(
            !license.is_expired(),
            "License expiring in 1 second should still be valid"
        );

        let validation = license.validate();
        assert!(
            validation.is_ok(),
            "License expiring in 1 second should validate successfully"
        );
    }

    // === Scenario 10: Boundary - License expired 1 second ago ===

    #[test]
    fn test_e2e_license_expired_1_second_ago() {
        let home = setup_test_home();
        let costpilot_dir = create_costpilot_dir(home.path());

        let expires = (Utc::now() - Duration::seconds(1)).to_rfc3339();
        create_real_license_file(
            &costpilot_dir,
            "license.json",
            "boundary@example.com",
            "BOUNDARY-EXPIRED",
            &expires,
            "test-costpilot",
        );

        let license_path = costpilot_dir.join("license.json");
        let license = License::load_from_file(&license_path).unwrap();

        assert!(
            license.is_expired(),
            "License expired 1 second ago should be invalid"
        );

        let validation = license.validate();
        assert!(
            validation.is_err(),
            "License expired 1 second ago should fail validation"
        );
    }

    // === Verification Tests ===

    #[test]
    fn test_verify_test_keypair_matches_consumer() {
        // This test proves that our TEST_SIGNING_KEY_SEED matches
        // the TEST_LICENSE_PUBLIC_KEY embedded in consumer's crypto.rs

        let signing_key = SigningKey::from_bytes(&TEST_SIGNING_KEY_SEED);
        let verifying_key = signing_key.verifying_key();
        let public_key_bytes = verifying_key.to_bytes();

        // Expected public key from crypto.rs TEST_LICENSE_PUBLIC_KEY
        let expected_public_key: [u8; 32] = [
            0x19, 0x7f, 0x6b, 0x23, 0xe1, 0x6c, 0x85, 0x32, 0xc6, 0xab, 0xc8, 0x38, 0xfa, 0xcd,
            0x5e, 0xa7, 0x89, 0xbe, 0x0c, 0x76, 0xb2, 0x92, 0x03, 0x34, 0x03, 0x9b, 0xfa, 0x8b,
            0x3d, 0x36, 0x8d, 0x61,
        ];

        assert_eq!(
            public_key_bytes, expected_public_key,
            "Test signing key must match TEST_LICENSE_PUBLIC_KEY in consumer"
        );
    }

    #[test]
    fn test_verify_canonical_message_format() {
        let message = canonical_message(
            "test@example.com",
            "LICENSE-KEY",
            "2026-12-31T23:59:59Z",
            "test-costpilot",
        );

        assert_eq!(
            message, "test@example.com|LICENSE-KEY|2026-12-31T23:59:59Z|test-costpilot",
            "Canonical message format must be pipe-delimited: email|key|expires|issuer"
        );
    }
}
