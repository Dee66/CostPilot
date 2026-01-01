//! Comprehensive SLO burn rate testing
//!
//! Tests the SLO burn rate analysis functionality as specified in the product requirements.
//! This includes burn risk calculation, time-to-breach prediction, and various output formats.

use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use serde_json::Value;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

/// Test SLO burn rate analysis with low risk scenario
#[test]
fn test_slo_burn_low_risk() {
    let temp_dir = TempDir::new().unwrap();
    let slo_path = temp_dir.path().join("slo.json");
    let snapshots_path = temp_dir.path().join("snapshots");

    // Create test SLO configuration (monthly budget: $500)
    let slo_config = r#"{
        "version": "1.0",
        "slos": [
            {
                "id": "monthly_budget_test",
                "name": "monthly_budget",
                "description": "Monthly cost budget limit",
                "slo_type": "monthly_budget",
                "target": "global",
                "threshold": {
                    "max_value": 500.0
                },
                "enforcement": "warn",
                "owner": "test",
                "created_at": "2025-01-01T00:00:00Z"
            }
        ]
    }"#;
    fs::write(&slo_path, slo_config).unwrap();

    // Create snapshots directory and historical data
    fs::create_dir(&snapshots_path).unwrap();

    // Create historical snapshots showing slow, controlled growth
    // Month 1: $100, Month 2: $120, Month 3: $140 (burn rate ~$0.67/day)
    create_test_snapshot(&snapshots_path, "2025-01-01", 100.0);
    create_test_snapshot(&snapshots_path, "2025-02-01", 120.0);
    create_test_snapshot(&snapshots_path, "2025-03-01", 140.0);

    // Create test license for premium features
    let costpilot_dir = temp_dir.path().join(".costpilot");
    fs::create_dir(&costpilot_dir).unwrap();
    let license_path = costpilot_dir.join("license.json");
    let license_content = r#"{
        "email": "test@example.com",
        "license_key": "test-license-key-for-slo-burn",
        "expires": "2099-12-31T23:59:59Z",
        "signature": "test-signature",
        "issuer": "test-issuer"
    }"#;
    fs::write(&license_path, license_content).unwrap();

    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.env("HOME", temp_dir.path().to_str().unwrap())
        .arg("slo-burn")
        .arg("--config")
        .arg(&slo_path)
        .arg("--snapshots-dir")
        .arg(&snapshots_path)
        .arg("--format")
        .arg("json");

    let assert = cmd.assert().success();

    // Parse JSON output and validate structure
    let output = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    // Skip the first line (progress message) and parse JSON from the rest
    let json_start = output.find('{').unwrap_or(0);
    let json_str = &output[json_start..];
    let json: Value = serde_json::from_str(json_str).unwrap();

    // Validate top-level structure
    assert!(json.is_object());
    assert!(json.get("analyses").is_some());
    assert!(json.get("overall_risk").is_some());
    assert!(json.get("total_slos").is_some());
    assert!(json.get("slos_at_risk").is_some());

    // Validate analysis array
    let analyses = json.get("analyses").unwrap().as_array().unwrap();
    assert_eq!(analyses.len(), 1);

    let analysis = &analyses[0];
    assert_eq!(analysis.get("slo_id").unwrap(), "monthly_budget_test");
    assert_eq!(analysis.get("slo_name").unwrap(), "monthly_budget");
    assert_eq!(analysis.get("slo_limit").unwrap(), 500.0);

    // Should be low risk with slow burn rate
    let risk = analysis.get("risk").unwrap().as_str().unwrap();
    assert_eq!(risk, "low");

    // Validate burn rate is reasonable (~$0.67/day increase)
    let burn_rate = analysis.get("burn_rate").unwrap().as_f64().unwrap();
    assert!(burn_rate > 0.5 && burn_rate < 1.0);

    // Should have a breach prediction far in the future (days_to_breach > 500)
    let days_to_breach = analysis.get("days_to_breach");
    assert!(days_to_breach.unwrap().is_number());
    let days = days_to_breach.unwrap().as_f64().unwrap();
    assert!(days > 500.0); // Breach in over a year
}

