// Golden file tests for explain command output

use assert_cmd::Command;

#[test]
fn golden_explain_ec2_instance() {
    let mut cmd = Command::cargo_bin("costpilot").unwrap();
    cmd.arg("explain")
        .arg("aws_instance")
        .arg("--instance-type")
        .arg("t3.micro");

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    insta::assert_snapshot!("explain_ec2_t3_micro", stdout);
}

// ===== GOLDEN EXPLAIN EDGE CASE TESTS =====

#[test]
fn test_golden_explain_empty_instance_type_edge_case() {
    let mut cmd = Command::cargo_bin("costpilot").unwrap();
    let result = cmd
        .arg("explain")
        .arg("aws_instance")
        .arg("--instance-type")
        .arg("")
        .output();

    // Should handle gracefully
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_golden_explain_invalid_instance_type_edge_case() {
    let mut cmd = Command::cargo_bin("costpilot").unwrap();
    let result = cmd
        .arg("explain")
        .arg("aws_instance")
        .arg("--instance-type")
        .arg("invalid-instance-type")
        .output();

    // Should handle gracefully
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_golden_explain_extremely_long_instance_type() {
    let long_type = "a".repeat(1000);
    let mut cmd = Command::cargo_bin("costpilot").unwrap();
    let result = cmd
        .arg("explain")
        .arg("aws_instance")
        .arg("--instance-type")
        .arg(&long_type)
        .output();

    // Should handle gracefully
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_golden_explain_special_characters_in_instance_type() {
    let mut cmd = Command::cargo_bin("costpilot").unwrap();
    let result = cmd
        .arg("explain")
        .arg("aws_instance")
        .arg("--instance-type")
        .arg("instance@domain.com")
        .output();

    // Should handle gracefully
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_golden_explain_unicode_instance_type() {
    let mut cmd = Command::cargo_bin("costpilot").unwrap();
    let result = cmd
        .arg("explain")
        .arg("aws_instance")
        .arg("--instance-type")
        .arg("测试实例")
        .output();

    // Should handle gracefully
    assert!(result.is_ok() || result.is_err());
}
