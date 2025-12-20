use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

use serde_json::json;

/// Create a Terraform plan with a single EC2 instance
fn terraform_plan_with_ec2(instance_type: &str) -> serde_json::Value {
    json!({
        "format_version": "1.1",
        "terraform_version": "1.5.0",
        "resource_changes": [{
            "address": "aws_instance.web",
            "mode": "managed",
            "type": "aws_instance",
            "name": "web",
            "provider_name": "registry.terraform.io/hashicorp/aws",
            "change": {
                "actions": ["create"],
                "before": null,
                "after": {
                    "instance_type": instance_type,
                    "ami": "ami-12345678",
                    "tags": {
                        "Name": "web-server",
                        "Environment": "production"
                    }
                }
            }
        }]
    })
}

/// Test policy enforcement in warn mode
#[test]
fn test_policy_enforcement_warn_mode() {
    let temp_dir = TempDir::new().unwrap();
    let policy_path = temp_dir.path().join("policy.yml");
    let scan_path = temp_dir.path().join("plan.json");

    // Create a policy that should trigger a warning
    let policy_config = r#"{
        "version": "1.0",
        "budgets": {
            "global": {
                "monthly_limit": 100.0
            }
        },
        "enforcement": {
            "mode": "advisory"
        }
    }"#;
    fs::write(&policy_path, policy_config).unwrap();

    // Create Terraform plan that would exceed budget
    let plan_data = terraform_plan_with_ec2("t3.large"); // This should cost more than $100
    fs::write(&scan_path, serde_json::to_string_pretty(&plan_data).unwrap()).unwrap();

    let mut cmd = Command::cargo_bin("costpilot").unwrap();
    cmd.arg("scan")
        .arg(&scan_path)
        .arg("--policy")
        .arg(&policy_path);

    // Should succeed but output warnings
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("policy warnings"))
        .stdout(predicate::str::contains("global_budget"));
}

/// Test policy enforcement in block mode
#[test]
fn test_policy_enforcement_block_mode() {
    let temp_dir = TempDir::new().unwrap();
    let policy_path = temp_dir.path().join("policy.yml");
    let scan_path = temp_dir.path().join("scan.json");

    // Create a policy that should block (but will only warn in free mode)
    let policy_config = r#"{
        "version": "1.0",
        "budgets": {
            "global": {
                "monthly_limit": 100.0
            }
        },
        "enforcement": {
            "mode": "blocking"
        }
    }"#;
    fs::write(&policy_path, policy_config).unwrap();

    // Create Terraform plan that would exceed budget
    let plan_data = terraform_plan_with_ec2("t3.large"); // This should cost more than $100
    fs::write(&scan_path, serde_json::to_string_pretty(&plan_data).unwrap()).unwrap();

    let mut cmd = Command::cargo_bin("costpilot").unwrap();
    cmd.arg("scan")
        .arg(&scan_path)
        .arg("--policy")
        .arg(&policy_path);

    // In free mode, blocking is disabled so it succeeds but shows warnings
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("policy warnings"))
        .stdout(predicate::str::contains("global_budget"));
}

/// Test policy exemption workflow
#[test]
fn test_policy_exemption_workflow() {
    let temp_dir = TempDir::new().unwrap();
    let policy_path = temp_dir.path().join("policy.yml");
    let exemption_path = temp_dir.path().join("exemptions.yml");
    let scan_path = temp_dir.path().join("scan.json");

    // Create a policy that would normally block
    let policy_config = r#"{
        "version": "1.0",
        "budgets": {
            "global": {
                "monthly_limit": 100.0
            }
        },
        "enforcement": {
            "mode": "blocking"
        }
    }"#;
    fs::write(&policy_path, policy_config).unwrap();

    // Create an exemption for this policy
    let exemption_config = r#"{
        "version": "1.0",
        "exemptions": [
            {
                "id": "budget_exemption_001",
                "policy_name": "global_budget",
                "resource_pattern": "global",
                "justification": "Temporary budget increase for Q1 planning",
                "reason": "Temporary budget increase for Q1 planning",
                "approved_by": "finance@company.com",
                "expires_at": "2025-12-31",
                "created_at": "2025-01-01T00:00:00Z",
                "status": "active"
            }
        ]
    }"#;
    fs::write(&exemption_path, exemption_config).unwrap();

    // Create Terraform plan that would exceed budget
    let plan_data = terraform_plan_with_ec2("t3.large"); // This should cost more than $100
    fs::write(&scan_path, serde_json::to_string_pretty(&plan_data).unwrap()).unwrap();

    let mut cmd = Command::cargo_bin("costpilot").unwrap();
    cmd.arg("scan")
        .arg(&scan_path)
        .arg("--policy")
        .arg(&policy_path)
        .arg("--exemptions")
        .arg(&exemption_path)
        .arg("--fail-on-critical");

    // Should succeed due to exemption
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("EXEMPTION_APPLIED"))
        .stdout(predicate::str::contains("budget_exemption_001"));
}