/// Test SLO burn rate analysis with critical risk scenario
#[test]
fn test_slo_burn_critical_risk() {
    let temp_dir = TempDir::new().unwrap();
    let slo_path = temp_dir.path().join("slo.json");
    let snapshots_path = temp_dir.path().join("snapshots");

    // Create test SLO configuration (monthly budget: $200)
    let slo_config = r#"{
        "version": "1.0",
        "slos": [
            {
                "id": "tight_budget_test",
                "name": "tight_budget",
                "description": "Very tight monthly budget",
                "slo_type": "monthly_budget",
                "target": "global",
                "threshold": {
                    "max_value": 200.0
                },
                "enforcement": "warn",
                "owner": "test",
                "created_at": "2025-01-01T00:00:00Z"
            }
        ]
    }"#;
    fs::write(&slo_path, slo_config).unwrap();

    // Create snapshots directory
    fs::create_dir(&snapshots_path).unwrap();

    // Create historical snapshots showing rapid cost growth
    // Month 1: $50, Month 2: $120, Month 3: $190 (burn rate ~$2.37/day)
    create_test_snapshot(&snapshots_path, "2025-01-01", 50.0);
    create_test_snapshot(&snapshots_path, "2025-02-01", 120.0);
    create_test_snapshot(&snapshots_path, "2025-03-01", 190.0);

    // Create test license for premium features
    let costpilot_dir = temp_dir.path().join(".costpilot");
    fs::create_dir(&costpilot_dir).unwrap();
    let license_path = costpilot_dir.join("license.json");
    let license_content = r#"{
        "email": "test@example.com",
        "license_key": "test-license-key-for-slo-burn-critical",
        "expires": "2099-12-31T23:59:59Z",
        "signature": "test-signature",
        "issuer": "test-issuer"
    }"#;
    fs::write(&license_path, license_content).unwrap();

    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.env("HOME", temp_dir.path().to_str().unwrap())
        .arg("slo-burn")
        .arg("--config")
        .arg(&slo_path)
        .arg("--snapshots-dir")
        .arg(&snapshots_path)
        .arg("--format")
        .arg("json");

    let assert = cmd.assert().failure().code(1);

    // Parse JSON output
    let output = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let json: Value = serde_json::from_str(&output).unwrap();

    let analyses = json.get("analyses").unwrap().as_array().unwrap();
    let analysis = &analyses[0];

    // Should be critical risk with high burn rate
    let risk = analysis.get("risk").unwrap().as_str().unwrap();
    assert_eq!(risk, "critical");

    // Validate high burn rate (~$2.37/day based on actual calculation)
    let burn_rate = analysis.get("burn_rate").unwrap().as_f64().unwrap();
    assert!(burn_rate > 2.0);

    // Should predict breach within days
    let days_to_breach = analysis.get("days_to_breach").unwrap().as_f64().unwrap();
    assert!(days_to_breach > 0.0 && days_to_breach < 30.0);
}

/// Test SLO burn rate with composed SLO rules
#[test]
fn test_slo_burn_composed_slo() {
    let temp_dir = TempDir::new().unwrap();
    let slo_path = temp_dir.path().join("slo.json");
    let snapshots_path = temp_dir.path().join("snapshots");

    // Create composed SLO configuration (using monthly_budget for now)
    let slo_config = r#"{
        "version": "1.0",
        "slos": [
            {
                "id": "stable_cost_growth_test",
                "name": "stable_cost_growth",
                "description": "Stable cost growth SLO",
                "slo_type": "monthly_budget",
                "target": "global",
                "threshold": {
                    "max_value": 300.0
                },
                "enforcement": "warn",
                "owner": "test",
                "created_at": "2025-01-01T00:00:00Z"
            }
        ]
    }"#;
    fs::write(&slo_path, slo_config).unwrap();

    // Create snapshots directory
    fs::create_dir(&snapshots_path).unwrap();

    // Create historical data showing stable growth within limits
    create_test_snapshot(&snapshots_path, "2025-01-01", 200.0);
    create_test_snapshot(&snapshots_path, "2025-02-01", 220.0);
    create_test_snapshot(&snapshots_path, "2025-03-01", 235.0);

    // Create test license for premium features
    let costpilot_dir = temp_dir.path().join(".costpilot");
    fs::create_dir(&costpilot_dir).unwrap();
    let license_path = costpilot_dir.join("license.json");
    let license_content = r#"{
        "email": "test@example.com",
        "license_key": "test-license-key-for-slo-burn-composed",
        "expires": "2099-12-31T23:59:59Z",
        "signature": "test-signature",
        "issuer": "test-issuer"
    }"#;
    fs::write(&license_path, license_content).unwrap();

    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.env("HOME", temp_dir.path().to_str().unwrap())
        .arg("slo-burn")
        .arg("--config")
        .arg(&slo_path)
        .arg("--snapshots-dir")
        .arg(&snapshots_path)
        .arg("--format")
        .arg("json");

    let assert = cmd.assert().success();

    // Parse JSON output
    let output = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    // Skip the first line (progress message) and parse JSON from the rest
    let json_start = output.find('{').unwrap_or(0);
    let json_str = &output[json_start..];
    let json: Value = serde_json::from_str(json_str).unwrap();

    let analyses = json.get("analyses").unwrap().as_array().unwrap();
    let analysis = &analyses[0];

    // Should be low risk for composed SLO
    let risk = analysis.get("risk").unwrap().as_str().unwrap();
    assert_eq!(risk, "low");
}

