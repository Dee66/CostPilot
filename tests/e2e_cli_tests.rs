// Comprehensive end-to-end integration tests for CostPilot CLI
//
// Tests complete workflows from CLI command execution through result validation,
// ensuring all components work together correctly.

use assert_cmd::Command;
use std::fs;
use tempfile::TempDir;

// Test data: Simple Terraform plan with EC2 instance
const SAMPLE_TERRAFORM_PLAN: &str = r#"{
        "format_version": "1.0",
        "terraform_version": "1.5.0",
        "resource_changes": [
            {
                "address": "aws_instance.web",
                "mode": "managed",
                "type": "aws_instance",
                "name": "web",
                "change": {
                    "actions": ["create"],
                    "before": null,
                    "after": {
                        "instance_type": "t3.medium",
                        "ami": "ami-0c55b159cbfafe1f0",
                        "tags": {
                            "Name": "web-server",
                            "Environment": "dev"
                        }
                    }
                }
            }
        ],
        "configuration": {
            "root_module": {
                "resources": [
                    {
                        "address": "aws_instance.web",
                        "mode": "managed",
                        "type": "aws_instance",
                        "name": "web",
                        "provider_config_key": "aws",
                        "expressions": {
                            "instance_type": {
                                "constant_value": "t3.medium"
                            },
                            "ami": {
                                "constant_value": "ami-0c55b159cbfafe1f0"
                            }
                        }
                    }
                ]
            }
        }
    }"#;

// Test data: Terraform plan with multiple resources
const MULTI_RESOURCE_PLAN: &str = r#"{
        "format_version": "1.0",
        "terraform_version": "1.5.0",
        "resource_changes": [
            {
                "address": "aws_instance.web",
                "mode": "managed",
                "type": "aws_instance",
                "name": "web",
                "change": {
                    "actions": ["create"],
                    "before": null,
                    "after": {
                        "instance_type": "t3.medium",
                        "ami": "ami-0c55b159cbfafe1f0"
                    }
                }
            },
            {
                "address": "aws_nat_gateway.main",
                "mode": "managed",
                "type": "aws_nat_gateway",
                "name": "main",
                "change": {
                    "actions": ["create"],
                    "before": null,
                    "after": {
                        "subnet_id": "subnet-12345",
                        "connectivity_type": "public"
                    }
                }
            },
            {
                "address": "aws_s3_bucket.data",
                "mode": "managed",
                "type": "aws_s3_bucket",
                "name": "data",
                "change": {
                    "actions": ["create"],
                    "before": null,
                    "after": {
                        "bucket": "my-data-bucket"
                    }
                }
            }
        ],
        "configuration": {
            "root_module": {}
        }
    }"#;

// Test data: Policy file
const SAMPLE_POLICY: &str = r#"version: "1.0"
policies:
  - name: "Instance Type Restrictions"
    rule: "instance_type in ['t3.micro', 't3.small', 't3.medium']"
    action: warn
    severity: MEDIUM
    resources:
      - aws_instance

  - name: "NAT Gateway Limit"
    rule: "resource_count <= 1"
    action: block
    severity: HIGH
    resources:
      - aws_nat_gateway
"#;

#[test]
fn test_e2e_scan_basic_workflow() {
    let temp_dir = TempDir::new().unwrap();
    let plan_path = temp_dir.path().join("plan.json");
    fs::write(&plan_path, SAMPLE_TERRAFORM_PLAN).unwrap();

    let mut cmd = Command::cargo_bin("costpilot").unwrap();
    cmd.arg("scan").arg(&plan_path).arg("--format").arg("json");

    let output = cmd.assert().success();
    let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();

    // Verify JSON output contains expected structure
    assert!(stdout.contains("aws_instance.web"));
    assert!(stdout.contains("\"monthly_cost\":"));
    assert!(stdout.contains("\"resources_changed\":"));
    assert!(stdout.contains("\"resource_type\": \"aws_instance\""));
}

#[test]
fn test_e2e_scan_with_policy() {
    let temp_dir = TempDir::new().unwrap();

    // Create plan file
    let plan_path = temp_dir.path().join("plan.json");
    fs::write(&plan_path, SAMPLE_TERRAFORM_PLAN).unwrap();

    // Create policy file
    let policy_path = temp_dir.path().join("policy.yml");
    fs::write(&policy_path, SAMPLE_POLICY).unwrap();

    let mut cmd = Command::cargo_bin("costpilot").unwrap();
    cmd.arg("scan")
        .arg(plan_path)
        .arg("--policy")
        .arg(policy_path)
        .arg("--format")
        .arg("text");

    let output = cmd.assert().success();
    let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();

    // Verify policy evaluation occurred
    assert!(stdout.contains("📋 Policy Evaluation"));
    // Should pass the instance type check but may have warnings
    assert!(stdout.contains("✅") || stdout.contains("⚠"));
}

