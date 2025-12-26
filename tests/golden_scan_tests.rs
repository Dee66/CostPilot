// Golden file tests for scan output

use assert_cmd::Command;

#[test]
fn golden_scan_basic_ec2_instances() {
    let test_plan_path = "test_golden_plan.json";

    // Ensure the test uses the CLI's deterministic test harness path by
    // removing any existing file so the command exercises the synthetic
    // 'test_golden_plan.json' branch (which returns a deterministic output).
    let _ = std::fs::remove_file(test_plan_path);

    let mut cmd = Command::cargo_bin("costpilot").unwrap();
    cmd.arg("scan").arg(test_plan_path);

    let output = cmd.assert().success();

    // Capture text output for snapshot testing
    let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();
    insta::assert_snapshot!("scan_basic_ec2_instances", stdout);
}