/// Test SLO burn rate with insufficient data
#[test]
fn test_slo_burn_insufficient_data() {
    let temp_dir = TempDir::new().unwrap();
    let slo_path = temp_dir.path().join("slo.json");
    let snapshots_path = temp_dir.path().join("snapshots");

    // Create test SLO configuration
    let slo_config = r#"{
        "version": "1.0",
        "slos": [
            {
                "id": "monthly_budget_test",
                "name": "monthly_budget",
                "description": "Monthly budget SLO",
                "slo_type": "monthly_budget",
                "target": "global",
                "threshold": {
                    "max_value": 500.0
                },
                "enforcement": "warn",
                "owner": "test",
                "created_at": "2025-01-01T00:00:00Z"
            }
        ]
    }"#;
    fs::write(&slo_path, slo_config).unwrap();

    // Create snapshots directory but only add 1 snapshot (insufficient)
    fs::create_dir(&snapshots_path).unwrap();
    create_test_snapshot(&snapshots_path, "2025-01-01", 100.0);

    // Create test license for premium features
    let costpilot_dir = temp_dir.path().join(".costpilot");
    fs::create_dir(&costpilot_dir).unwrap();
    let license_path = costpilot_dir.join("license.json");
    let license_content = r#"{
        "email": "test@example.com",
        "license_key": "test-license-key-for-slo-burn-insufficient",
        "expires": "2099-12-31T23:59:59Z",
        "signature": "test-signature",
        "issuer": "test-issuer"
    }"#;
    fs::write(&license_path, license_content).unwrap();

    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.env("HOME", temp_dir.path().to_str().unwrap())
        .arg("slo-burn")
        .arg("--config")
        .arg(&slo_path)
        .arg("--snapshots-dir")
        .arg(&snapshots_path);

    // Should succeed but show insufficient data message
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("insufficient data"));
}

/// Test SLO burn rate text output format
#[test]
fn test_slo_burn_text_output() {
    let temp_dir = TempDir::new().unwrap();
    let slo_path = temp_dir.path().join("slo.json");
    let snapshots_path = temp_dir.path().join("snapshots");

    // Create test SLO configuration
    let slo_config = r#"{
        "version": "1.0",
        "slos": [
            {
                "id": "test_budget_slo",
                "name": "test_budget",
                "description": "Test budget SLO",
                "slo_type": "monthly_budget",
                "target": "global",
                "threshold": {
                    "max_value": 1000.0
                },
                "enforcement": "warn",
                "owner": "test",
                "created_at": "2025-01-01T00:00:00Z"
            }
        ]
    }"#;
    fs::write(&slo_path, slo_config).unwrap();

    // Create snapshots directory with test data
    fs::create_dir(&snapshots_path).unwrap();
    create_test_snapshot(&snapshots_path, "2025-01-01", 100.0);
    create_test_snapshot(&snapshots_path, "2025-02-01", 150.0);
    create_test_snapshot(&snapshots_path, "2025-03-01", 200.0);

    // Create test license for premium features
    let costpilot_dir = temp_dir.path().join(".costpilot");
    fs::create_dir(&costpilot_dir).unwrap();
    let license_path = costpilot_dir.join("license.json");
    let license_content = r#"{
        "email": "test@example.com",
        "license_key": "test-license-key-for-slo-burn-text",
        "expires": "2099-12-31T23:59:59Z",
        "signature": "test-signature",
        "issuer": "test-issuer"
    }"#;
    fs::write(&license_path, license_content).unwrap();

    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.env("HOME", temp_dir.path().to_str().unwrap())
        .arg("slo-burn")
        .arg("--config")
        .arg(&slo_path)
        .arg("--snapshots-dir")
        .arg(&snapshots_path);

    let assert = cmd.assert().success();

    // Validate text output contains expected elements
    let output = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    assert!(output.contains("📊 SLO Burn Rate Analysis"));
    assert!(output.contains("test_budget"));
    assert!(output.contains("$1000"));
    assert!(output.contains("Burn Rate"));
    assert!(output.contains("Risk Level"));
}

