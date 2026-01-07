use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use std::env;
use std::fs;
use tempfile::TempDir;

// License enforcement edge cases tests

#[test]
fn test_expired_license() {
    let temp_dir = TempDir::new().unwrap();
    let license_path = temp_dir.path().join("license.json");
    let expired_license = r#"{
        "email": "test@example.com",
        "license_key": "TEST-KEY-12345",
        "expires": "2020-01-01T00:00:00Z",
        "signature": "dummy_signature",
        "issuer": "costpilot-v1"
    }"#;
    fs::write(&license_path, expired_license).unwrap();

    env::set_var("COSTPILOT_LICENSE", &license_path);

    let temp_plan_dir = TempDir::new().unwrap();
    let plan_path = temp_plan_dir.path().join("plan.json");
    fs::write(&plan_path, r#"{"format_version": "0.2", "resource_changes": []}"#).unwrap();

    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("diff")
        .arg("--format=text")
        .arg(&plan_path)
        .arg(&plan_path);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Invalid license"));

    env::remove_var("COSTPILOT_LICENSE");
}

#[test]
fn test_invalid_signature_license() {
    let temp_dir = TempDir::new().unwrap();
    let license_path = temp_dir.path().join("license.json");
    let invalid_sig_license = r#"{
        "email": "test@example.com",
        "license_key": "TEST-KEY-12345",
        "expires": "2026-12-31T23:59:59Z",
        "signature": "invalid_signature",
        "issuer": "costpilot-v1"
    }"#;
    fs::write(&license_path, invalid_sig_license).unwrap();

    env::set_var("COSTPILOT_LICENSE", &license_path);

    let temp_plan_dir = TempDir::new().unwrap();
    let plan_path = temp_plan_dir.path().join("plan.json");
    fs::write(&plan_path, r#"{"format_version": "0.2", "resource_changes": []}"#).unwrap();

    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("diff")
        .arg("--format=text")
        .arg(&plan_path)
        .arg(&plan_path);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Invalid license"));

    env::remove_var("COSTPILOT_LICENSE");
}

#[test]
fn test_missing_email_license() {
    let temp_dir = TempDir::new().unwrap();
    let license_path = temp_dir.path().join("license.json");
    let missing_email_license = r#"{
        "license_key": "TEST-KEY-12345",
        "expires": "2026-12-31T23:59:59Z",
        "signature": "dummy_signature",
        "issuer": "costpilot-v1"
    }"#;
    fs::write(&license_path, missing_email_license).unwrap();

    env::set_var("COSTPILOT_LICENSE", &license_path);

    let temp_plan_dir = TempDir::new().unwrap();
    let plan_path = temp_plan_dir.path().join("plan.json");
    fs::write(&plan_path, r#"{"format_version": "0.2", "resource_changes": []}"#).unwrap();

    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("diff")
        .arg("--format=text")
        .arg(&plan_path)
        .arg(&plan_path);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Missing required field"));

    env::remove_var("COSTPILOT_LICENSE");
}

#[test]
fn test_malformed_json_license() {
    let temp_dir = TempDir::new().unwrap();
    let license_path = temp_dir.path().join("license.json");
    let malformed_license = r#"{
        "email": "test@example.com",
        "license_key": "TEST-KEY-12345",
        "expires": "2026-12-31T23:59:59Z",
        "signature": "dummy_signature",
        "issuer": "costpilot-v1"
    "#; // missing closing brace
    fs::write(&license_path, malformed_license).unwrap();

    env::set_var("COSTPILOT_LICENSE", &license_path);

    let temp_plan_dir = TempDir::new().unwrap();
    let plan_path = temp_plan_dir.path().join("plan.json");
    fs::write(&plan_path, r#"{"format_version": "0.2", "resource_changes": []}"#).unwrap();

    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("diff")
        .arg("--format=text")
        .arg(&plan_path)
        .arg(&plan_path);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Invalid license"));

    env::remove_var("COSTPILOT_LICENSE");
}

#[test]
fn test_future_expires_license() {
    let temp_dir = TempDir::new().unwrap();
    let license_path = temp_dir.path().join("license.json");
    let future_license = r#"{
        "email": "test@example.com",
        "license_key": "TEST-KEY-12345",
        "expires": "2030-01-01T00:00:00Z",
        "signature": "dummy_signature",
        "issuer": "costpilot-v1"
    }"#;
    fs::write(&license_path, future_license).unwrap();

    env::set_var("COSTPILOT_LICENSE", &license_path);

    let temp_plan_dir = TempDir::new().unwrap();
    let plan_path = temp_plan_dir.path().join("plan.json");
    fs::write(&plan_path, r#"{"format_version": "0.2", "resource_changes": []}"#).unwrap();

    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("diff")
        .arg("--format=text")
        .arg(&plan_path)
        .arg(&plan_path);

    // Should succeed if signature is valid, but since dummy, might fail on sig
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Invalid license"));

    env::remove_var("COSTPILOT_LICENSE");
}
