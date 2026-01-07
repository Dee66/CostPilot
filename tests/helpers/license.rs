/// License generation helpers for testing with actual licenses

use clap::{Arg, ArgMatches, Command};
use costpilot::license_issuer::{generate_keypair, generate_license};
use std::path::Path;

/// Generate a test keypair for license testing
pub fn generate_test_keypair(temp_dir: &Path, key_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let matches = mock_keypair_matches(key_name);
    generate_keypair(&matches, temp_dir)?;
    Ok(())
}

/// Generate a test license for testing
pub fn generate_test_license(
    temp_dir: &Path,
    email: &str,
    license_key: &str,
    expires: &str,
    private_key_path: &str,
    issuer: &str,
    output_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let matches = mock_license_matches(
        email,
        license_key,
        expires,
        private_key_path,
        issuer,
        output_path,
    );
    generate_license(&matches, temp_dir)?;
    Ok(())
}

/// Mock ArgMatches for generate_keypair
fn mock_keypair_matches(key_name: &str) -> ArgMatches {
    Command::new("license_issuer")
        .subcommand(
            Command::new("generate-key")
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
) -> ArgMatches {
    Command::new("license_issuer")
        .subcommand(
            Command::new("generate-license")
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
