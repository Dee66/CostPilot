/// Integration tests using actual licenses instead of mocks

use std::fs;
use std::path::Path;
use tempfile::TempDir;
use clap::{Arg, Command as ClapCommand};
use costpilot::license_issuer::{generate_keypair, generate_license};
use costpilot::edition::EditionContext;
use std::process::Command;

/// Mock ArgMatches for generate_keypair
fn mock_keypair_matches(key_name: &str) -> clap::ArgMatches {
    ClapCommand::new("license_issuer")
        .subcommand(
            ClapCommand::new("generate-key")
                .arg(
                    Arg::new("key-name")
                        .long("key-name")
                        .default_value(key_name),
                ),
        )
        .get_matches_from(vec!["license_issuer", "generate-key", "--key-name", key_name])
}

/// Mock ArgMatches for generate_license
fn mock_license_matches(
    email: &str,
    license_key: &str,
    expires: &str,
    private_key: &str,
    issuer: &str,
    output: &str,
) -> clap::ArgMatches {
    ClapCommand::new("license_issuer")
        .subcommand(
            ClapCommand::new("generate-license")
                .arg(Arg::new("email").long("email").required(true))
                .arg(Arg::new("license-key").long("license-key").required(true))
                .arg(Arg::new("expires").long("expires").required(true))
                .arg(Arg::new("private-key").long("private-key").required(true))
                .arg(Arg::new("issuer").long("issuer").default_value("CostPilot"))
                .arg(Arg::new("output").long("output").default_value("license.json")),
        )
        .get_matches_from(vec![
            "license_issuer",
            "generate-license",
            "--email",
            email,
            "--license-key",
            license_key,
            "--expires",
            expires,
            "--private-key",
            private_key,
            "--issuer",
            issuer,
            "--output",
            output,
        ])
}

#[test]
fn test_generate_and_verify_real_license() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Generate test keypair
    let key_matches = mock_keypair_matches("test_key");
    generate_keypair(&key_matches, temp_path).unwrap();

    // Generate test license
    let license_matches = mock_license_matches(
        "test@example.com",
        "test-license-key",
        "2025-12-31",
        "test_key.pem",
        "CostPilot",
        "license.json",
    );
    generate_license(&license_matches, temp_path).unwrap();

    // Verify license file exists
    let license_path = temp_path.join("license.json");
    assert!(license_path.exists());

    // Load license and verify
    let license = costpilot::edition::License::load_from_file(&license_path).unwrap().unwrap();
    assert!(license.verify_signature());
}

#[test]
fn test_cli_with_real_license() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Generate test keypair
    let key_matches = mock_keypair_matches("test_key");
    generate_keypair(&key_matches, temp_path).unwrap();

    // Generate test license
    let license_matches = mock_license_matches(
        "test@example.com",
        "test-license-key",
        "2025-12-31",
        "test_key.pem",
        "CostPilot",
        "license.json",
    );
    generate_license(&license_matches, temp_path).unwrap();

    // Create .costpilot dir and copy license
    let costpilot_dir = temp_path.join(".costpilot");
    fs::create_dir(&costpilot_dir).unwrap();
    fs::copy(temp_path.join("license.json"), costpilot_dir.join("license.json")).unwrap();

    // Set HOME to temp dir
    let home_dir = temp_path.to_str().unwrap();

    // Run CLI command with license
    let output = Command::new("cargo")
        .args(&["run", "--quiet", "--", "explain", "aws_instance", "--instance-type", "t3.micro"])
        .env("HOME", home_dir)
        .output()
        .unwrap();

    // Should succeed
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(!stdout.is_empty());
}
