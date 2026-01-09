use costpilot_license_issuer::{EditionTier, LicenseIssuer, LicenseRequest};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== CostPilot License Generator Example ===\n");

    // Step 1: Generate a keypair
    println!("1. Generating Ed25519 keypair...");
    let keypair = LicenseIssuer::generate_keypair()?;

    println!("   Private Key (hex): {}", hex::encode(&keypair.private_key_bytes));
    println!("   Public Key (hex):  {}", keypair.public_key_hex);
    println!("   Fingerprint:       {}\n", keypair.fingerprint);

    // Step 2: Create an issuer
    println!("2. Creating license issuer...");
    let issuer = LicenseIssuer::from_private_key_bytes(&keypair.private_key_bytes)?;
    println!("   Issuer fingerprint: {}\n", issuer.fingerprint());

    // Step 3: Issue licenses for different tiers
    println!("3. Issuing licenses for each tier:\n");

    // Free license
    let free_request = LicenseRequest {
        email: "test-free@example.com".to_string(),
        license_key: format!("FREE-{}", uuid::Uuid::new_v4().to_string().to_uppercase()[..8].to_string()),
        edition: EditionTier::Free,
        expires_days: 365,
    };
    let free_license = issuer.issue_license(free_request)?;
    println!("--- FREE TIER LICENSE ---");
    println!("{}\n", serde_json::to_string_pretty(&free_license)?);

    // Premium license
    let premium_request = LicenseRequest {
        email: "test-premium@example.com".to_string(),
        license_key: format!("PREMIUM-{}", uuid::Uuid::new_v4().to_string().to_uppercase()[..8].to_string()),
        edition: EditionTier::Premium,
        expires_days: 365,
    };
    let premium_license = issuer.issue_license(premium_request)?;
    println!("--- PREMIUM TIER LICENSE ---");
    println!("{}\n", serde_json::to_string_pretty(&premium_license)?);

    // Enterprise license
    let enterprise_request = LicenseRequest {
        email: "test-enterprise@example.com".to_string(),
        license_key: format!("ENTERPRISE-{}", uuid::Uuid::new_v4().to_string().to_uppercase()[..8].to_string()),
        edition: EditionTier::Enterprise,
        expires_days: 365,
    };
    let enterprise_license = issuer.issue_license(enterprise_request)?;
    println!("--- ENTERPRISE TIER LICENSE ---");
    println!("{}\n", serde_json::to_string_pretty(&enterprise_license)?);

    // Step 4: Save licenses to files
    println!("4. Saving licenses to files...");
    std::fs::write(
        "license_free.json",
        serde_json::to_string_pretty(&free_license)?,
    )?;
    std::fs::write(
        "license_premium.json",
        serde_json::to_string_pretty(&premium_license)?,
    )?;
    std::fs::write(
        "license_enterprise.json",
        serde_json::to_string_pretty(&enterprise_license)?,
    )?;

    // Save keypair info
    let keypair_info = serde_json::json!({
        "private_key_hex": hex::encode(&keypair.private_key_bytes),
        "public_key_hex": keypair.public_key_hex,
        "public_key_base64": keypair.public_key_base64,
        "fingerprint": keypair.fingerprint,
        "warning": "KEEP PRIVATE KEY SECURE - Never commit to version control!"
    });
    std::fs::write("keypair_info.json", serde_json::to_string_pretty(&keypair_info)?)?;

    println!("   ✓ license_free.json");
    println!("   ✓ license_premium.json");
    println!("   ✓ license_enterprise.json");
    println!("   ✓ keypair_info.json\n");

    println!("=== IMPORTANT NOTES ===");
    println!("• Keep the private key secure (in keypair_info.json)");
    println!("• Share the public key with CostPilot for verification");
    println!("• The licenses are ready to use for testing");
    println!("• To verify: CostPilot needs the public key configured\n");

    Ok(())
}
