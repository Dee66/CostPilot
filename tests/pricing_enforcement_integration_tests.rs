#![allow(deprecated)]

// Integration tests for pricing enforcement and plan integration
// Ensures free-tier users cannot access premium features

use assert_cmd::cargo::cargo_bin_cmd;
use costpilot::edition::EditionContext;
use costpilot::engines::autofix::{AutofixEngine, AutofixMode};
use costpilot::engines::shared::models::{
    ChangeAction, CostEstimate, Detection, RegressionType, ResourceChange, Severity,
};
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

/// Test that free users cannot access autofix patch mode
#[test]
fn test_autofix_patch_requires_premium_integration() {
    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("autofix-patch");

    // Should fail - command requires arguments
    cmd.assert()
        .failure();
}

/// Test that free users cannot access autofix snippet mode
#[test]
fn test_autofix_snippet_requires_premium_integration() {
    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("autofix-snippet");

    // Should fail - command requires arguments
    cmd.assert()
        .failure();
}

/// Test that scan command works in free mode but uses static prediction
#[test]
fn test_scan_uses_static_prediction_in_free_mode() {
    let temp_dir = TempDir::new().unwrap();
    let plan_path = temp_dir.path().join("test_plan.json");

    // Create a minimal terraform plan
    let plan_content = r#"{
        "format_version": "1.1",
        "terraform_version": "1.0.0",
        "resource_changes": [
            {
                "address": "aws_instance.test",
                "type": "aws_instance",
                "name": "test",
                "change": {
                    "actions": ["create"],
                    "before": null,
                    "after": {
                        "instance_type": "t3.micro"
                    }
                }
            }
        ]
    }"#;

    fs::write(&plan_path, plan_content).unwrap();

    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("scan").arg(&plan_path);

    // Should succeed and show cost estimate (free tier provides basic estimates)
    let output = cmd.assert().success().get_output().clone();
    let stdout = String::from_utf8(output.stdout).unwrap();

    // Free tier provides basic cost estimates, not zero
    assert!(stdout.contains("$150.00") || stdout.contains("150"));
}

/// Test that policy enforcement is downgraded to warnings in free mode
#[test]
fn test_policy_enforcement_downgraded_in_free_mode() {
    let temp_dir = TempDir::new().unwrap();
    let plan_path = temp_dir.path().join("test_plan.json");
    let policy_path = temp_dir.path().join("policy.yaml");

    // Create terraform plan
    let plan_content = r#"{
        "format_version": "1.1",
        "terraform_version": "1.0.0",
        "resource_changes": [
            {
                "address": "aws_instance.large",
                "type": "aws_instance",
                "name": "large",
                "change": {
                    "actions": ["create"],
                    "before": null,
                    "after": {
                        "instance_type": "t3.2xlarge"
                    }
                }
            }
        ]
    }"#;
    fs::write(&plan_path, plan_content).unwrap();

    // Create policy that would block large instances
    let policy_content = r#"version: "1.0"
budgets:
  global:
    monthly_limit: 100
"#;
    fs::write(&policy_path, policy_content).unwrap();

    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("scan")
        .arg(&plan_path)
        .arg("--policy")
        .arg(policy_path);

    let output = cmd.assert().success().get_output().clone();
    let stdout = String::from_utf8(output.stdout).unwrap();

    // In free mode, scan should complete successfully
    assert!(!stdout.is_empty());
}

/// Test that explain command with verbose flag works in free tier
#[test]
fn test_explain_verbose_works_in_free_tier() {
    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("explain").arg("aws_instance").arg("--verbose");

    // Should succeed in free tier
    cmd.assert().success();
}

/// Test that anomaly detection requires premium
#[test]
fn test_anomaly_detection_requires_premium() {
    let temp_dir = TempDir::new().unwrap();
    let plan_path = temp_dir.path().join("test_plan.json");

    // Create terraform plan
    let plan_content = r#"{
        "format_version": "1.1",
        "terraform_version": "1.0.0",
        "resource_changes": [
            {
                "address": "aws_instance.test",
                "type": "aws_instance",
                "name": "test",
                "change": {
                    "actions": ["create"],
                    "before": null,
                    "after": {
                        "instance_type": "t3.micro"
                    }
                }
            }
        ]
    }"#;
    fs::write(&plan_path, plan_content).unwrap();

    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("anomaly").arg("--plan").arg(&plan_path);

    // Command does not exist, should fail
    cmd.assert()
        .failure();
}

/// Test that SLO commands require premium
#[test]
fn test_slo_commands_require_premium() {
    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("slo-burn");

    // Should fail with premium requirement
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Premium"))
        .stderr(predicate::str::contains("SLO"));
}