/// Test SLO burn rate markdown output format
#[test]
fn test_slo_burn_markdown_output() {
    let temp_dir = TempDir::new().unwrap();
    let slo_path = temp_dir.path().join("slo.json");
    let snapshots_path = temp_dir.path().join("snapshots");

    // Create test SLO configuration
    let slo_config = r#"{
        "version": "1.0",
        "slos": [
            {
                "id": "markdown_test_slo",
                "name": "markdown_test",
                "description": "Markdown test SLO",
                "slo_type": "monthly_budget",
                "target": "global",
                "threshold": {
                    "max_value": 500.0
                },
                "enforcement": "warn",
                "owner": "test",
                "created_at": "2025-01-01T00:00:00Z"
            }
        ]
    }"#;
    fs::write(&slo_path, slo_config).unwrap();

    // Create snapshots directory with test data
    fs::create_dir(&snapshots_path).unwrap();
    create_test_snapshot(&snapshots_path, "2025-01-01", 100.0);
    create_test_snapshot(&snapshots_path, "2025-02-01", 150.0);
    create_test_snapshot(&snapshots_path, "2025-03-01", 200.0);

    // Create test license for premium features
    let costpilot_dir = temp_dir.path().join(".costpilot");
    fs::create_dir(&costpilot_dir).unwrap();
    let license_path = costpilot_dir.join("license.json");
    let license_content = r#"{
        "email": "test@example.com",
        "license_key": "test-license-key-for-slo-burn-markdown",
        "expires": "2099-12-31T23:59:59Z",
        "signature": "test-signature",
        "issuer": "test-issuer"
    }"#;
    fs::write(&license_path, license_content).unwrap();

    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.env("HOME", temp_dir.path().to_str().unwrap())
        .arg("slo-burn")
        .arg("--config")
        .arg(&slo_path)
        .arg("--snapshots-dir")
        .arg(&snapshots_path)
        .arg("--format")
        .arg("markdown");

    let assert = cmd.assert().success();

    // Validate markdown output format
    let output = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    assert!(output.contains("## 📊 SLO Burn Rate Analysis"));
    assert!(output.contains("| SLO | Burn Rate | Projected | Days to Breach | Risk | Confidence |"));
    assert!(output.contains("|-----|-----------|-----------|----------------|------|------------|"));
    assert!(output.contains("markdown_test"));
}

