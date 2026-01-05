#![allow(deprecated)]

use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use std::fs;
use std::path::Path;

#[test]
fn test_cli_help() {
    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Usage: costpilot"))
        .stdout(predicate::str::contains("scan"))
        .stdout(predicate::str::contains("diff"))
        .stdout(predicate::str::contains("validate"));
}

#[test]
fn test_cli_version() {
    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("--version");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("costpilot"));
}

#[test]
fn test_cli_no_args() {
    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Usage: costpilot"));
}

#[test]
fn test_cli_scan_command() {
    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("scan").arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("scan"));
}

#[test]
fn test_cli_diff_command() {
    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("diff").arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("diff"));
}

#[test]
fn test_cli_validate_command() {
    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("validate").arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("validate"));
}

#[test]
fn test_cli_explain_command() {
    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("explain").arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("explain"));
}

#[test]
fn test_cli_map_command() {
    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("map").arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("map"));
}

#[test]
fn test_cli_group_command() {
    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("group").arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("group"));
}

#[test]
fn test_cli_escrow_command() {
    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("escrow").arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("escrow"));
}

#[test]
fn test_cli_policy_lifecycle_command() {
    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("policy-lifecycle").arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("policy-lifecycle"));
}

#[test]
fn test_cli_autofix_patch_command() {
    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("autofix-patch").arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("autofix-patch"));
}

#[test]
fn test_cli_autofix_snippet_command() {
    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("autofix-snippet").arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("autofix-snippet"));
}

#[test]
fn test_cli_slo_burn_command() {
    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("slo-burn").arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("slo-burn"));
}

#[test]
fn test_cli_slo_check_command() {
    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("slo-check").arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("slo-check"));
}

#[test]
fn test_cli_performance_command() {
    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("performance").arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("performance"));
}

#[test]
fn test_cli_init_command() {
    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("init").arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("init"));
}

#[test]
fn test_cli_heuristics_command() {
    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("heuristics").arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("heuristics"));
}

#[test]
fn test_cli_policy_dsl_command() {
    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("policy-dsl").arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("policy-dsl"));
}

#[test]
fn test_cli_audit_command() {
    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("audit").arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("audit"));
}

#[test]
fn test_cli_scan_missing_file() {
    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("scan").arg("nonexistent.json");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("terraform file not found"));
}

#[test]
fn test_cli_validate_missing_file() {
    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("validate").arg("nonexistent.json");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("No such file"));
}

#[test]
fn test_cli_diff_missing_before_file() {
    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("diff")
        .arg("nonexistent1.json")
        .arg("nonexistent2.json");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("UpgradeRequired"))
        .stderr(predicate::str::contains("Diff"));
}

#[test]
fn test_cli_init_creates_structure() {
    let temp_dir = tempfile::tempdir().unwrap();
    let temp_path = temp_dir.path();

    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("init").arg("--path").arg(temp_path);
    cmd.assert().success();

    // Check that .costpilot directory was created
    assert!(temp_path.join(".costpilot").exists());
    assert!(temp_path.join(".costpilot").join("config.yml").exists());
}

#[test]
fn test_cli_heuristics_execute() {
    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("heuristics").arg("execute");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Free heuristics"));
}

#[test]
fn test_cli_performance_budgets() {
    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("performance").arg("budgets");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Performance budgets"));
}

#[test]
fn test_cli_usage_days_in_month() {
    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("usage").arg("--days-in-month").arg("2024-01");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("31"));
}

#[test]
fn test_cli_scan_with_valid_json() {
    let temp_file = tempfile::NamedTempFile::new().unwrap();
    let json_content = r#"{
        "format_version": "1.1",
        "terraform_version": "1.0.0",
        "planned_values": {
            "root_module": {
                "resources": []
            }
        },
        "resource_changes": []
    }"#;
    fs::write(temp_file.path(), json_content).unwrap();

    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("scan").arg(temp_file.path());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("No resource changes detected"));
}

#[test]
fn test_cli_validate_with_valid_json() {
    let temp_dir = tempfile::tempdir().unwrap();
    let temp_path = temp_dir.path().join("baselines.json");
    let json_content = r#"{
        "version": "1.0",
        "baselines": {
            "TestInstance": {
                "monthly_cost": 10.0
            }
        }
    }"#;
    fs::write(&temp_path, json_content).unwrap();

    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("validate").arg(&temp_path);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Status: Valid"));
}

#[test]
fn test_cli_diff_with_identical_files() {
    let temp_file1 = tempfile::NamedTempFile::new().unwrap();
    let temp_file2 = tempfile::NamedTempFile::new().unwrap();
    let json_content = r#"{
        "Resources": {
            "TestInstance": {
                "Type": "AWS::EC2::Instance",
                "Properties": {
                    "InstanceType": "t3.micro"
                }
            }
        }
    }"#;
    fs::write(temp_file1.path(), json_content).unwrap();
    fs::write(temp_file2.path(), json_content).unwrap();

    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("diff")
        .arg(temp_file1.path())
        .arg(temp_file2.path());
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("UpgradeRequired"))
        .stderr(predicate::str::contains("Diff"));
}

#[test]
fn test_cli_explain_with_valid_resource() {
    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("explain")
        .arg("aws_instance")
        .arg("--instance-type")
        .arg("t3.micro");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Explanation"));
}