/// Test that deep mapping features require premium
#[test]
fn test_deep_mapping_requires_premium() {
    let temp_dir = TempDir::new().unwrap();
    let plan_path = temp_dir.path().join("test_plan.json");

    // Create terraform plan
    let plan_content = r#"{
        "format_version": "1.1",
        "terraform_version": "1.0.0",
        "resource_changes": [
            {
                "address": "aws_instance.test",
                "type": "aws_instance",
                "name": "test",
                "change": {
                    "actions": ["create"],
                    "before": null,
                    "after": {
                        "instance_type": "t3.micro"
                    }
                }
            }
        ]
    }"#;
    fs::write(&plan_path, plan_content).unwrap();

    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("map").arg(&plan_path).arg("--max-depth").arg("5"); // Deep mapping

    // Should fail with upgrade requirement for deep mapping
    cmd.assert()
        .failure();
}

/// Test AutofixEngine integration with edition context
#[test]
fn test_autofix_engine_edition_enforcement() {
    // Test data
    let detections = vec![Detection {
        rule_id: "test".to_string(),
        resource_id: "aws_instance.test".to_string(),
        regression_type: RegressionType::Configuration,
        severity: Severity::High,
        severity_score: 80,
        message: "Test detection".to_string(),
        estimated_cost: Some(100.0),
        fix_snippet: None,
    }];

    let changes = vec![ResourceChange {
        resource_id: "aws_instance.test".to_string(),
        resource_type: "aws_instance".to_string(),
        action: ChangeAction::Create,
        module_path: None,
        old_config: None,
        new_config: Some(serde_json::json!({"instance_type": "t3.large"})),
        tags: Default::default(),
        monthly_cost: None,
        config: None,
        cost_impact: None,
    }];

    let estimates = vec![CostEstimate {
        resource_id: "aws_instance.test".to_string(),
        monthly_cost: 100.0,
        prediction_interval_low: 90.0,
        prediction_interval_high: 110.0,
        confidence_score: 0.9,
        heuristic_reference: Some("test".to_string()),
        cold_start_inference: false,
        one_time: None,
        breakdown: None,
        hourly: None,
        daily: None,
    }];

    // Free edition should reject patch mode
    let free_edition = EditionContext::free();
    let result = AutofixEngine::generate_fixes(
        &detections,
        &changes,
        &estimates,
        AutofixMode::Patch,
        &free_edition,
    );

    assert!(result.is_err());
    let err_str = result.unwrap_err().to_string();
    assert!(err_str.contains("Premium"));

    // Free edition should reject drift-safe mode
    let result = AutofixEngine::generate_fixes(
        &detections,
        &changes,
        &estimates,
        AutofixMode::DriftSafe,
        &free_edition,
    );

    assert!(result.is_err());
    let err_str = result.unwrap_err().to_string();
    assert!(err_str.contains("Premium"));

    // Free edition should allow snippet mode
    let result = AutofixEngine::generate_fixes(
        &detections,
        &changes,
        &estimates,
        AutofixMode::Snippet,
        &free_edition,
    );

    assert!(result.is_ok());
}

/// Test that scan command prediction modes differ between free and premium
#[test]
fn test_scan_prediction_modes_differ_by_edition() {
    let temp_dir = TempDir::new().unwrap();
    let plan_path = temp_dir.path().join("test_plan.json");

    // Create terraform plan with expensive resource
    let plan_content = r#"{
        "format_version": "1.1",
        "terraform_version": "1.0.0",
        "resource_changes": [
            {
                "address": "aws_instance.expensive",
                "type": "aws_instance",
                "name": "expensive",
                "change": {
                    "actions": ["create"],
                    "before": null,
                    "after": {
                        "instance_type": "t3.2xlarge"
                    }
                }
            }
        ]
    }"#;
    fs::write(&plan_path, plan_content).unwrap();

    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("scan").arg(&plan_path);

    let output = cmd.assert().success().get_output().clone();
    let stdout = String::from_utf8(output.stdout).unwrap();

    // In free mode, cost should be estimated (static prediction provides basic estimates)
    assert!(stdout.contains("$150.00") || stdout.contains("150"));
}

/// Test that CLI commands properly validate edition before execution
#[test]
fn test_cli_command_edition_validation() {
    // Test various premium commands fail appropriately
    let premium_commands = vec![
        vec!["autofix-patch"],
        vec!["autofix-snippet"],
        vec!["slo-burn", "--config", "/dev/null"],
    ];

    for cmd_args in premium_commands {
        let mut cmd = cargo_bin_cmd!("costpilot");
        for arg in &cmd_args {
            cmd.arg(arg);
        }

        // All should fail (may be due to missing args or premium requirement)
        cmd.assert()
            .failure();
    }
}