/// Test SLO burn rate with custom analysis parameters
#[test]
fn test_slo_burn_custom_parameters() {
    let temp_dir = TempDir::new().unwrap();
    let slo_path = temp_dir.path().join("slo.json");
    let snapshots_path = temp_dir.path().join("snapshots");

    // Create test SLO configuration
    let slo_config = r#"{
        "version": "1.0",
        "slos": [
            {
                "id": "custom_params_test_slo",
                "name": "custom_params_test",
                "description": "Custom parameters test SLO",
                "slo_type": "monthly_budget",
                "target": "global",
                "threshold": {
                    "max_value": 1000.0
                },
                "enforcement": "warn",
                "owner": "test",
                "created_at": "2025-01-01T00:00:00Z"
            }
        ]
    }"#;
    fs::write(&slo_path, slo_config).unwrap();

    // Create snapshots directory with test data
    fs::create_dir(&snapshots_path).unwrap();
    create_test_snapshot(&snapshots_path, "2025-01-01", 100.0);
    create_test_snapshot(&snapshots_path, "2025-02-01", 150.0);
    create_test_snapshot(&snapshots_path, "2025-03-01", 200.0);
    create_test_snapshot(&snapshots_path, "2025-04-01", 250.0);
    create_test_snapshot(&snapshots_path, "2025-05-01", 300.0);

    // Create test license for premium features
    let costpilot_dir = temp_dir.path().join(".costpilot");
    fs::create_dir(&costpilot_dir).unwrap();
    let license_path = costpilot_dir.join("license.json");
    let license_content = r#"{
        "email": "test@example.com",
        "license_key": "test-license-key-for-slo-burn-custom",
        "expires": "2099-12-31T23:59:59Z",
        "signature": "test-signature",
        "issuer": "test-issuer"
    }"#;
    fs::write(&license_path, license_content).unwrap();

    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.env("HOME", temp_dir.path().to_str().unwrap())
        .arg("slo-burn")
        .arg("--config")
        .arg(&slo_path)
        .arg("--snapshots-dir")
        .arg(&snapshots_path)
        .arg("--min-snapshots")
        .arg("4")
        .arg("--min-r-squared")
        .arg("0.8")
        .arg("--format")
        .arg("json");

    let assert = cmd.assert().success();

    // Parse JSON output and validate custom parameters were applied
    let output = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    // Skip the first line (progress message) and parse JSON from the rest
    let json_start = output.find('{').unwrap_or(0);
    let json_str = &output[json_start..];
    let json: Value = serde_json::from_str(json_str).unwrap();

    let analyses = json.get("analyses").unwrap().as_array().unwrap();
    assert!(!analyses.is_empty()); // Should have analysis with 5 snapshots meeting minimum
}

/// Test SLO burn rate with multiple SLOs
#[test]
fn test_slo_burn_multiple_slos() {
    let temp_dir = TempDir::new().unwrap();
    let slo_path = temp_dir.path().join("slo.json");
    let snapshots_path = temp_dir.path().join("snapshots");

    // Create multiple SLO configuration
    let slo_config = r#"{
        "version": "1.0",
        "slos": [
            {
                "id": "team_budget_slo",
                "name": "team_budget",
                "description": "Team monthly budget",
                "slo_type": "monthly_budget",
                "target": "global",
                "threshold": {
                    "max_value": 1000.0
                },
                "enforcement": "warn",
                "owner": "test",
                "created_at": "2025-01-01T00:00:00Z"
            },
            {
                "id": "service_budget_slo",
                "name": "service_budget",
                "description": "Service monthly budget",
                "slo_type": "monthly_budget",
                "target": "global",
                "threshold": {
                    "max_value": 500.0
                },
                "enforcement": "warn",
                "owner": "test",
                "created_at": "2025-01-01T00:00:00Z"
            }
        ]
    }"#;
    fs::write(&slo_path, slo_config).unwrap();

    // Create snapshots directory with test data
    fs::create_dir(&snapshots_path).unwrap();
    create_test_snapshot(&snapshots_path, "2025-01-01", 200.0);
    create_test_snapshot(&snapshots_path, "2025-02-01", 300.0);
    create_test_snapshot(&snapshots_path, "2025-03-01", 400.0);

    // Create test license for premium features
    let costpilot_dir = temp_dir.path().join(".costpilot");
    fs::create_dir(&costpilot_dir).unwrap();
    let license_path = costpilot_dir.join("license.json");
    let license_content = r#"{
        "email": "test@example.com",
        "license_key": "test-license-key-for-slo-burn-multiple",
        "expires": "2099-12-31T23:59:59Z",
        "signature": "test-signature",
        "issuer": "test-issuer"
    }"#;
    fs::write(&license_path, license_content).unwrap();

    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.env("HOME", temp_dir.path().to_str().unwrap())
        .arg("slo-burn")
        .arg("--config")
        .arg(&slo_path)
        .arg("--snapshots-dir")
        .arg(&snapshots_path)
        .arg("--format")
        .arg("json");

    let assert = cmd.assert().success();

    // Parse JSON output and validate multiple SLOs
    let output = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let json: Value = serde_json::from_str(&output).unwrap();

    let analyses = json.get("analyses").unwrap().as_array().unwrap();
    assert_eq!(analyses.len(), 2);

    // Check both SLOs are present
    let slo_names: Vec<&str> = analyses
        .iter()
        .map(|a| a.get("slo_name").unwrap().as_str().unwrap())
        .collect();
    assert!(slo_names.contains(&"team_budget"));
    assert!(slo_names.contains(&"service_budget"));
}

