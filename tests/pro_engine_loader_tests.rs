// Pro Engine loader tests with encrypted bundle simulation

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use chrono::Utc;
use costpilot::pro_engine::{load_pro_engine_from_file, LicenseInfo, LoaderError};
use curve25519_dalek::scalar::Scalar;
use ed25519_dalek::{Signature, Signer, SigningKey};
use rand::rngs::OsRng;
use rand::RngCore; // Import the RngCore trait to bring `fill_bytes` into scope
use std::fs;
use tempfile::tempdir;

fn create_test_bundle(
    wasm_bytes: &[u8],
    license: &LicenseInfo,
    signing_key: &SigningKey,
) -> Vec<u8> {
    // Derive encryption key
    let salt = b"test-salt";
    let key = costpilot::pro_engine::loader::derive_decryption_key(license, salt).unwrap();

    // Prepare metadata
    let metadata = serde_json::json!({
        "alg": "AES-GCM-256",
        "salt": "test-salt",
        "heuristics_version": "1.0.0"
    });
    let metadata_bytes = serde_json::to_vec(&metadata).unwrap();
    let metadata_len = metadata_bytes.len() as u32;

    // Generate random nonce
    let mut nonce_bytes = [0u8; 12];
    rand::RngCore::fill_bytes(&mut OsRng, &mut nonce_bytes);

    // Encrypt with AAD
    let cipher = Aes256Gcm::new(&key.into());
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher
        .encrypt(
            nonce,
            aes_gcm::aead::Payload {
                msg: wasm_bytes,
                aad: &metadata_bytes,
            },
        )
        .unwrap();

    // Sign: metadata + nonce + ciphertext
    let mut signed_data = Vec::new();
    signed_data.extend_from_slice(&metadata_bytes);
    signed_data.extend_from_slice(&nonce_bytes);
    signed_data.extend_from_slice(&ciphertext);
    let signature: Signature = signing_key.sign(&signed_data);

    // omitted debug logging in production test run

    // Assemble bundle: [len][metadata][nonce][ciphertext][signature]
    let mut bundle = Vec::new();
    bundle.extend_from_slice(&metadata_len.to_be_bytes());
    bundle.extend_from_slice(&metadata_bytes);
    bundle.extend_from_slice(&nonce_bytes);
    bundle.extend_from_slice(&ciphertext);
    bundle.extend_from_slice(&signature.to_bytes());

    // omitted immediate signature validation logging

    bundle
}

// Refine the signing key generation logic to ensure valid scalars
fn generate_valid_signing_key() -> SigningKey {
    loop {
        let mut secret_key_bytes = [0u8; 32];
        OsRng.fill_bytes(&mut secret_key_bytes);

        // Ensure the scalar's high bit is not set and the scalar is clamped
        secret_key_bytes[0] &= 248; // Clamp the first byte
        secret_key_bytes[31] &= 127; // Clamp the last byte
        secret_key_bytes[31] |= 64; // Set the second highest bit

        // Validate the scalar to ensure it is within the Ed25519 curve's range
        if let Some(_) = <subtle::CtOption<Scalar> as Into<Option<Scalar>>>::into(Scalar::from_canonical_bytes(secret_key_bytes)) {
            return SigningKey::from_bytes(&secret_key_bytes);
        }

        // regenerate silently
    }
}

#[test]
fn test_parse_and_verify_signature_ok() {
    let wat = r#"(module (func (export "test") (result i32) i32.const 1))"#;
    let wasm_bytes = wat::parse_str(wat).unwrap();

    let license = LicenseInfo {
        license_key: "test-key-12345".to_string(),
        subject: "test@example.com".to_string(),
        expires: Some(Utc::now() + chrono::Duration::days(30)),
        machine_binding: None,
    };

    // Generate keypair with valid scalar
    let signing_key = generate_valid_signing_key();

    let verifying_key = signing_key.verifying_key();

    // Create bundle
    let bundle_bytes = create_test_bundle(&wasm_bytes, &license, &signing_key);

    // Parse bundle
    let bundle = costpilot::pro_engine::loader::parse_bundle(&bundle_bytes).unwrap();

    // Verify signature
    let result = costpilot::pro_engine::loader::verify_signature(&bundle, verifying_key.as_bytes());
    assert!(result.is_ok());
}