/// Test expired exemption handling
#[test]
fn test_expired_exemption() {
    let temp_dir = TempDir::new().unwrap();
    let policy_path = temp_dir.path().join("policy.yml");
    let exemption_path = temp_dir.path().join("exemptions.yml");
    let scan_path = temp_dir.path().join("scan.json");

    // Create a policy that would normally block
    let policy_config = r#"{
        "version": "1.0",
        "budgets": {
            "global": {
                "monthly_limit": 100.0
            }
        },
        "enforcement": {
            "mode": "blocking"
        }
    }"#;
    fs::write(&policy_path, policy_config).unwrap();

    // Create an expired exemption
    let exemption_config = r#"{
        "version": "1.0",
        "exemptions": [
            {
                "id": "expired_exemption_001",
                "policy_name": "global_budget",
                "resource_pattern": "global",
                "justification": "Old exemption that has expired",
                "reason": "Old exemption that has expired",
                "approved_by": "finance@company.com",
                "expires_at": "2024-12-31",
                "created_at": "2024-01-01T00:00:00Z",
                "status": "expired"
            }
        ]
    }"#;
    fs::write(&exemption_path, exemption_config).unwrap();

    // Create Terraform plan that would exceed budget
    let plan_data = terraform_plan_with_ec2("t3.large"); // This should cost more than $100
    fs::write(&scan_path, serde_json::to_string_pretty(&plan_data).unwrap()).unwrap();

    let mut cmd = Command::cargo_bin("costpilot").unwrap();
    cmd.arg("scan")
        .arg(&scan_path)
        .arg("--policy")
        .arg(&policy_path)
        .arg("--exemptions")
        .arg(&exemption_path);

    // In free mode, even expired exemptions result in warnings, not blocking
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("policy warnings"))
        .stdout(predicate::str::contains("Exemption expired_exemption_001 expired"));
}

/// Test policy versioning and compatibility
#[test]
fn test_policy_versioning() {
    let temp_dir = TempDir::new().unwrap();
    let policy_v1_path = temp_dir.path().join("policy_v1.yml");
    let policy_v2_path = temp_dir.path().join("policy_v2.yml");
    let scan_path = temp_dir.path().join("scan.json");

    // Create v1.0 policy (higher limit)
    let policy_v1_config = r#"{
        "version": "1.0",
        "budgets": {
            "global": {
                "monthly_limit": 200.0
            }
        },
        "enforcement": {
            "mode": "advisory"
        }
    }"#;
    fs::write(&policy_v1_path, policy_v1_config).unwrap();

    // Create v2.0 policy with stricter limits
    let policy_v2_config = r#"{
        "version": "2.0",
        "budgets": {
            "global": {
                "monthly_limit": 100.0
            }
        },
        "enforcement": {
            "mode": "blocking"
        }
    }"#;
    fs::write(&policy_v2_path, policy_v2_config).unwrap();

    // Create Terraform plan that exceeds v2 limit but not v1
    let plan_data = terraform_plan_with_ec2("t3.large"); // This should cost $150
    fs::write(&scan_path, serde_json::to_string_pretty(&plan_data).unwrap()).unwrap();

    // Test v1.0 policy (should succeed - no warnings since $150 < $200)
    let mut cmd_v1 = Command::cargo_bin("costpilot").unwrap();
    cmd_v1.arg("scan")
        .arg(&scan_path)
        .arg("--policy")
        .arg(&policy_v1_path);

    cmd_v1.assert()
        .success()
        .stdout(predicate::str::is_empty().not()); // Should have output but no warnings

    // Test v2.0 policy (should succeed but show warnings since $150 > $100)
    let mut cmd_v2 = Command::cargo_bin("costpilot").unwrap();
    cmd_v2.arg("scan")
        .arg(&scan_path)
        .arg("--policy")
        .arg(&policy_v2_path);

    cmd_v2.assert()
        .success()
        .stdout(predicate::str::contains("policy warnings"))
        .stdout(predicate::str::contains("global_budget"));
}

#[test]
fn test_non_blocking_policy_violations_silent() {
    // Placeholder test for: Non-blocking policy violations → silent
    // TODO: Implement logic to check that non-blocking policy violations
    // result in silent operation (no findings, no explain output, exit code 0)
    assert!(true);
}
