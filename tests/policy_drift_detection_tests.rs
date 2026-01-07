use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

// Policy drift detection tests

#[test]
fn test_budget_policy_drift_detection() {
    let temp_dir = TempDir::new().unwrap();
    let policy_path = temp_dir.path().join("policy.yml");
    let plan_path = temp_dir.path().join("plan.json");

    // Create a policy with low budget
    let policy = r#"version: "1.0"
budgets:
  global:
    monthly_limit: 50.0
"#;
    fs::write(&policy_path, policy).unwrap();

    // Create a plan that exceeds budget
    let plan = r#"{
        "format_version": "0.2",
        "terraform_version": "1.5.0",
        "resource_changes": [
            {
                "address": "aws_instance.expensive",
                "type": "aws_instance",
                "name": "expensive",
                "change": {
                    "actions": ["create"],
                    "before": null,
                    "after": {
                        "instance_type": "m5.24xlarge"
                    }
                }
            }
        ]
    }"#;
    fs::write(&plan_path, plan).unwrap();

    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("scan")
        .arg("--policy")
        .arg(&policy_path)
        .arg("--format=json")
        .arg(&plan_path);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("policy_result"))
        .stdout(predicate::str::contains("budget"));
}

#[test]
fn test_tag_policy_drift_detection() {
    let temp_dir = TempDir::new().unwrap();
    let policy_path = temp_dir.path().join("policy.yml");
    let plan_path = temp_dir.path().join("plan.json");

    // Create a policy requiring tags
    let policy = r#"version: "1.0"
policies:
  - name: "require_environment_tag"
    condition: "resource.tags contains 'Environment'"
    action: "deny"
"#;
    fs::write(&policy_path, policy).unwrap();

    // Create a plan with resource missing tag
    let plan = r#"{
        "format_version": "0.2",
        "terraform_version": "1.5.0",
        "resource_changes": [
            {
                "address": "aws_instance.untagged",
                "type": "aws_instance",
                "name": "untagged",
                "change": {
                    "actions": ["create"],
                    "before": null,
                    "after": {
                        "instance_type": "t3.medium",
                        "tags": {
                            "Name": "test"
                        }
                    }
                }
            }
        ]
    }"#;
    fs::write(&plan_path, plan).unwrap();

    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("scan")
        .arg("--policy")
        .arg(&policy_path)
        .arg("--format=json")
        .arg(&plan_path);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("policy_result"));
}

#[test]
fn test_instance_type_policy_drift_detection() {
    let temp_dir = TempDir::new().unwrap();
    let policy_path = temp_dir.path().join("policy.yml");
    let plan_path = temp_dir.path().join("plan.json");

    // Create a policy restricting instance types
    let policy = r#"version: "1.0"
policies:
  - name: "restrict_instance_types"
    condition: "resource.instance_type not in ['t3.micro', 't3.small', 't3.medium']"
    action: "warn"
"#;
    fs::write(&policy_path, policy).unwrap();

    // Create a plan with restricted instance type
    let plan = r#"{
        "format_version": "0.2",
        "terraform_version": "1.5.0",
        "resource_changes": [
            {
                "address": "aws_instance.large",
                "type": "aws_instance",
                "name": "large",
                "change": {
                    "actions": ["create"],
                    "before": null,
                    "after": {
                        "instance_type": "m5.large"
                    }
                }
            }
        ]
    }"#;
    fs::write(&plan_path, plan).unwrap();

    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("scan")
        .arg("--policy")
        .arg(&policy_path)
        .arg("--format=json")
        .arg(&plan_path);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("policy_result"));
}

#[test]
fn test_no_policy_drift_clean_plan() {
    let temp_dir = TempDir::new().unwrap();
    let policy_path = temp_dir.path().join("policy.yml");
    let plan_path = temp_dir.path().join("plan.json");

    // Create a policy
    let policy = r#"version: "1.0"
budgets:
  global:
    monthly_limit: 1000.0
"#;
    fs::write(&policy_path, policy).unwrap();

    // Create a clean plan within budget
    let plan = r#"{
        "format_version": "0.2",
        "terraform_version": "1.5.0",
        "resource_changes": [
            {
                "address": "aws_instance.clean",
                "type": "aws_instance",
                "name": "clean",
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
    fs::write(&plan_path, plan).unwrap();

    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("scan")
        .arg("--policy")
        .arg(&policy_path)
        .arg("--format=json")
        .arg(&plan_path);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("policy_result"));
}

#[test]
fn test_policy_file_not_found() {
    let temp_dir = TempDir::new().unwrap();
    let plan_path = temp_dir.path().join("plan.json");

    let plan = r#"{
        "format_version": "0.2",
        "terraform_version": "1.5.0",
        "resource_changes": []
    }"#;
    fs::write(&plan_path, plan).unwrap();

    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("scan")
        .arg("--policy")
        .arg("nonexistent.yml")
        .arg("--format=json")
        .arg(&plan_path);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains(r#""policy_result": null"#));
}
