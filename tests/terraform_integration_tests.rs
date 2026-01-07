use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use std::path::Path;

// Integration tests with real Terraform plans

#[test]
fn test_scan_ec2_create_plan() {
    let plan_path = Path::new("tests/fixtures/terraform/ec2_create.json");

    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("scan")
        .arg("--format=json")
        .arg(plan_path);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("changes"));
}

#[test]
fn test_scan_rds_create_plan() {
    let plan_path = Path::new("tests/fixtures/terraform/rds_create.json");

    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("scan")
        .arg("--format=json")
        .arg(plan_path);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("changes"));
}

#[test]
fn test_diff_with_real_plans() {
    let plan_path = Path::new("tests/fixtures/terraform/ec2_create.json");

    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("diff")
        .arg("--format=text")
        .arg(plan_path)
        .arg(plan_path); // diff with itself should show no changes

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("UpgradeRequired"));
}