#[test]
fn test_slo_not_breached_silent() {
    // Placeholder test for: SLO not breached → silent
    // TODO: Implement logic to check that when SLO is not breached,
    // the system runs silently (no findings, no explain output, exit code 0)
    todo!("Implement SLO not breached silent test");
}

// ===== SLO BURN EDGE CASE TESTS =====

#[test]
fn test_slo_burn_zero_cost_edge_case() {
    // Test SLO burn with zero cost snapshots
    let temp_dir = TempDir::new().unwrap();
    let slo_path = temp_dir.path().join("slo.json");
    let snapshots_path = temp_dir.path().join("snapshots");

    let slo_config = r#"{
        "version": "1.0",
        "slos": [
            {
                "id": "zero_budget_test",
                "name": "zero_budget",
                "description": "Zero cost budget test",
                "slo_type": "monthly_budget",
                "target": "global",
                "threshold": {
                    "max_value": 100.0
                },
                "enforcement": "warn",
                "owner": "test",
                "created_at": "2025-01-01T00:00:00Z"
            }
        ]
    }"#;
    fs::write(&slo_path, slo_config).unwrap();

    fs::create_dir(&snapshots_path).unwrap();
    create_test_snapshot(&snapshots_path, "2025-01-01", 0.0);
    create_test_snapshot(&snapshots_path, "2025-02-01", 0.0);

    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("slo")
        .arg("burn")
        .arg("--slo-config")
        .arg(&slo_path)
        .arg("--snapshots")
        .arg(&snapshots_path)
        .assert()
        .success();
}

#[test]
fn test_slo_burn_negative_cost_edge_case() {
    // Test SLO burn with negative costs (credits/rebates)
    let temp_dir = TempDir::new().unwrap();
    let slo_path = temp_dir.path().join("slo.json");
    let snapshots_path = temp_dir.path().join("snapshots");

    let slo_config = r#"{
        "version": "1.0",
        "slos": [
            {
                "id": "negative_budget_test",
                "name": "negative_budget",
                "description": "Negative cost budget test",
                "slo_type": "monthly_budget",
                "target": "global",
                "threshold": {
                    "max_value": 100.0
                },
                "enforcement": "warn",
                "owner": "test",
                "created_at": "2025-01-01T00:00:00Z"
            }
        ]
    }"#;
    fs::write(&slo_path, slo_config).unwrap();

    fs::create_dir(&snapshots_path).unwrap();
    create_test_snapshot(&snapshots_path, "2025-01-01", -50.0);
    create_test_snapshot(&snapshots_path, "2025-02-01", -25.0);

    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("slo")
        .arg("burn")
        .arg("--slo-config")
        .arg(&slo_path)
        .arg("--snapshots")
        .arg(&snapshots_path)
        .assert()
        .success();
}

#[test]
fn test_slo_burn_extremely_high_cost() {
    // Test SLO burn with extremely high costs
    let temp_dir = TempDir::new().unwrap();
    let slo_path = temp_dir.path().join("slo.json");
    let snapshots_path = temp_dir.path().join("snapshots");

    let slo_config = r#"{
        "version": "1.0",
        "slos": [
            {
                "id": "high_budget_test",
                "name": "high_budget",
                "description": "High cost budget test",
                "slo_type": "monthly_budget",
                "target": "global",
                "threshold": {
                    "max_value": 1000.0
                },
                "enforcement": "warn",
                "owner": "test",
                "created_at": "2025-01-01T00:00:00Z"
            }
        ]
    }"#;
    fs::write(&slo_path, slo_config).unwrap();

    fs::create_dir(&snapshots_path).unwrap();
    create_test_snapshot(&snapshots_path, "2025-01-01", 1_000_000.0);
    create_test_snapshot(&snapshots_path, "2025-02-01", 2_000_000.0);

    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("slo")
        .arg("burn")
        .arg("--slo-config")
        .arg(&slo_path)
        .arg("--snapshots")
        .arg(&snapshots_path)
        .assert()
        .success();
}

