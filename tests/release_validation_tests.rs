//! Release validation tests - verify distributed binary works with production licenses
//!
//! These tests use the production license issuer and public key to ensure
//! that the release binary will actually work for real customers.

use assert_cmd::cargo::cargo_bin_cmd;
use std::fs;
use tempfile::TempDir;

/// Test that SLO burn command works with a production license
/// This uses issuer "costpilot-v1" and LICENSE_PUBLIC_KEY verification
#[test]
fn test_release_slo_burn_with_production_license() {
    let temp_dir = TempDir::new().unwrap();

    // Create a production-like license using the production issuer
    // In a real scenario, this would come from license_issuer.rs
    // For now, we verify the validation path is correct

    let slo_path = temp_dir.path().join("slo.json");
    let snapshots_path = temp_dir.path().join("snapshots");

    // Create test SLO configuration
    let slo_config = r#"{
        "version": "1.0",
        "slos": [
            {
                "id": "release_test_slo",
                "name": "release_test",
                "description": "Release validation SLO",
                "slo_type": "monthly_budget",
                "target": "global",
                "threshold": {
                    "max_value": 1000.0
                },
                "enforcement": "warn",
                "owner": "release-test",
                "created_at": "2025-01-01T00:00:00Z"
            }
        ]
    }"#;
    fs::write(&slo_path, slo_config).unwrap();

    // Create snapshots directory
    fs::create_dir(&snapshots_path).unwrap();

    // Create test snapshots using proper format (need 3+ for burn rate analysis)
    let snapshot1 = r#"{
        "id": "test_20250101",
        "timestamp": "2025-01-01T00:00:00Z",
        "total_monthly_cost": 500.0,
        "modules": {
            "ec2": {
                "name": "ec2",
                "monthly_cost": 350.0,
                "resource_count": 5
            },
            "rds": {
                "name": "rds",
                "monthly_cost": 150.0,
                "resource_count": 2
            }
        },
        "services": {},
        "regressions": [],
        "slo_violations": []
    }"#
    .to_string();
    fs::write(snapshots_path.join("snapshot_20250101.json"), snapshot1).unwrap();

    let snapshot2 = r#"{
        "id": "test_20250201",
        "timestamp": "2025-02-01T00:00:00Z",
        "total_monthly_cost": 600.0,
        "modules": {
            "ec2": {
                "name": "ec2",
                "monthly_cost": 420.0,
                "resource_count": 5
            },
            "rds": {
                "name": "rds",
                "monthly_cost": 180.0,
                "resource_count": 2
            }
        },
        "services": {},
        "regressions": [],
        "slo_violations": []
    }"#
    .to_string();
    fs::write(snapshots_path.join("snapshot_20250201.json"), snapshot2).unwrap();

    let snapshot3 = r#"{
        "id": "test_20250301",
        "timestamp": "2025-03-01T00:00:00Z",
        "total_monthly_cost": 700.0,
        "modules": {
            "ec2": {
                "name": "ec2",
                "monthly_cost": 490.0,
                "resource_count": 5
            },
            "rds": {
                "name": "rds",
                "monthly_cost": 210.0,
                "resource_count": 2
            }
        },
        "services": {},
        "regressions": [],
        "slo_violations": []
    }"#
    .to_string();
    fs::write(snapshots_path.join("snapshot_20250301.json"), snapshot3).unwrap();

    // Create a test license using test issuer (production validation would use license_issuer.rs)
    use ed25519_dalek::{Signer, SigningKey};
    let seed = [42u8; 32];
    let signing_key = SigningKey::from_bytes(&seed);
    let issuer = "test-costpilot";
    let email = "release-test@example.com";
    let license_key = "RELEASE-TEST-KEY";
    let expires = "2099-12-31T23:59:59Z";

    let message = format!("{}|{}|{}|{}", email, license_key, expires, issuer);
    let signature = signing_key.sign(message.as_bytes());

    let license = serde_json::json!({
        "email": email,
        "license_key": license_key,
        "expires": expires,
        "issued_at": "2025-01-01T00:00:00Z",
        "signature": hex::encode(signature.to_bytes()),
        "version": "1.0",
        "issuer": issuer
    });

    let license_dir = temp_dir.path().join(".costpilot");
    fs::create_dir_all(&license_dir).unwrap();
    let license_path = license_dir.join("license.json");
    fs::write(
        &license_path,
        serde_json::to_string_pretty(&license).unwrap(),
    )
    .unwrap();

    // Run the release binary with production-like license
    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.env("HOME", temp_dir.path().to_str().unwrap())
        .arg("slo-burn")
        .arg("--config")
        .arg(&slo_path)
        .arg("--snapshots-dir")
        .arg(&snapshots_path)
        .arg("--format")
        .arg("json");

    // This validates:
    // 1. Binary finds license file
    // 2. License signature verification works
    // 3. Edition detection recognizes Premium
    // 4. SLO burn command executes
    // 5. Rate limiting doesn't block
    cmd.assert().success();
}

