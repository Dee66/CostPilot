// CLI messaging and error format tests
// Verifies user-facing messages are clear, helpful, and appropriately toned

use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

/// Test that free mode doesn't show confusing warnings when no license exists
#[test]
fn test_free_mode_silent_no_license() {
    let temp_dir = TempDir::new().unwrap();
    let plan_path = temp_dir.path().join("plan.json");

    // Create minimal valid terraform plan
    let plan_content = r#"{
        "format_version": "1.0",
        "resource_changes": []
    }"#;
    fs::write(&plan_path, plan_content).unwrap();

    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("scan").arg(&plan_path);

    let output = cmd.assert().success().get_output().clone();
    let stderr = String::from_utf8(output.stderr).unwrap();

    // Should NOT contain confusing warnings about failed license/engine loading
    assert!(
        !stderr.contains("Failed to load Premium engine"),
        "Free mode should not show engine load warnings"
    );
    assert!(
        !stderr.contains("License signature verification failed"),
        "Free mode should not show license verification messages"
    );
}

/// Test that passing a .tf file gives clear error message
#[test]
fn test_tf_file_clear_error() {
    let temp_dir = TempDir::new().unwrap();
    let tf_file = temp_dir.path().join("main.tf");

    // Create a .tf file
    fs::write(&tf_file, "resource \"aws_instance\" \"test\" {}").unwrap();

    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("scan").arg(&tf_file);

    let output = cmd.assert().failure().get_output().clone();
    let stderr = String::from_utf8(output.stderr).unwrap();

    // Should clearly state the problem
    assert!(
        stderr.contains("CostPilot requires Terraform plan JSON")
            || stderr.contains("not .tf source files")
            || stderr.contains("source files"),
        "Should explain that .tf files are not accepted"
    );

    // Should provide actionable hint
    assert!(
        stderr.contains("terraform plan") && stderr.contains("terraform show -json"),
        "Should provide command to generate JSON plan"
    );
}

/// Test that passing a .tfvars file gives clear error message
#[test]
fn test_tfvars_file_clear_error() {
    let temp_dir = TempDir::new().unwrap();
    let tfvars_file = temp_dir.path().join("terraform.tfvars");

    fs::write(&tfvars_file, "region = \"us-east-1\"").unwrap();

    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("scan").arg(&tfvars_file);

    let output = cmd.assert().failure().get_output().clone();
    let stderr = String::from_utf8(output.stderr).unwrap();

    // Should identify the file type issue
    assert!(
        stderr.contains("CostPilot requires Terraform plan JSON") || stderr.contains("tfvars"),
        "Should explain tfvars files are not accepted"
    );
}

/// Test that premium command messaging is clear and helpful
#[test]
fn test_premium_command_clear_message() {
    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("slo");

    let output = cmd.assert().failure().get_output().clone();
    let stderr = String::from_utf8(output.stderr).unwrap();

    // Should NOT say "Unknown command"
    assert!(
        !stderr.contains("Unknown command"),
        "Should not call premium commands 'unknown'"
    );

    // Should explain it's a premium feature
    assert!(
        stderr.contains("requires Premium") || stderr.contains("Premium edition"),
        "Should explain command requires Premium"
    );

    // Should list available free commands
    assert!(
        stderr.contains("scan") || stderr.contains("Available commands"),
        "Should guide users to available commands"
    );

    // Should provide upgrade path
    assert!(
        stderr.contains("shieldcraft-ai.com/costpilot/upgrade") || stderr.contains("upgrade"),
        "Should provide upgrade information"
    );
}

/// Test that non-existent file error is clear
#[test]
fn test_missing_file_clear_error() {
    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("scan").arg("/nonexistent/plan.json");

    let output = cmd.assert().failure().get_output().clone();
    let stderr = String::from_utf8(output.stderr).unwrap();

    // Should mention file not found
    assert!(
        stderr.contains("not found") || stderr.contains("does not exist"),
        "Should indicate file doesn't exist"
    );

    // Should provide terraform command hint
    assert!(
        stderr.contains("terraform") && (stderr.contains("plan") || stderr.contains("show")),
        "Should hint at terraform command"
    );
}

/// Test that invalid JSON gets helpful error
#[test]
fn test_invalid_json_clear_error() {
    let temp_dir = TempDir::new().unwrap();
    let json_file = temp_dir.path().join("invalid.json");

    // Write invalid JSON
    fs::write(&json_file, "{ this is not valid json }").unwrap();

    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("scan").arg(&json_file);

    let output = cmd.assert().failure().get_output().clone();
    let stderr = String::from_utf8(output.stderr).unwrap();

    // Should indicate parsing issue
    assert!(
        stderr.contains("parse") || stderr.contains("JSON") || stderr.contains("invalid"),
        "Should indicate JSON parsing problem"
    );

    // Should hint at correct format
    assert!(
        stderr.contains("terraform show -json"),
        "Should provide command to generate correct format"
    );
}