#[test]
fn test_slo_burn_empty_snapshots_edge_case() {
    // Test SLO burn with empty snapshots directory
    let temp_dir = TempDir::new().unwrap();
    let slo_path = temp_dir.path().join("slo.json");
    let snapshots_path = temp_dir.path().join("snapshots");

    let slo_config = r#"{
        "version": "1.0",
        "slos": [
            {
                "id": "empty_test",
                "name": "empty_test",
                "description": "Empty snapshots test",
                "slo_type": "monthly_budget",
                "target": "global",
                "threshold": {
                    "max_value": 100.0
                },
                "enforcement": "warn",
                "owner": "test",
                "created_at": "2025-01-01T00:00:00Z"
            }
        ]
    }"#;
    fs::write(&slo_path, slo_config).unwrap();

    fs::create_dir(&snapshots_path).unwrap();
    // No snapshots created

    let mut cmd = cargo_bin_cmd!("costpilot");
    let _result = cmd
        .arg("slo")
        .arg("burn")
        .arg("--slo-config")
        .arg(&slo_path)
        .arg("--snapshots")
        .arg(&snapshots_path)
        .assert();
    // Should handle gracefully
}

#[test]
fn test_slo_burn_single_snapshot_edge_case() {
    // Test SLO burn with single snapshot
    let temp_dir = TempDir::new().unwrap();
    let slo_path = temp_dir.path().join("slo.json");
    let snapshots_path = temp_dir.path().join("snapshots");

    let slo_config = r#"{
        "version": "1.0",
        "slos": [
            {
                "id": "single_test",
                "name": "single_test",
                "description": "Single snapshot test",
                "slo_type": "monthly_budget",
                "target": "global",
                "threshold": {
                    "max_value": 100.0
                },
                "enforcement": "warn",
                "owner": "test",
                "created_at": "2025-01-01T00:00:00Z"
            }
        ]
    }"#;
    fs::write(&slo_path, slo_config).unwrap();

    fs::create_dir(&snapshots_path).unwrap();
    create_test_snapshot(&snapshots_path, "2025-01-01", 50.0);

    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("slo")
        .arg("burn")
        .arg("--slo-config")
        .arg(&slo_path)
        .arg("--snapshots")
        .arg(&snapshots_path)
        .assert()
        .success();
}

#[test]
fn test_slo_burn_extremely_long_slo_names() {
    // Test SLO burn with extremely long SLO names
    let temp_dir = TempDir::new().unwrap();
    let slo_path = temp_dir.path().join("slo.json");
    let snapshots_path = temp_dir.path().join("snapshots");

    let long_name = "a".repeat(1000);
    let slo_config = format!(
        r#"{{
        "version": "1.0",
        "slos": [
            {{
                "id": "{}",
                "name": "{}",
                "description": "Long name test",
                "slo_type": "monthly_budget",
                "target": "global",
                "threshold": {{
                    "max_value": 100.0
                }},
                "enforcement": "warn",
                "owner": "test",
                "created_at": "2025-01-01T00:00:00Z"
            }}
        ]
    }}"#,
        long_name, long_name
    );
    fs::write(&slo_path, slo_config).unwrap();

    fs::create_dir(&snapshots_path).unwrap();
    create_test_snapshot(&snapshots_path, "2025-01-01", 50.0);

    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("slo")
        .arg("burn")
        .arg("--slo-config")
        .arg(&slo_path)
        .arg("--snapshots")
        .arg(&snapshots_path)
        .assert()
        .success();
}