#[test]
fn test_cli_map_with_valid_json() {
    let temp_file = tempfile::NamedTempFile::new().unwrap();
    let json_content = r#"{
        "Resources": {
            "TestInstance": {
                "Type": "AWS::EC2::Instance",
                "Properties": {
                    "InstanceType": "t3.micro"
                }
            },
            "TestDB": {
                "Type": "AWS::RDS::DBInstance",
                "Properties": {
                    "DBInstanceClass": "db.t3.micro"
                }
            }
        }
    }"#;
    fs::write(temp_file.path(), json_content).unwrap();

    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("map").arg(temp_file.path());
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("UpgradeRequired"))
        .stderr(predicate::str::contains("Deep mapping"));
}

#[test]
fn test_cli_group_by_service() {
    let temp_file = tempfile::NamedTempFile::new().unwrap();
    let json_content = r#"{
        "format_version": "1.1",
        "terraform_version": "1.0.0",
        "planned_values": {
            "root_module": {
                "resources": []
            }
        },
        "resource_changes": []
    }"#;
    fs::write(temp_file.path(), json_content).unwrap();

    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("group").arg("by-service").arg(temp_file.path());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Service Cost Summary"));
}

#[test]
fn test_cli_group_by_environment() {
    let temp_file = tempfile::NamedTempFile::new().unwrap();
    let json_content = r#"{
        "format_version": "1.1",
        "terraform_version": "1.0.0",
        "planned_values": {
            "root_module": {
                "resources": []
            }
        },
        "resource_changes": []
    }"#;
    fs::write(temp_file.path(), json_content).unwrap();

    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("group").arg("by-environment").arg(temp_file.path());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Environment Grouping Report"));
}

#[test]
fn test_cli_slo_burn_with_valid_config() {
    let temp_dir = tempfile::tempdir().unwrap();
    let slo_path = temp_dir.path().join("slo.json");
    let snapshots_path = temp_dir.path().join("snapshots");

    // Create test SLO configuration
    let slo_config = r#"{
        "version": "1.0",
        "slos": [
            {
                "id": "test_slo_cli",
                "name": "test_slo",
                "description": "Test SLO for CLI integration",
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

    // Create snapshots directory and minimal historical data
    fs::create_dir(&snapshots_path).unwrap();
    create_test_snapshot(&snapshots_path, "2025-01-01", 100.0);
    create_test_snapshot(&snapshots_path, "2025-02-01", 120.0);
    create_test_snapshot(&snapshots_path, "2025-03-01", 140.0);

    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("slo-burn")
        .arg("--config")
        .arg(&slo_path)
        .arg("--snapshots-dir")
        .arg(&snapshots_path);
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("require CostPilot Premium"));
}

// Helper function to create test snapshots
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

#[test]
fn test_cli_version_stable() {
    let mut cmd1 = cargo_bin_cmd!("costpilot");
    cmd1.arg("--version");
    let output1 = cmd1.output().unwrap();
    assert!(output1.status.success());
    let version1 = String::from_utf8_lossy(&output1.stdout).trim().to_string();

    let mut cmd2 = cargo_bin_cmd!("costpilot");
    cmd2.arg("--version");
    let output2 = cmd2.output().unwrap();
    assert!(output2.status.success());
    let version2 = String::from_utf8_lossy(&output2.stdout).trim().to_string();

    assert_eq!(version1, version2, "Version output not stable across runs");
}

#[test]
fn test_cli_scan_no_cost_risk_silent() {
    let temp_file = tempfile::NamedTempFile::new().unwrap();
    let json_content = r#"{
        "format_version": "1.0",
        "terraform_version": "1.5.0",
        "planned_values": {
            "root_module": {
                "resources": []
            }
        },
        "resource_changes": []
    }"#;
    fs::write(temp_file.path(), json_content).unwrap();

    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("scan").arg(temp_file.path());
    let output = cmd.assert().success();

    // When no cost risk exists, output should not contain warnings or errors
    let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();
    assert!(!stdout.contains("WARNING"));
    assert!(!stdout.contains("ERROR"));
    assert!(!stdout.contains("RISK"));
    assert!(stdout.contains("No resource changes detected"));
}

#[test]
fn test_terraform_plan_delta_below_threshold_silent() {
    // TODO: Implement test for silence invariant
    // When Terraform plan delta is below baseline threshold, command should be silent
    // - Exit code 0
    // - No stdout output
    // - No stderr output
    // Placeholder: test not implemented yet
}

#[test]
fn test_json_output_canonical_serialization() {
    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("scan")
        .arg("test_golden_plan.json")
        .arg("--format")
        .arg("json");

    let output1 = cmd.assert().success().get_output().clone();
    let stdout1 = String::from_utf8_lossy(&output1.stdout);

    // Run the same command again
    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("scan")
        .arg("test_golden_plan.json")
        .arg("--format")
        .arg("json");

    let output2 = cmd.assert().success().get_output().clone();
    let stdout2 = String::from_utf8_lossy(&output2.stdout);

    // Extract JSON part (skip the banner)
    let json1 = extract_json_from_output(&stdout1);
    let json2 = extract_json_from_output(&stdout2);

    // JSON outputs should be byte-for-byte identical
    assert_eq!(
        json1, json2,
        "JSON output should be deterministic and canonical"
    );
}

fn extract_json_from_output(output: &str) -> &str {
    // Find the first '{' character and return everything from there
    if let Some(start) = output.find('{') {
        &output[start..]
    } else {
        output
    }
}
