use ed25519_dalek::{Signer, SigningKey};
/// Verify that expired licenses with valid signatures do NOT grant Premium access
/// This test verifies the fix for the claimed "license expiry bug"
use std::fs;
use tempfile::TempDir;

#[test]
fn test_expired_license_with_valid_signature_does_not_grant_premium() {
    // Create isolated test environment
    let temp_home = TempDir::new().unwrap();
    std::env::set_var("HOME", temp_home.path());

    let costpilot_dir = temp_home.path().join(".costpilot");
    fs::create_dir_all(&costpilot_dir).unwrap();

    // Create an EXPIRED license with a VALID signature
    let seed = [42u8; 32]; // Test keypair seed
    let signing_key = SigningKey::from_bytes(&seed);
    let email = "expired@example.com";
    let license_key = "EXPIRED-VALID-SIG-KEY";
    let expires = "2020-01-01T00:00:00Z"; // Expired in 2020
    let issuer = "test-costpilot";

    // Sign the license (valid signature)
    let message = format!("{}|{}|{}|{}", email, license_key, expires, issuer);
    let signature = signing_key.sign(message.as_bytes());

    let license_json = serde_json::json!({
        "email": email,
        "license_key": license_key,
        "expires": expires,
        "signature": hex::encode(signature.to_bytes()),
        "issuer": issuer
    });

    let license_path = costpilot_dir.join("license.json");
    fs::write(
        &license_path,
        serde_json::to_string_pretty(&license_json).unwrap(),
    )
    .unwrap();

    // Now detect edition - should be Free, not Premium
    let edition = costpilot::edition::detect_edition().expect("detect_edition should succeed");

    // ASSERT: Expired license should NOT grant Premium access
    assert!(
        !matches!(edition.mode, costpilot::edition::EditionMode::Premium),
        "❌ BUG CONFIRMED: Expired license with valid signature granted Premium access!"
    );

    // Edition should be Free
    assert!(
        matches!(edition.mode, costpilot::edition::EditionMode::Free),
        "Expired license should fall back to Free edition"
    );

    // License should not be loaded
    assert!(
        edition.license.is_none(),
        "Expired license should not be loaded into Edition struct"
    );
}