/// Test that wrong JSON schema gets clear error
#[test]
fn test_wrong_schema_clear_error() {
    let temp_dir = TempDir::new().unwrap();
    let json_file = temp_dir.path().join("wrong.json");

    // Valid JSON but not a terraform plan
    fs::write(&json_file, r#"{"some": "data", "but": "not terraform"}"#).unwrap();

    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("scan").arg(&json_file);

    let output = cmd.assert().failure().get_output().clone();
    let stderr = String::from_utf8(output.stderr).unwrap();

    // Should indicate schema mismatch
    assert!(
        stderr.contains("format_version")
            || stderr.contains("Terraform plan")
            || stderr.contains("resource_changes"),
        "Should indicate missing required fields"
    );
}

/// Test that error IDs are included for debugging
#[test]
fn test_errors_include_ids() {
    let temp_dir = TempDir::new().unwrap();
    let json_file = temp_dir.path().join("bad.json");

    fs::write(&json_file, "not json").unwrap();

    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("scan").arg(&json_file);

    let output = cmd.assert().failure().get_output().clone();
    let stderr = String::from_utf8(output.stderr).unwrap();

    // Should include error code for support/debugging
    assert!(
        stderr.contains("PARSE_")
            || stderr.contains("SCAN_")
            || stderr.contains("[") && stderr.contains("]"),
        "Errors should include identifiable codes"
    );
}

/// Test that hints are actionable
#[test]
fn test_hints_are_actionable() {
    let temp_dir = TempDir::new().unwrap();
    let tf_file = temp_dir.path().join("test.tf");

    fs::write(&tf_file, "# terraform file").unwrap();

    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("scan").arg(&tf_file);

    let output = cmd.assert().failure().get_output().clone();
    let stderr = String::from_utf8(output.stderr).unwrap();

    // Hints should contain actual commands user can run
    assert!(
        stderr.contains("terraform plan -out=") || stderr.contains("terraform show -json"),
        "Hints should include specific commands"
    );

    // Should show complete command pipeline if needed
    if stderr.contains("&&") {
        assert!(
            stderr.contains("plan") && stderr.contains("show"),
            "Multi-step hints should show full pipeline"
        );
    }
}

/// Test that scan with valid file produces clean output
#[test]
fn test_scan_happy_path_clean_output() {
    let temp_dir = TempDir::new().unwrap();
    let plan_file = temp_dir.path().join("plan.json");

    // Valid terraform plan with no changes
    let plan_content = r#"{
        "format_version": "1.0",
        "terraform_version": "1.0.0",
        "resource_changes": []
    }"#;
    fs::write(&plan_file, plan_content).unwrap();

    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("scan").arg(&plan_file);

    let output = cmd.assert().success().get_output().clone();
    let stderr = String::from_utf8(output.stderr).unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    // Stderr should be clean (no warnings)
    assert!(
        stderr.is_empty() || !stderr.contains("warning") && !stderr.contains("error"),
        "Happy path should have clean stderr"
    );

    // Stdout should have scan results
    assert!(
        stdout.contains("Scan") || stdout.contains("CostPilot"),
        "Should show scan output"
    );
}

/// Test that default scan (implicit) works
#[test]
fn test_implicit_scan_works() {
    let temp_dir = TempDir::new().unwrap();
    let plan_file = temp_dir.path().join("test.json");

    let plan_content = r#"{
        "format_version": "1.0",
        "resource_changes": []
    }"#;
    fs::write(&plan_file, plan_content).unwrap();

    // Test without explicit 'scan' command
    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg(&plan_file); // Just the file, no 'scan' subcommand

    let output = cmd.assert().success().get_output().clone();
    let stdout = String::from_utf8(output.stdout).unwrap();

    // Should execute scan
    assert!(
        stdout.contains("Scan") || stdout.contains("Monthly"),
        "Implicit scan should work"
    );
}

/// Test that error messages are user-friendly for Terraform users
#[test]
fn test_terraform_user_friendly_language() {
    let temp_dir = TempDir::new().unwrap();
    let tf_file = temp_dir.path().join("main.tf");

    fs::write(&tf_file, "resource \"aws_instance\" \"web\" {}").unwrap();

    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg(&tf_file);

    let output = cmd.assert().failure().get_output().clone();
    let stderr = String::from_utf8(output.stderr).unwrap();

    // Should use Terraform-familiar terms
    assert!(
        stderr.contains("plan") || stderr.contains("terraform"),
        "Should reference terraform workflows"
    );

    // Should not use overly technical jargon
    assert!(
        !stderr.contains("serde")
            && !stderr.contains("deserialization")
            && !stderr.contains("parser"),
        "Should avoid internal technical terms"
    );

    // Should mention the specific commands users know
    assert!(
        stderr.contains("terraform plan") || stderr.contains("terraform show"),
        "Should reference familiar terraform commands"
    );
}

/// Test that version command shows edition clearly
#[test]
fn test_version_shows_edition() {
    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("--version");

    let output = cmd.assert().success().get_output().clone();
    let stdout = String::from_utf8(output.stdout).unwrap();

    // Should show edition type
    assert!(
        stdout.contains("Free") || stdout.contains("Premium"),
        "Version should show edition"
    );

    // Should show version number
    assert!(
        stdout.contains("1.0") || stdout.contains("costpilot"),
        "Should show version number"
    );
}

/// Test that help output is organized and clear
#[test]
fn test_help_output_organized() {
    let mut cmd = cargo_bin_cmd!("costpilot");
    cmd.arg("--help");

    let output = cmd.assert().success().get_output().clone();
    let stdout = String::from_utf8(output.stdout).unwrap();

    // Should list main commands
    assert!(
        stdout.contains("scan") && stdout.contains("init") && stdout.contains("version"),
        "Help should list primary commands"
    );

    // Should have structure
    assert!(
        stdout.contains("Usage") || stdout.contains("USAGE"),
        "Help should show usage patterns"
    );

    // Should mention options
    assert!(
        stdout.contains("Options") || stdout.contains("FLAGS"),
        "Help should document options"
    );
}