#[test]
fn test_decrypt_with_correct_license_ok() {
    let wat = r#"(module (func (export "test") (result i32) i32.const 42))"#;
    let wasm_bytes = wat::parse_str(wat).unwrap();

    let license = LicenseInfo {
        license_key: "correct-license-key".to_string(),
        subject: "user@test.com".to_string(),
        expires: Some(Utc::now() + chrono::Duration::days(30)),
        machine_binding: Some("machine-123".to_string()),
    };

    // Generate keypair with valid scalar
    let signing_key = generate_valid_signing_key();

    let verifying_key = signing_key.verifying_key();

    // Create bundle and write to temp file
    let dir = tempdir().unwrap();
    let bundle_path = dir.path().join("test.bundle");
    let bundle_bytes = create_test_bundle(&wasm_bytes, &license, &signing_key);
    fs::write(&bundle_path, bundle_bytes).unwrap();

    // Load and decrypt
    let decrypted =
        load_pro_engine_from_file(&bundle_path, &license, verifying_key.as_bytes()).unwrap();

    // Verify WASM magic
    assert_eq!(&decrypted[0..4], b"\0asm");
    assert_eq!(decrypted, wasm_bytes);
}

#[test]
fn test_decrypt_with_wrong_license_fails() {
    let wat = r#"(module (func (export "test") (result i32) i32.const 1))"#;
    let wasm_bytes = wat::parse_str(wat).unwrap();

    let correct_license = LicenseInfo {
        license_key: "correct-key".to_string(),
        subject: "user@test.com".to_string(),
        expires: Some(Utc::now() + chrono::Duration::days(30)),
        machine_binding: None,
    };

    let wrong_license = LicenseInfo {
        license_key: "wrong-key".to_string(),
        subject: "user@test.com".to_string(),
        expires: Some(Utc::now() + chrono::Duration::days(30)),
        machine_binding: None,
    };

    // Generate keypair with valid scalar
    let signing_key = generate_valid_signing_key();

    let verifying_key = signing_key.verifying_key();

    // Create bundle with correct license
    let dir = tempdir().unwrap();
    let bundle_path = dir.path().join("test.bundle");
    let bundle_bytes = create_test_bundle(&wasm_bytes, &correct_license, &signing_key);
    fs::write(&bundle_path, bundle_bytes).unwrap();

    // Try to decrypt with wrong license
    let result = load_pro_engine_from_file(&bundle_path, &wrong_license, verifying_key.as_bytes());

    assert!(matches!(result, Err(LoaderError::DecryptionFailed)));
}

#[test]
fn test_signature_tamper_detected() {
    let wat = r#"(module (func (export "test") (result i32) i32.const 1))"#;
    let wasm_bytes = wat::parse_str(wat).unwrap();

    let license = LicenseInfo {
        license_key: "test-key".to_string(),
        subject: "user@test.com".to_string(),
        expires: Some(Utc::now() + chrono::Duration::days(30)),
        machine_binding: None,
    };

    // Generate keypair with valid scalar
    let signing_key = generate_valid_signing_key();

    let verifying_key = signing_key.verifying_key();

    // Create bundle and tamper with ciphertext
    let mut bundle_bytes = create_test_bundle(&wasm_bytes, &license, &signing_key);

    // Tamper: flip a bit in ciphertext region (after metadata+nonce, before signature)
    let tamper_offset = 100; // somewhere in the middle
    if bundle_bytes.len() > tamper_offset + 64 {
        bundle_bytes[tamper_offset] ^= 0xFF;
    }

    // Write to temp file
    let dir = tempdir().unwrap();
    let bundle_path = dir.path().join("tampered.bundle");
    fs::write(&bundle_path, bundle_bytes).unwrap();

    // Try to load - should fail signature verification
    let result = load_pro_engine_from_file(&bundle_path, &license, verifying_key.as_bytes());

    assert!(matches!(result, Err(LoaderError::SignatureInvalid)));
}
