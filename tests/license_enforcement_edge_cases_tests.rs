use assert_cmd::cargo::cargo_bin_cmd;
use ed25519_dalek::{Signer, SigningKey};
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

// License enforcement edge cases tests

#[test]
fn test_expired_license() {
    let temp_dir = TempDir::new().unwrap();
    let costpilot_dir = temp_dir.path().join(".costpilot");
    fs::create_dir_all(&costpilot_dir).unwrap();
    let license_path = costpilot_dir.join("license.json");

    // Generate a valid signature for an expired license
    let seed = [42u8; 32];
    let signing_key = SigningKey::from_bytes(&seed);
    let issuer = "test-costpilot";
    let email = "test@example.com";
    let license_key = "TEST-KEY-12345";
    let expires = "2020-01-01T00:00:00Z"; // Expired

    let message = format!("{}|{}|{}|{}", email, license_key, expires, issuer);
    let signature = signing_key.sign(message.as_bytes());

    let expired_license = format!(
        r#"{{
        "email": "{}",
        "license_key": "{}",
        "expires": "{}",
        "signature": "{}",
        "issuer": "{}"
    }}"#,
        email,
        license_key,
        expires,
        hex::encode(signature.to_bytes()),
        issuer
    );
    fs::write(&license_path, expired_license).unwrap();

    let temp_plan_dir = TempDir::new().unwrap();
    let plan_path = temp_plan_dir.path().join("plan.json");
    fs::write(
        &plan_path,
        r#"{"format_version": "0.2", "resource_changes": []}"#,
    )
    .unwrap();

    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.env("HOME", temp_dir.path())
        .arg("diff")
        .arg("--format=text")
        .arg(&plan_path)
        .arg(&plan_path);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("license expired"));
}

#[test]
fn test_invalid_signature_license() {
    let temp_dir = TempDir::new().unwrap();
    let costpilot_dir = temp_dir.path().join(".costpilot");
    fs::create_dir_all(&costpilot_dir).unwrap();
    let license_path = costpilot_dir.join("license.json");

    // Create license with intentionally invalid signature
    let invalid_sig_license = r#"{
        "email": "test@example.com",
        "license_key": "TEST-KEY-12345",
        "expires": "2026-12-31T23:59:59Z",
        "signature": "0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
        "issuer": "test-costpilot"
    }"#;
    fs::write(&license_path, invalid_sig_license).unwrap();

    let temp_plan_dir = TempDir::new().unwrap();
    let plan_path = temp_plan_dir.path().join("plan.json");
    fs::write(
        &plan_path,
        r#"{"format_version": "0.2", "resource_changes": []}"#,
    )
    .unwrap();

    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.env("HOME", temp_dir.path())
        .arg("diff")
        .arg("--format=text")
        .arg(&plan_path)
        .arg(&plan_path);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("signature verification failed"));
}

#[test]
fn test_missing_email_license() {
    let temp_dir = TempDir::new().unwrap();
    let costpilot_dir = temp_dir.path().join(".costpilot");
    fs::create_dir_all(&costpilot_dir).unwrap();
    let license_path = costpilot_dir.join("license.json");

    // Missing email field
    let missing_email_license = r#"{
        "license_key": "TEST-KEY-12345",
        "expires": "2026-12-31T23:59:59Z",
        "signature": "dummy_signature",
        "issuer": "test-costpilot"
    }"#;
    fs::write(&license_path, missing_email_license).unwrap();

    let temp_plan_dir = TempDir::new().unwrap();
    let plan_path = temp_plan_dir.path().join("plan.json");
    fs::write(
        &plan_path,
        r#"{"format_version": "0.2", "resource_changes": []}"#,
    )
    .unwrap();

    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.env("HOME", temp_dir.path())
        .arg("diff")
        .arg("--format=text")
        .arg(&plan_path)
        .arg(&plan_path);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("missing field"));
}

#[test]
fn test_malformed_json_license() {
    let temp_dir = TempDir::new().unwrap();
    let costpilot_dir = temp_dir.path().join(".costpilot");
    fs::create_dir_all(&costpilot_dir).unwrap();
    let license_path = costpilot_dir.join("license.json");

    // Malformed JSON (missing closing brace)
    let malformed_license = r#"{
        "email": "test@example.com",
        "license_key": "TEST-KEY-12345",
        "expires": "2026-12-31T23:59:59Z",
        "signature": "dummy_signature",
        "issuer": "test-costpilot"
    "#;
    fs::write(&license_path, malformed_license).unwrap();

    let temp_plan_dir = TempDir::new().unwrap();
    let plan_path = temp_plan_dir.path().join("plan.json");
    fs::write(
        &plan_path,
        r#"{"format_version": "0.2", "resource_changes": []}"#,
    )
    .unwrap();

    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.env("HOME", temp_dir.path())
        .arg("diff")
        .arg("--format=text")
        .arg(&plan_path)
        .arg(&plan_path);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Invalid license"));
}

#[test]
fn test_future_expires_license() {
    let temp_dir = TempDir::new().unwrap();
    let costpilot_dir = temp_dir.path().join(".costpilot");
    fs::create_dir_all(&costpilot_dir).unwrap();
    let license_path = costpilot_dir.join("license.json");

    // Generate a valid signature for a future expiry license
    let seed = [42u8; 32];
    let signing_key = SigningKey::from_bytes(&seed);
    let issuer = "test-costpilot";
    let email = "test@example.com";
    let license_key = "TEST-KEY-12345";
    let expires = "2030-01-01T00:00:00Z"; // Future date

    let message = format!("{}|{}|{}|{}", email, license_key, expires, issuer);
    let signature = signing_key.sign(message.as_bytes());

    let future_license = format!(
        r#"{{
        "email": "{}",
        "license_key": "{}",
        "expires": "{}",
        "signature": "{}",
        "issuer": "{}"
    }}"#,
        email,
        license_key,
        expires,
        hex::encode(signature.to_bytes()),
        issuer
    );
    fs::write(&license_path, future_license).unwrap();

    let temp_plan_dir = TempDir::new().unwrap();
    let plan_path = temp_plan_dir.path().join("plan.json");
    fs::write(
        &plan_path,
        r#"{"format_version": "0.2", "resource_changes": []}"#,
    )
    .unwrap();

    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.env("HOME", temp_dir.path())
        .arg("diff")
        .arg("--format=text")
        .arg(&plan_path)
        .arg(&plan_path);

    // Should succeed because license is valid and not expired
    cmd.assert().success();
}