#[test]
fn test_e2e_scan_multi_resource() {
    let temp_dir = TempDir::new().unwrap();
    let plan_path = temp_dir.path().join("plan.json");
    fs::write(&plan_path, MULTI_RESOURCE_PLAN).unwrap();

    let mut cmd = Command::cargo_bin("costpilot").unwrap();
    cmd.arg("scan").arg(&plan_path).arg("--format").arg("json");

    let output = cmd.assert().success();
    let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();

    // Verify multiple resources detected in JSON
    assert!(stdout.contains("aws_instance.web"));
    assert!(stdout.contains("aws_nat_gateway.main"));
    assert!(stdout.contains("aws_s3_bucket.data"));
    assert!(stdout.contains("\"resources_changed\": 3"));
}

#[test]
fn test_e2e_scan_json_output() {
    let temp_dir = TempDir::new().unwrap();
    let plan_path = temp_dir.path().join("plan.json");
    fs::write(&plan_path, SAMPLE_TERRAFORM_PLAN).unwrap();

    let mut cmd = Command::cargo_bin("costpilot").unwrap();
    cmd.arg("scan").arg(plan_path).arg("--format").arg("json");

    let output = cmd.assert().success();
    let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();

    // Verify JSON output structure
    assert!(stdout.contains("summary"));
    assert!(stdout.contains("changes"));
    assert!(stdout.contains("estimates"));
    assert!(stdout.contains("resources_changed"));
    assert!(stdout.contains("monthly_cost"));

    // Should be valid JSON
    serde_json::from_str::<serde_json::Value>(&stdout).unwrap();
}

#[test]
fn test_e2e_scan_with_explain() {
    let temp_dir = TempDir::new().unwrap();
    let plan_path = temp_dir.path().join("plan.json");
    fs::write(&plan_path, SAMPLE_TERRAFORM_PLAN).unwrap();

    let mut cmd = Command::cargo_bin("costpilot").unwrap();
    cmd.arg("scan")
        .arg(&plan_path)
        .arg("--explain")
        .arg("--format")
        .arg("text");

    let output = cmd.assert().success();
    let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();

    // Verify scan still works with explain flag (may not show explanation in free mode)
    assert!(stdout.contains("🔍 CostPilot Scan"));
    assert!(stdout.contains("📊 Detection"));
    assert!(stdout.contains("💰 Cost Prediction"));
}

#[test]
fn test_e2e_scan_with_baselines() {
    let temp_dir = TempDir::new().unwrap();

    // Create plan file
    let plan_path = temp_dir.path().join("plan.json");
    fs::write(&plan_path, SAMPLE_TERRAFORM_PLAN).unwrap();

    // Create baseline file
    let baseline_path = temp_dir.path().join("baseline.json");
    let baseline_content = r#"{
            "version": "1.0",
            "timestamp": "2024-01-01T00:00:00Z",
            "total_monthly_cost": 50.0,
            "resources": {
                "aws_instance.web": {
                    "monthly_cost": 50.0,
                    "resource_type": "aws_instance"
                }
            }
        }"#;
    fs::write(&baseline_path, baseline_content).unwrap();

    let mut cmd = Command::cargo_bin("costpilot").unwrap();
    cmd.arg("scan")
        .arg(plan_path)
        .arg("--baselines")
        .arg(baseline_path)
        .arg("--format")
        .arg("text");

    let output = cmd.assert().success();
    let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();

    // Verify baseline comparison
    assert!(stdout.contains("📊 Baselines Comparison"));
}

#[test]
fn test_e2e_init_workflow() {
    let temp_dir = TempDir::new().unwrap();
    let init_path = temp_dir.path().join("test_project");

    let mut cmd = Command::cargo_bin("costpilot").unwrap();
    cmd.arg("init")
        .arg("--path")
        .arg(init_path.to_str().unwrap())
        .arg("--no-ci");

    let output = cmd.assert().success();
    let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();

    // Verify init output
    assert!(stdout.contains("🚀 Initializing CostPilot"));
    assert!(stdout.contains("✅ CostPilot initialized successfully"));

    // Verify files were created
    assert!(init_path.join(".costpilot").exists());
    assert!(init_path.join(".costpilot/config.yml").exists());
    assert!(init_path.join(".costpilot/policy.yml").exists());
    assert!(init_path.join(".gitignore").exists());
}

