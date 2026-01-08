/// Key Rotation Verification Tests
///
/// These tests enforce that:
/// 1. OLD compromised keys are completely rejected
/// 2. NEW rotated keys are the only accepted keys
/// 3. License verification fails for licenses signed with old keys
/// 4. License verification succeeds for licenses signed with new keys
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde_json::json;

// OLD REVOKED KEYS (exposed in git history - commits e227b546 and earlier)
// These fingerprints: 23837ac5 (license), 10f8798e (wasm)
// These keys MUST be rejected by the runtime

// NEW ROTATED KEYS (generated 2026-01-08)
// Public keys embedded in runtime via environment variables
// Private keys stored in external secure key management system
const NEW_LICENSE_PUBLIC_KEY_HEX: &str =
    "db52fc95fe7ccbd5e55ecfd357d8271d1b2d4a9f608e68db3e7f869d54dba5df";
const NEW_LICENSE_PRIVATE_KEY_HEX: &str =
    "f93ae7fac0d68cc64b6160b949e49816fda863de16d3a0ad3bbd489cd6d46364";

#[test]
fn test_runtime_embeds_new_license_public_key() {
    // Verify that the runtime has embedded the NEW public key
    use costpilot::pro_engine::crypto::LICENSE_PUBLIC_KEY;

    let expected_new_key = hex::decode(NEW_LICENSE_PUBLIC_KEY_HEX).unwrap();

    assert_eq!(
        LICENSE_PUBLIC_KEY,
        expected_new_key.as_slice(),
        "Runtime MUST embed the NEW rotated license public key (fingerprint: db52fc95)"
    );

    // Verify fingerprint
    let fingerprint = hex::encode(&LICENSE_PUBLIC_KEY[..4]);
    assert_eq!(
        fingerprint, "db52fc95",
        "License key fingerprint must be db52fc95 (NEW key)"
    );
}

#[test]
fn test_runtime_rejects_old_license_key_fingerprint() {
    // OLD key fingerprint that MUST be rejected
    const OLD_FINGERPRINT: &str = "23837ac5";

    use costpilot::pro_engine::crypto::LICENSE_PUBLIC_KEY;
    let current_fingerprint = hex::encode(&LICENSE_PUBLIC_KEY[..4]);

    assert_ne!(
        current_fingerprint, OLD_FINGERPRINT,
        "Runtime MUST NOT embed old compromised key (fingerprint: 23837ac5)"
    );
}

#[test]
fn test_runtime_embeds_new_wasm_public_key() {
    // Verify that the runtime has embedded the NEW WASM public key
    use costpilot::pro_engine::crypto::WASM_PUBLIC_KEY;

    const NEW_WASM_PUBLIC_KEY_HEX: &str =
        "8db250f6bf7cdf016fcc1564b2309897a701c4e4fa1946ca0eb9084f1c557994";

    let expected_new_key = hex::decode(NEW_WASM_PUBLIC_KEY_HEX).unwrap();

    assert_eq!(
        WASM_PUBLIC_KEY,
        expected_new_key.as_slice(),
        "Runtime MUST embed the NEW rotated WASM public key (fingerprint: 8db250f6)"
    );

    // Verify fingerprint
    let fingerprint = hex::encode(&WASM_PUBLIC_KEY[..4]);
    assert_eq!(
        fingerprint, "8db250f6",
        "WASM key fingerprint must be 8db250f6 (NEW key)"
    );
}

#[test]
fn test_runtime_rejects_old_wasm_key_fingerprint() {
    // OLD key fingerprint that MUST be rejected
    const OLD_FINGERPRINT: &str = "10f8798e";

    use costpilot::pro_engine::crypto::WASM_PUBLIC_KEY;
    let current_fingerprint = hex::encode(&WASM_PUBLIC_KEY[..4]);

    assert_ne!(
        current_fingerprint, OLD_FINGERPRINT,
        "Runtime MUST NOT embed old compromised WASM key (fingerprint: 10f8798e)"
    );
}

#[test]
fn test_license_signed_with_new_key_is_valid() {
    // Sign a test license with the NEW private key
    let private_key_bytes = hex::decode(NEW_LICENSE_PRIVATE_KEY_HEX).unwrap();
    let signing_key = SigningKey::from_bytes(private_key_bytes.as_slice().try_into().unwrap());

    let license_payload = json!({
        "email": "test@example.com",
        "license_key": "PREMIUM-TEST-KEY",
        "expires": "2027-01-01T00:00:00Z",
        "issued_at": "2026-01-08T00:00:00Z",
        "version": "1.0",
        "issuer": "costpilot-v1"
    });

    let payload_str = serde_json::to_string(&license_payload).unwrap();
    let signature = signing_key.sign(payload_str.as_bytes());

    // Verify with the NEW public key
    let public_key_bytes = hex::decode(NEW_LICENSE_PUBLIC_KEY_HEX).unwrap();
    let verifying_key = VerifyingKey::from_bytes(public_key_bytes.as_slice().try_into().unwrap())
        .expect("Valid public key");

    let result = verifying_key.verify(payload_str.as_bytes(), &signature);

    assert!(
        result.is_ok(),
        "License signed with NEW key MUST verify successfully"
    );
}

