use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use std::env;
use std::fs;
use tempfile::TempDir;

// Test CI integration scenarios

#[test]
fn test_ci_github_actions_environment() {
    // Simulate GitHub Actions environment
    env::set_var("GITHUB_ACTIONS", "true");
    env::set_var("GITHUB_WORKFLOW", "CI");
    env::set_var("GITHUB_RUN_ID", "12345");

    let temp_dir = TempDir::new().unwrap();
    let plan_path = temp_dir.path().join("plan.json");
    fs::write(
        &plan_path,
        r#"{
        "format_version": "1.0",
        "terraform_version": "1.5.0",
        "resource_changes": []
    }"#,
    )
    .unwrap();

    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("scan")
        .arg("--format=github-annotations")
        .arg(&plan_path);

    cmd.assert().success();

    env::remove_var("GITHUB_ACTIONS");
    env::remove_var("GITHUB_WORKFLOW");
    env::remove_var("GITHUB_RUN_ID");
}

#[test]
fn test_ci_jenkins_environment() {
    // Simulate Jenkins environment
    env::set_var("JENKINS_HOME", "/var/lib/jenkins");
    env::set_var("BUILD_NUMBER", "42");

    let temp_dir = TempDir::new().unwrap();
    let plan_path = temp_dir.path().join("plan.json");
    fs::write(
        &plan_path,
        r#"{
        "format_version": "1.0",
        "terraform_version": "1.5.0",
        "resource_changes": []
    }"#,
    )
    .unwrap();

    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("scan").arg("--format=text").arg(&plan_path);

    cmd.assert().success();

    env::remove_var("JENKINS_HOME");
    env::remove_var("BUILD_NUMBER");
}

#[test]
fn test_ci_failure_handling() {
    let temp_dir = TempDir::new().unwrap();
    let invalid_plan_path = temp_dir.path().join("invalid.json");
    fs::write(&invalid_plan_path, r#"invalid json"#).unwrap();

    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("scan")
        .arg("--format=github-annotations")
        .arg(&invalid_plan_path);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Error"));
}