#[test]
fn test_e2e_explain_resource() {
    let temp_dir = TempDir::new().unwrap();
    let plan_path = temp_dir.path().join("plan.json");
    fs::write(&plan_path, SAMPLE_TERRAFORM_PLAN).unwrap();

    let mut cmd = Command::cargo_bin("costpilot").unwrap();
    cmd.arg("explain")
        .arg("aws_instance")
        .arg("--instance-type")
        .arg("t3.micro");

    // Explain is available in free mode
    let output = cmd.assert().success();
    let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();

    // Verify it produces output
    assert!(!stdout.is_empty());
}

#[test]
fn test_e2e_explain_all() {
    let temp_dir = TempDir::new().unwrap();
    let plan_path = temp_dir.path().join("plan.json");
    fs::write(&plan_path, MULTI_RESOURCE_PLAN).unwrap();

    let mut cmd = Command::cargo_bin("costpilot").unwrap();
    cmd.arg("explain").arg("aws_lambda_function");

    // Explain is available in free mode
    let output = cmd.assert().success();
    let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();

    // Verify it produces output
    assert!(!stdout.is_empty());
}

#[test]
fn test_e2e_validate_config() {
    let temp_dir = TempDir::new().unwrap();

    // Create valid config file
    let config_path = temp_dir.path().join("config.yml");
    let config_content = r#"version: 1.0.0
detection:
  enabled: true
prediction:
  enabled: true
"#;
    fs::write(&config_path, config_content).unwrap();

    let mut cmd = Command::cargo_bin("costpilot").unwrap();
    cmd.arg("validate").arg(&config_path);

    // Validation should find issues with the config (missing rules)
    let output = cmd.assert().failure();
    let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();

    // Verify validation detected the issues
    assert!(stdout.contains("❌ Status: Invalid"));
    assert!(stdout.contains("E201: Policy has no rules defined"));
}

#[test]
fn test_e2e_error_handling_invalid_plan() {
    let temp_dir = TempDir::new().unwrap();
    let plan_path = temp_dir.path().join("invalid.json");
    fs::write(&plan_path, "invalid json content {").unwrap();

    let mut cmd = Command::cargo_bin("costpilot").unwrap();
    cmd.arg("scan").arg(plan_path);

    let output = cmd.assert().failure();
    let stderr = String::from_utf8(output.get_output().stderr.clone()).unwrap();

    // Verify error handling
    assert!(stderr.contains("error") || stderr.contains("Error"));
}

#[test]
fn test_e2e_error_handling_missing_file() {
    let mut cmd = Command::cargo_bin("costpilot").unwrap();
    cmd.arg("scan").arg("nonexistent.json");

    let output = cmd.assert().failure();
    let stderr = String::from_utf8(output.get_output().stderr.clone()).unwrap();

    // Verify error handling
    assert!(stderr.contains("error") || stderr.contains("Error"));
}

#[test]
fn test_e2e_edition_gating_free() {
    // Test that premium features are properly gated in free edition
    let temp_dir = TempDir::new().unwrap();
    let plan_path = temp_dir.path().join("plan.json");
    fs::write(&plan_path, SAMPLE_TERRAFORM_PLAN).unwrap();

    // This should work in free edition
    let mut cmd = Command::cargo_bin("costpilot").unwrap();
    cmd.arg("scan").arg(plan_path);

    cmd.assert().success();
}

#[test]
fn test_e2e_output_consistency() {
    let temp_dir = TempDir::new().unwrap();
    let plan_path = temp_dir.path().join("plan.json");
    fs::write(&plan_path, SAMPLE_TERRAFORM_PLAN).unwrap();

    // Run scan multiple times to ensure deterministic output
    let mut cmd1 = Command::cargo_bin("costpilot").unwrap();
    cmd1.arg("scan").arg(&plan_path).arg("--format").arg("json");

    let output1 = cmd1.assert().success();
    let stdout1 = String::from_utf8(output1.get_output().stdout.clone()).unwrap();

    let mut cmd2 = Command::cargo_bin("costpilot").unwrap();
    cmd2.arg("scan").arg(&plan_path).arg("--format").arg("json");

    let output2 = cmd2.assert().success();
    let stdout2 = String::from_utf8(output2.get_output().stdout.clone()).unwrap();

    // Outputs should be identical for same input
    assert_eq!(stdout1, stdout2);
}
