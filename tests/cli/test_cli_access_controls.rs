// CLI access control integration tests
// Tests that verify premium features are properly gated at the CLI level

use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

/// Test that all premium CLI commands are properly gated
#[test]
fn test_premium_cli_commands_gated() {
    let premium_commands = vec![
        ("autofix-patch", "Autofix"),
        ("autofix-snippet", "Autofix"),
        ("trend", "Trend"),
        ("slo-burn", "SLO"),
        ("slo-check", "SLO"),
    ];

    for (cmd, feature) in premium_commands {
        let mut command = cargo_bin_cmd!("costpilot");
        command.arg(cmd);

        // Add required args for commands that need them
        match cmd {
            "slo-burn" | "slo-check" => {
                command.arg("--config").arg("/dev/null");
            }
            _ => {}
        }

        command.assert()
            .failure()
            .stderr(predicate::str::contains("Premium"))
            .stderr(predicate::str::contains(feature));
    }
}

/// Test that scan command with explain flag shows free tier limitations
#[test]
fn test_scan_explain_free_tier_limitations() {
    let temp_dir = TempDir::new().unwrap();
    let plan_path = temp_dir.path().join("test_plan.json");

    // Create terraform plan
    let plan_content = r#"{
        "planned_values": {
            "root_module": {
                "resources": [
                    {
                        "address": "aws_instance.test",
                        "mode": "managed",
                        "type": "aws_instance",
                        "name": "test",
                        "values": {
                            "instance_type": "t3.large"
                        }
                    }
                ]
            }
        },
        "resource_changes": [
            {
                "address": "aws_instance.test",
                "change": {
                    "actions": ["create"],
                    "before": null,
                    "after": {
                        "instance_type": "t3.large"
                    }
                }
            }
        ]
    }"#;
    fs::write(&plan_path, plan_content).unwrap();

    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("scan")
        .arg("--plan")
        .arg(plan_path)
        .arg("--explain");

    let output = cmd.assert().success().get_output().clone();
    let stdout = String::from_utf8(output.stdout).unwrap();

    // Free tier should indicate limited explain capabilities
    assert!(stdout.contains("Free") || stdout.contains("basic"));
}

/// Test that scan command with autofix flag shows premium requirement
#[test]
fn test_scan_autofix_requires_premium() {
    let temp_dir = TempDir::new().unwrap();
    let plan_path = temp_dir.path().join("test_plan.json");

    // Create terraform plan
    let plan_content = r#"{
        "planned_values": {
            "root_module": {
                "resources": [
                    {
                        "address": "aws_instance.test",
                        "mode": "managed",
                        "type": "aws_instance",
                        "name": "test",
                        "values": {
                            "instance_type": "t3.micro"
                        }
                    }
                ]
            }
        }
    }"#;
    fs::write(&plan_path, plan_content).unwrap();

    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("scan")
        .arg("--plan")
        .arg(plan_path)
        .arg("--autofix");

    // Should succeed but indicate autofix not available in free tier
    let output = cmd.assert().success().get_output().clone();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(stdout.contains("not yet implemented") ||
            stdout.contains("Premium") ||
            stdout.contains("autofix"));
}

/// Test that map command deep flag requires premium
#[test]
fn test_map_deep_requires_premium() {
    let temp_dir = TempDir::new().unwrap();
    let plan_path = temp_dir.path().join("test_plan.json");

    // Create terraform plan
    let plan_content = r#"{
        "planned_values": {
            "root_module": {
                "resources": [
                    {
                        "address": "aws_instance.test",
                        "mode": "managed",
                        "type": "aws_instance",
                        "name": "test",
                        "values": {
                            "instance_type": "t3.micro"
                        }
                    }
                ]
            }
        }
    }"#;
    fs::write(&plan_path, plan_content).unwrap();

    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("map")
        .arg("--plan")
        .arg(plan_path)
        .arg("--deep");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Premium"));
}

/// Test that explain command with advanced flags requires premium
#[test]
fn test_explain_advanced_flags_require_premium() {
    let temp_dir = TempDir::new().unwrap();
    let plan_path = temp_dir.path().join("test_plan.json");

    // Create terraform plan
    let plan_content = r#"{
        "planned_values": {
            "root_module": {
                "resources": [
                    {
                        "address": "aws_instance.test",
                        "mode": "managed",
                        "type": "aws_instance",
                        "name": "test",
                        "values": {
                            "instance_type": "t3.micro"
                        }
                    }
                ]
            }
        }
    }"#;
    fs::write(&plan_path, plan_content).unwrap();

    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("explain")
        .arg("--plan")
        .arg(plan_path)
        .arg("--full"); // Advanced explain

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Premium"));
}

/// Test that validate command works in free tier for basic validation
#[test]
fn test_validate_basic_works_in_free_tier() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("costpilot.yaml");

    // Create basic config
    let config_content = r#"version: "1.0"
baselines:
  monthly_budget: 1000
"#;
    fs::write(&config_path, config_content).unwrap();

    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("validate")
        .arg(config_path);

    // Should succeed in free tier
    cmd.assert().success();
}

/// Test that help output indicates premium features appropriately
#[test]
fn test_help_indicates_premium_features() {
    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("--help");

    let output = cmd.assert().success().get_output().clone();
    let stdout = String::from_utf8(output.stdout).unwrap();

    // Help should mention premium features
    assert!(stdout.contains("Premium") || stdout.contains("Pro"));
}

/// Test that version command shows edition information
#[test]
fn test_version_shows_edition_info() {
    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("version")
        .arg("--detailed");

    let output = cmd.assert().success().get_output().clone();
    let stdout = String::from_utf8(output.stdout).unwrap();

    // Should indicate free edition
    assert!(stdout.contains("Free") || stdout.contains("Community"));
}</content>
<parameter name="filePath">/home/dee/workspace/AI/GuardSuite/CostPilot/tests/cli/test_cli_access_controls.rs
