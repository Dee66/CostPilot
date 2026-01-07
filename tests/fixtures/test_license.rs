/// Shared test license infrastructure
///
/// Generates a real Ed25519 keypair and valid licenses for testing Premium features
/// This uses REAL signature verification - no bypasses

use ed25519_dalek::{SigningKey, Signer, VerifyingKey};
use serde_json::json;
use std::fs;
use std::path::Path;
use std::sync::OnceLock;

/// Fixed test keypair (deterministic for consistent testing)
/// In production, this would be the real license issuer's keypair
static TEST_KEYPAIR: OnceLock<SigningKey> = OnceLock::new();

/// Get or generate the test signing key (deterministic seed for reproducibility)
pub fn get_test_signing_key() -> &'static SigningKey {
    TEST_KEYPAIR.get_or_init(|| {
        // Use a fixed seed for deterministic test keys
        // This allows TEST_LICENSE_PUBLIC_KEY in crypto.rs to be hardcoded
        let seed = [42u8; 32]; // Fixed seed for tests
        SigningKey::from_bytes(&seed)
    })
}

/// Get the corresponding public key bytes for embedding in crypto.rs
#[allow(dead_code)]
pub fn get_test_public_key_bytes() -> [u8; 32] {
    let signing_key = get_test_signing_key();
    let verifying_key: VerifyingKey = signing_key.verifying_key();
    verifying_key.to_bytes()
}

/// Generate a valid test license and write to specified path
pub fn create_test_license(
    output_path: &Path,
    email: &str,
    license_key: &str,
    expires: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let signing_key = get_test_signing_key();
    let issuer = "test-costpilot"; // Uses real verification with TEST_LICENSE_PUBLIC_KEY

    // Create canonical message matching license_issuer.rs format
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

    // Ensure parent directory exists
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Write to file
    fs::write(output_path, serde_json::to_string_pretty(&license)?)?;

    Ok(())
}

/// Create a valid Premium license in the standard location for a test
pub fn setup_premium_license_for_test(
    home_dir: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let costpilot_dir = home_dir.join(".costpilot");
    let license_path = costpilot_dir.join("license.json");

    create_test_license(
        &license_path,
        "test@example.com",
        "TEST-PREMIUM-LICENSE-KEY",
        "2099-12-31T23:59:59Z",
    )
}