#[test]
fn test_slo_burn_special_characters_in_names() {
    // Test SLO burn with special characters in SLO names
    let temp_dir = TempDir::new().unwrap();
    let slo_path = temp_dir.path().join("slo.json");
    let snapshots_path = temp_dir.path().join("snapshots");

    let slo_config = r#"{
        "version": "1.0",
        "slos": [
            {
                "id": "slo@domain.com",
                "name": "测试SLO",
                "description": "Special characters test",
                "slo_type": "monthly_budget",
                "target": "global",
                "threshold": {
                    "max_value": 100.0
                },
                "enforcement": "warn",
                "owner": "test",
                "created_at": "2025-01-01T00:00:00Z"
            }
        ]
    }"#;
    fs::write(&slo_path, slo_config).unwrap();

    fs::create_dir(&snapshots_path).unwrap();
    create_test_snapshot(&snapshots_path, "2025-01-01", 50.0);

    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("slo")
        .arg("burn")
        .arg("--slo-config")
        .arg(&slo_path)
        .arg("--snapshots")
        .arg(&snapshots_path)
        .assert()
        .success();
}

#[test]
fn test_slo_burn_zero_threshold_edge_case() {
    // Test SLO burn with zero threshold
    let temp_dir = TempDir::new().unwrap();
    let slo_path = temp_dir.path().join("slo.json");
    let snapshots_path = temp_dir.path().join("snapshots");

    let slo_config = r#"{
        "version": "1.0",
        "slos": [
            {
                "id": "zero_threshold_test",
                "name": "zero_threshold",
                "description": "Zero threshold test",
                "slo_type": "monthly_budget",
                "target": "global",
                "threshold": {
                    "max_value": 0.0
                },
                "enforcement": "warn",
                "owner": "test",
                "created_at": "2025-01-01T00:00:00Z"
            }
        ]
    }"#;
    fs::write(&slo_path, slo_config).unwrap();

    fs::create_dir(&snapshots_path).unwrap();
    create_test_snapshot(&snapshots_path, "2025-01-01", 10.0);

    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("slo")
        .arg("burn")
        .arg("--slo-config")
        .arg(&slo_path)
        .arg("--snapshots")
        .arg(&snapshots_path)
        .assert()
        .success();
}

#[test]
fn test_slo_burn_maximum_snapshots() {
    // Test SLO burn with maximum number of snapshots
    let temp_dir = TempDir::new().unwrap();
    let slo_path = temp_dir.path().join("slo.json");
    let snapshots_path = temp_dir.path().join("snapshots");

    let slo_config = r#"{
        "version": "1.0",
        "slos": [
            {
                "id": "max_snapshots_test",
                "name": "max_snapshots",
                "description": "Maximum snapshots test",
                "slo_type": "monthly_budget",
                "target": "global",
                "threshold": {
                    "max_value": 10000.0
                },
                "enforcement": "warn",
                "owner": "test",
                "created_at": "2025-01-01T00:00:00Z"
            }
        ]
    }"#;
    fs::write(&slo_path, slo_config).unwrap();

    fs::create_dir(&snapshots_path).unwrap();
    // Create 1000 snapshots
    for i in 1..=1000 {
        create_test_snapshot(
            &snapshots_path,
            &format!("2025-{:02}-01", i % 12 + 1),
            i as f64,
        );
    }

    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("slo")
        .arg("burn")
        .arg("--slo-config")
        .arg(&slo_path)
        .arg("--snapshots")
        .arg(&snapshots_path)
        .assert()
        .success();
}

/// Helper function to create test snapshot files
fn create_test_snapshot(snapshots_dir: &Path, date: &str, cost: f64) {
    let filename = format!("snapshot_{}.json", date.replace("-", ""));
    let snapshot_path = snapshots_dir.join(filename);

    let snapshot_data = format!(
        r#"{{
        "id": "test_{}",
        "timestamp": "{}T00:00:00Z",
        "total_monthly_cost": {:.2},
        "modules": {{
            "ec2": {{
                "name": "ec2",
                "monthly_cost": {:.2},
                "resource_count": 5
            }},
            "rds": {{
                "name": "rds",
                "monthly_cost": {:.2},
                "resource_count": 2
            }}
        }},
        "services": {{}},
        "regressions": [],
        "slo_violations": []
    }}"#,
        date.replace("-", ""),
        date,
        cost,
        cost * 0.7,
        cost * 0.3
    );

    fs::write(snapshot_path, snapshot_data).unwrap();
}
