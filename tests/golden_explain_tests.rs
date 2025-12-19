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
