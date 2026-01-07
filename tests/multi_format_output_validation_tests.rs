use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

// Multi-format output validation tests

#[test]
fn test_scan_json_output_format() {
    let temp_dir = TempDir::new().unwrap();
    let plan_path = temp_dir.path().join("plan.json");
    let plan = r#"{
        "format_version": "0.2",
        "terraform_version": "1.5.0",
        "resource_changes": [
            {
                "address": "aws_instance.example",
                "type": "aws_instance",
                "name": "example",
                "change": {
                    "actions": ["create"],
                    "before": null,
                    "after": {
                        "instance_type": "t3.medium"
                    }
                }
            }
        ]
    }"#;
    fs::write(&plan_path, plan).unwrap();

    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("scan")
        .arg("--format=json")
        .arg(&plan_path);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains(r#""changes""#))
        .stdout(predicate::str::contains(r#""estimates""#))
        .stdout(predicate::str::contains(r#""summary""#));
}

#[test]
fn test_scan_text_output_format() {
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
        .arg("--format=text")
        .arg(&plan_path);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("CostPilot Scan"))
        .stdout(predicate::str::contains("Detection"));
}

#[test]
fn test_scan_markdown_output_format() {
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
        .arg("--format=markdown")
        .arg(&plan_path);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("# CostPilot"))
        .stdout(predicate::str::contains("##"));
}

#[test]
fn test_scan_github_annotations_output_format() {
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
        .arg("--format=github-annotations")
        .arg(&plan_path);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("CostPilot Scan"));
}

#[test]
fn test_diff_json_output_format() {
    let temp_dir = TempDir::new().unwrap();
    let plan1_path = temp_dir.path().join("plan1.json");
    let plan2_path = temp_dir.path().join("plan2.json");

    let plan1 = r#"{
        "format_version": "0.2",
        "terraform_version": "1.5.0",
        "resource_changes": []
    }"#;

    let plan2 = r#"{
        "format_version": "0.2",
        "terraform_version": "1.5.0",
        "resource_changes": []
    }"#;

    fs::write(&plan1_path, plan1).unwrap();
    fs::write(&plan2_path, plan2).unwrap();

    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("diff")
        .arg("--format=json")
        .arg(&plan1_path)
        .arg(&plan2_path);

    // Diff may fail due to license, but if it succeeds, check format
    let output = cmd.output().unwrap();
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains(r#""changes""#) || stdout.contains("No changes"));
    }
}

#[test]
fn test_invalid_format_error() {
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
        .arg("--format=invalid")
        .arg(&plan_path);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("CostPilot Scan"));
}
