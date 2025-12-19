use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_version_command() {
    let mut cmd = Command::cargo_bin("costpilot").unwrap();
    cmd.arg("version")
        .assert()
        .success()
        .stdout(predicate::str::contains("costpilot"));
}

#[test]
fn test_version_detailed_command() {
    let mut cmd = Command::cargo_bin("costpilot").unwrap();
    cmd.arg("version")
        .arg("--detailed")
        .assert()
        .success()
        .stdout(predicate::str::contains("CostPilot"));
}

#[test]
fn test_init_command() {
    let temp_dir = TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("costpilot").unwrap();
    cmd.arg("init")
        .current_dir(&temp_dir)
        .assert()
        .success();

    // Check if .costpilot directory was created
    assert!(temp_dir.path().join(".costpilot").exists());
}

#[test]
fn test_init_no_ci_command() {
    let temp_dir = TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("costpilot").unwrap();
    cmd.arg("init")
        .arg("--no-ci")
        .current_dir(&temp_dir)
        .assert()
        .success();

    // Check if .costpilot directory was created
    assert!(temp_dir.path().join(".costpilot").exists());
}

#[test]
fn test_heuristics_command() {
    let mut cmd = Command::cargo_bin("costpilot").unwrap();
    cmd.arg("heuristics")
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("Available heuristics"));
}

#[test]
fn test_explain_command() {
    let mut cmd = Command::cargo_bin("costpilot").unwrap();
    cmd.arg("explain")
        .arg("aws_instance")
        .arg("--instance-type")
        .arg("t3.micro")
        .assert()
        .success()
        .stdout(predicate::str::contains("aws_instance"));
}

#[test]
fn test_explain_unknown_resource() {
    let mut cmd = Command::cargo_bin("costpilot").unwrap();
    cmd.arg("explain")
        .arg("unknown_resource")
        .arg("--instance-type")
        .arg("unknown")
        .assert()
        .success()
        .stdout(predicate::str::contains("Explanation for unknown_resource"))
        .stdout(predicate::str::contains("Predicted monthly cost: $10.00"))
        .stdout(predicate::str::contains("Confidence: 65.0%"));
}

#[test]
fn test_validate_command_valid_file() {
    let temp_dir = TempDir::new().unwrap();
    let yaml_content = r#"---
metadata:
  id: test-policy
  name: Test Policy
  description: A test policy
  category: budget
  severity: warning
  status: active
  version: "1.0.0"
  lifecycle:
    phase: experimental
    created_at: "2024-01-01T00:00:00Z"
    updated_at: "2024-01-01T00:00:00Z"
  ownership:
    team: test-team
    contact: test@example.com
    author: test-author
    owner: test-owner
rules:
  - name: test-rule
    description: A test rule
    enabled: true
    severity: Info
    conditions:
      - condition_type:
          type: expression
          expr: "true"
        operator: equals
        value: true
    action:
      type: warn
      message: "Test warning"
"#;
    let file_path = temp_dir.path().join("test.yaml");
    fs::write(&file_path, yaml_content).unwrap();

    let mut cmd = Command::cargo_bin("costpilot").unwrap();
    cmd.arg("validate")
        .arg(&file_path)
        .assert()
        .success();
}

#[test]
fn test_validate_command_invalid_file() {
    let mut cmd = Command::cargo_bin("costpilot").unwrap();
    cmd.arg("validate")
        .arg("nonexistent.yaml")
        .assert()
        .failure()
        .stderr(predicate::str::contains("No such file"));
}

#[test]
fn test_validate_command_invalid_json() {
    let temp_dir = TempDir::new().unwrap();
    let invalid_yaml = r#"---
Resources:
  TestInstance:
    Type: AWS::EC2::Instance
    Properties:
      InstanceType: t3.micro
# Missing closing
"#;
    let file_path = temp_dir.path().join("invalid.yaml");
    fs::write(&file_path, invalid_yaml).unwrap();

    let mut cmd = Command::cargo_bin("costpilot").unwrap();
    cmd.arg("validate")
        .arg(&file_path)
        .assert()
        .failure()
        .stdout(predicate::str::contains("Policy has no rules defined"));
}