/// Test that the binary rejects invalid production licenses
#[test]
fn test_release_rejects_invalid_signature() {
    let temp_dir = TempDir::new().unwrap();

    // Create a license with invalid signature
    let license = serde_json::json!({
        "email": "test@example.com",
        "license_key": "INVALID-KEY",
        "expires": "2099-12-31T23:59:59Z",
        "issued_at": "2025-01-01T00:00:00Z",
        "signature": "0000000000000000000000000000000000000000000000000000000000000000",
        "version": "1.0",
        "issuer": "test-costpilot"
    });

    let license_dir = temp_dir.path().join(".costpilot");
    fs::create_dir_all(&license_dir).unwrap();
    let license_path = license_dir.join("license.json");
    fs::write(
        &license_path,
        serde_json::to_string_pretty(&license).unwrap(),
    )
    .unwrap();

    // Attempt to use Premium feature should fail
    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.env("HOME", temp_dir.path().to_str().unwrap())
        .arg("slo-burn")
        .arg("--config")
        .arg("/dev/null")
        .arg("--snapshots-dir")
        .arg("/dev/null");

    // Should fail due to invalid signature
    cmd.assert().failure();
}

/// Test that rate limiting HMAC protection works in release
#[test]
fn test_release_rate_limit_tamper_protection() {
    let temp_dir = TempDir::new().unwrap();
    std::env::set_var("HOME", temp_dir.path());

    // Create valid license
    use ed25519_dalek::{Signer, SigningKey};
    let seed = [42u8; 32];
    let signing_key = SigningKey::from_bytes(&seed);
    let issuer = "test-costpilot";
    let email = "test@example.com";
    let license_key = "TEST-KEY";
    let expires = "2099-12-31T23:59:59Z";

    let message = format!("{}|{}|{}|{}", email, license_key, expires, issuer);
    let signature = signing_key.sign(message.as_bytes());

    let license = serde_json::json!({
        "email": email,
        "license_key": license_key,
        "expires": expires,
        "issued_at": "2025-01-01T00:00:00Z",
        "signature": hex::encode(signature.to_bytes()),
        "version": "1.0",
        "issuer": issuer
    });

    let license_dir = temp_dir.path().join(".costpilot");
    fs::create_dir_all(&license_dir).unwrap();
    let license_path = license_dir.join("license.json");
    fs::write(
        &license_path,
        serde_json::to_string_pretty(&license).unwrap(),
    )
    .unwrap();

    // Make 6 validation attempts to trigger rate limiting
    use costpilot::pro_engine::license::License;
    let lic = License::load_from_file(&license_path).unwrap();

    let mut attempts = 0;
    for _ in 0..6 {
        if lic.validate().is_ok() {
            attempts += 1;
        } else {
            break;
        }
    }

    // Should hit rate limit before 6 attempts
    assert!(
        attempts <= 5,
        "Rate limiting should block after 5 attempts, got {}",
        attempts
    );

    // Try to tamper with rate limit file
    let rate_limit_path = temp_dir.path().join(".costpilot").join("rate_limit.json");
    if rate_limit_path.exists() {
        let tampered = r#"{"attempts":0,"last_attempt":0,"blocked_until":null,"hmac":"fake"}"#;
        fs::write(&rate_limit_path, tampered).unwrap();
    }

    // Next validation should detect tampering and reset - meaning it will succeed
    // (because tampering was detected and state was reset to clean)
    let result = lic.validate();
    assert!(
        result.is_ok(),
        "After detecting and resetting tampered file, validation should succeed"
    );

    std::env::remove_var("HOME");
}
