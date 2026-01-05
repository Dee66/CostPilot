use clap::{Arg, Command};
use costpilot::license_issuer::{generate_keypair, generate_license};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = Command::new("CostPilot License Issuer")
        .version("1.0")
        .author("CostPilot Team")
        .about("Generates Ed25519-signed licenses for CostPilot Pro")
        .subcommand(
            Command::new("generate-key")
                .about("Generate a new Ed25519 keypair")
                .arg(
                    Arg::new("key-name")
                        .value_name("NAME")
                        .help("Base name for key files")
                        .default_value("license_key"),
                ),
        )
        .subcommand(
            Command::new("generate-license")
                .about("Generate a signed license")
                .arg(
                    Arg::new("email")
                        .short('e')
                        .long("email")
                        .value_name("EMAIL")
                        .help("User email address")
                        .required(true),
                )
                .arg(
                    Arg::new("license-key")
                        .short('k')
                        .long("license-key")
                        .value_name("KEY")
                        .help("License key string")
                        .required(true),
                )
                .arg(
                    Arg::new("expires")
                        .short('x')
                        .long("expires")
                        .value_name("DATE")
                        .help("Expiration date in ISO 8601 format (e.g., 2025-12-31T23:59:59Z)")
                        .required(true),
                )
                .arg(
                    Arg::new("private-key")
                        .short('p')
                        .long("private-key")
                        .value_name("FILE")
                        .help("Path to Ed25519 private key file (raw 32 bytes)")
                        .required(true),
                )
                .arg(
                    Arg::new("issuer")
                        .short('i')
                        .long("issuer")
                        .value_name("ISSUER")
                        .help("License issuer identifier (default: costpilot-v1)")
                        .default_value("costpilot-v1"),
                )
                .arg(
                    Arg::new("output")
                        .short('o')
                        .long("output")
                        .value_name("FILE")
                        .help("Output file path")
                        .default_value("license.json"),
                ),
        )
        .get_matches();

    match matches.subcommand() {
        Some(("generate-key", sub_matches)) => {
            generate_keypair(sub_matches, &std::env::current_dir().unwrap())
        }
        Some(("generate-license", sub_matches)) => {
            generate_license(sub_matches, &std::env::current_dir().unwrap())
        }
        _ => {
            println!("Use --help for usage information");
            Ok(())
        }
    }
}
