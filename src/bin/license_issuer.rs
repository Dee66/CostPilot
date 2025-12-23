use clap::{Arg, ArgMatches, Command};
use ed25519_dalek::{Signer, SigningKey};
use rand::rngs::OsRng;
use rand::RngCore;
use serde_json::json;
use std::fs;

fn generate_keypair(matches: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let key_name = matches.get_one::<String>("key-name").unwrap();
    let private_path = format!("{}.pem", key_name);
    let public_path = format!("{}.pub.pem", key_name);

    // Generate random 32-byte secret key
    let mut csprng = OsRng;
    let mut secret_bytes = [0u8; 32];
    csprng.fill_bytes(&mut secret_bytes);
    let signing_key = SigningKey::from_bytes(&secret_bytes);
    let verifying_key = signing_key.verifying_key();

    // Write raw private key
    fs::write(&private_path, signing_key.to_bytes())?;

    // Write public key in PEM-like format (just base64 for simplicity)
    use base64::Engine;
    let public_pem = format!(
        "-----BEGIN PUBLIC KEY-----\n{}\n-----END PUBLIC KEY-----",
        base64::engine::general_purpose::STANDARD.encode(verifying_key.to_bytes())
    );
    fs::write(&public_path, public_pem)?;

    println!("Keypair generated:");
    println!("  Private: {}", private_path);
    println!("  Public:  {}", public_path);
    println!(
        "  Fingerprint: {}",
        hex::encode(&verifying_key.to_bytes()[..8])
    );

    Ok(())
}

fn generate_license(matches: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let email = matches.get_one::<String>("email").unwrap();
    let license_key = matches.get_one::<String>("license-key").unwrap();
    let expires = matches.get_one::<String>("expires").unwrap();
    let issuer = matches
        .get_one::<String>("issuer")
        .cloned()
        .unwrap_or_else(|| "costpilot-v1".to_string());
    let private_key_path = matches.get_one::<String>("private-key").unwrap();
    let output_path = matches.get_one::<String>("output").unwrap();

    // Load private key (raw bytes)
    let key_data = fs::read(private_key_path)?;
    let key_bytes: [u8; 32] = key_data
        .try_into()
        .map_err(|_| "Invalid key length: expected 32 bytes")?;
    let signing_key = SigningKey::from_bytes(&key_bytes);

    // Create canonical message (now includes issuer)
    let canonical_message = format!("{}|{}|{}|{}", email, license_key, expires, issuer);

    // Sign the message
    let signature = signing_key.sign(canonical_message.as_bytes());

    // Generate issued_at timestamp
    let issued_at = chrono::Utc::now().to_rfc3339();

    // Create license JSON
    let license = json!({
        "email": email,
        "license_key": license_key,
        "expires": expires,
        "issued_at": issued_at,
        "signature": hex::encode(signature.to_bytes()),
        "version": "1.0",
        "issuer": issuer
    });

    // Write to file
    fs::write(output_path, serde_json::to_string_pretty(&license)?)?;

    println!("License generated successfully: {}", output_path);
    println!(
        "Key fingerprint: {}",
        hex::encode(&signing_key.verifying_key().to_bytes()[..8])
    );

    Ok(())
}

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
        Some(("generate-key", sub_matches)) => generate_keypair(sub_matches),
        Some(("generate-license", sub_matches)) => generate_license(sub_matches),
        _ => {
            println!("Use --help for usage information");
            Ok(())
        }
    }
}