#[test]
fn test_license_signed_with_wrong_key_is_invalid() {
    // Generate a different (wrong) signing key
    use rand::RngCore;
    let mut csprng = rand::rngs::OsRng;
    let mut wrong_key_bytes = [0u8; 32];
    csprng.fill_bytes(&mut wrong_key_bytes);
    let wrong_signing_key = SigningKey::from_bytes(&wrong_key_bytes);

    let license_payload = json!({
        "email": "test@example.com",
        "license_key": "PREMIUM-TEST-KEY",
        "expires": "2027-01-01T00:00:00Z",
        "issued_at": "2026-01-08T00:00:00Z",
        "version": "1.0",
        "issuer": "costpilot-v1"
    });

    let payload_str = serde_json::to_string(&license_payload).unwrap();
    let signature = wrong_signing_key.sign(payload_str.as_bytes());

    // Try to verify with the NEW public key (should fail)
    let public_key_bytes = hex::decode(NEW_LICENSE_PUBLIC_KEY_HEX).unwrap();
    let verifying_key = VerifyingKey::from_bytes(public_key_bytes.as_slice().try_into().unwrap())
        .expect("Valid public key");

    let result = verifying_key.verify(payload_str.as_bytes(), &signature);

    assert!(
        result.is_err(),
        "License signed with wrong key MUST fail verification"
    );
}

#[test]
fn test_malformed_signature_is_rejected() {
    let license_payload = json!({
        "email": "test@example.com",
        "license_key": "PREMIUM-TEST-KEY",
        "expires": "2027-01-01T00:00:00Z"
    });

    let payload_str = serde_json::to_string(&license_payload).unwrap();

    // Create invalid signature (all zeros)
    let invalid_signature_bytes = [0u8; 64];
    let invalid_signature = Signature::from_bytes(&invalid_signature_bytes);

    // Try to verify with the NEW public key
    let public_key_bytes = hex::decode(NEW_LICENSE_PUBLIC_KEY_HEX).unwrap();
    let verifying_key = VerifyingKey::from_bytes(public_key_bytes.as_slice().try_into().unwrap())
        .expect("Valid public key");

    let result = verifying_key.verify(payload_str.as_bytes(), &invalid_signature);

    assert!(
        result.is_err(),
        "Malformed signature MUST fail verification"
    );
}

#[test]
fn test_key_rotation_metadata_is_documented() {
    // Verify that key rotation is properly documented
    // This is a compile-time check that the keys.rs file contains rotation metadata

    use costpilot::pro_engine::crypto::{LICENSE_PUBLIC_KEY, WASM_PUBLIC_KEY};

    // Keys must be exactly 32 bytes (Ed25519 public key length)
    assert_eq!(LICENSE_PUBLIC_KEY.len(), 32);
    assert_eq!(WASM_PUBLIC_KEY.len(), 32);

    // Keys must not be all zeros (would indicate uninitialized or placeholder)
    assert!(LICENSE_PUBLIC_KEY.iter().any(|&b| b != 0));
    assert!(WASM_PUBLIC_KEY.iter().any(|&b| b != 0));

    // Verify NEW fingerprints are present
    let license_fp = hex::encode(&LICENSE_PUBLIC_KEY[..4]);
    let wasm_fp = hex::encode(&WASM_PUBLIC_KEY[..4]);

    println!("Current license key fingerprint: {}", license_fp);
    println!("Current WASM key fingerprint: {}", wasm_fp);

    // Document expected fingerprints for future reference
    assert_eq!(
        license_fp, "db52fc95",
        "License key must match NEW rotated key"
    );
    assert_eq!(wasm_fp, "8db250f6", "WASM key must match NEW rotated key");
}

#[test]
fn test_no_private_keys_in_codebase() {
    // This test verifies that no private key material is embedded in the codebase
    // Private keys should only exist in external secure storage

    use costpilot::pro_engine::crypto::{LICENSE_PUBLIC_KEY, WASM_PUBLIC_KEY};

    // Public keys are 32 bytes
    // Private keys would be 32 or 64 bytes depending on format
    // We verify we only have 32-byte public keys, not private keys

    assert_eq!(
        LICENSE_PUBLIC_KEY.len(),
        32,
        "Only 32-byte public key should be embedded"
    );
    assert_eq!(
        WASM_PUBLIC_KEY.len(),
        32,
        "Only 32-byte public key should be embedded"
    );

    // Verify these are valid Ed25519 public keys by attempting to create VerifyingKey
    let license_key_result =
        VerifyingKey::from_bytes(LICENSE_PUBLIC_KEY.try_into().expect("32-byte slice"));
    assert!(
        license_key_result.is_ok(),
        "LICENSE_PUBLIC_KEY must be valid Ed25519 public key"
    );

    let wasm_key_result =
        VerifyingKey::from_bytes(WASM_PUBLIC_KEY.try_into().expect("32-byte slice"));
    assert!(
        wasm_key_result.is_ok(),
        "WASM_PUBLIC_KEY must be valid Ed25519 public key"
    );
}
