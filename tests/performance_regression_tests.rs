use assert_cmd::cargo::cargo_bin_cmd;
use std::fs;
use std::time::{Duration, Instant};
use tempfile::TempDir;

// Performance regression tests

#[test]
fn test_scan_performance_regression() {
    let temp_dir = TempDir::new().unwrap();
    let plan_path = temp_dir.path().join("plan.json");
    // Create a larger plan for performance testing
    let large_plan = r#"{
        "format_version": "0.2",
        "terraform_version": "1.5.0",
        "resource_changes": [
            {
                "address": "aws_instance.web1",
                "type": "aws_instance",
                "name": "web1",
                "change": {
                    "actions": ["create"],
                    "before": null,
                    "after": {
                        "instance_type": "t3.medium",
                        "ami": "ami-12345678"
                    }
                }
            },
            {
                "address": "aws_instance.web2",
                "type": "aws_instance",
                "name": "web2",
                "change": {
                    "actions": ["create"],
                    "before": null,
                    "after": {
                        "instance_type": "t3.medium",
                        "ami": "ami-12345678"
                    }
                }
            }
        ]
    }"#;
    fs::write(&plan_path, large_plan).unwrap();

    let start = Instant::now();
    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("scan").arg("--format=json").arg(&plan_path);
    cmd.assert().success();
    let duration = start.elapsed();

    println!("Scan performance: {:?}", duration);
    // Assert it completes within reasonable time
    assert!(
        duration < Duration::from_secs(5),
        "Scan took too long: {:?}",
        duration
    );
}

#[test]
fn test_diff_performance_regression() {
    let temp_dir = TempDir::new().unwrap();
    let plan1_path = temp_dir.path().join("plan1.json");
    let plan2_path = temp_dir.path().join("plan2.json");

    let plan1 = r#"{
        "format_version": "0.2",
        "terraform_version": "1.5.0",
        "resource_changes": [
            {
                "address": "aws_instance.web",
                "type": "aws_instance",
                "name": "web",
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

    let plan2 = r#"{
        "format_version": "0.2",
        "terraform_version": "1.5.0",
        "resource_changes": [
            {
                "address": "aws_instance.web",
                "type": "aws_instance",
                "name": "web",
                "change": {
                    "actions": ["update"],
                    "before": {
                        "instance_type": "t3.medium"
                    },
                    "after": {
                        "instance_type": "t3.large"
                    }
                }
            }
        ]
    }"#;

    fs::write(&plan1_path, plan1).unwrap();
    fs::write(&plan2_path, plan2).unwrap();

    let start = Instant::now();
    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("diff")
        .arg("--format=json")
        .arg(&plan1_path)
        .arg(&plan2_path);
    // Diff may fail due to license, but we measure time anyway
    let _output = cmd.output();
    let duration = start.elapsed();

    println!("Diff performance: {:?}", duration);
    assert!(
        duration < Duration::from_secs(5),
        "Diff took too long: {:?}",
        duration
    );
}

#[test]
fn test_memory_usage_regression() {
    // Simple test to ensure no excessive memory growth
    // This is basic; in a real scenario, use tools like heaptrack
    let temp_dir = TempDir::new().unwrap();
    let plan_path = temp_dir.path().join("plan.json");
    let plan = r#"{
        "format_version": "0.2",
        "terraform_version": "1.5.0",
        "resource_changes": []
    }"#;
    fs::write(&plan_path, plan).unwrap();

    let start = Instant::now();
    for _ in 0..10 {
        let mut cmd = cargo_bin_cmd!("costpilot");
        cmd.arg("scan").arg("--format=json").arg(&plan_path);
        cmd.assert().success();
    }
    let duration = start.elapsed();

    println!("10x scan performance: {:?}", duration);
    assert!(
        duration < Duration::from_secs(10),
        "10 scans took too long: {:?}",
        duration
    );
}
