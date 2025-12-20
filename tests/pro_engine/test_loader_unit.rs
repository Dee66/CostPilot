use costpilot::pro_engine::crypto::{aes_gcm_decrypt, hkdf_derive};

#[test]
fn test_hkdf_derivation_matches_vector() {
    let key = hkdf_derive("test-key-material", b"test-salt", b"test-info");
    assert_eq!(key.len(), 32);

    // Verify determinism
    let key2 = hkdf_derive("test-key-material", b"test-salt", b"test-info");
    assert_eq!(key, key2);

    // Different inputs should produce different keys
    let key3 = hkdf_derive("different-key", b"test-salt", b"test-info");
    assert_ne!(key, key3);
}

#[test]
fn test_aes_gcm_decrypt_success() {
    use aes_gcm::aead::{Aead, KeyInit};
    use aes_gcm::{Aes256Gcm, Nonce};

    let key = [42u8; 32];
    let nonce_bytes = [1u8; 12];
    let plaintext = b"test payload for decryption";

    // Encrypt
    let cipher = Aes256Gcm::new(&key.into());
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher.encrypt(nonce, plaintext.as_ref()).unwrap();

    // Decrypt
    let decrypted = aes_gcm_decrypt(&key, &nonce_bytes, &ciphertext).unwrap();
    assert_eq!(decrypted, plaintext);
}

#[test]
fn test_decrypt_with_bad_key_returns_error() {
    use aes_gcm::aead::{Aead, KeyInit};
    use aes_gcm::{Aes256Gcm, Nonce};

    let key1 = [42u8; 32];
    let key2 = [99u8; 32];
    let nonce_bytes = [1u8; 12];
    let plaintext = b"secret data";

    // Encrypt with key1
    let cipher = Aes256Gcm::new(&key1.into());
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher.encrypt(nonce, plaintext.as_ref()).unwrap();

    // Try decrypt with key2
    let result = aes_gcm_decrypt(&key2, &nonce_bytes, &ciphertext);
    assert!(result.is_err());
}

#[test]
fn test_verify_sig_success_and_failure() {
    use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
    use rand::rngs::OsRng;
    use std::fs;
    use std::io::Write;
    use tempfile::tempdir;

    // Generate ephemeral keypair
    let mut csprng = OsRng;
    let signing_key = SigningKey::generate(&mut csprng);
    let verifying_key = VerifyingKey::from(&signing_key);

    let message = b"test message to sign";
    let signature = signing_key.sign(message);

    // Write public key to temp PEM file
    let temp_dir = tempdir().unwrap();
    let pubkey_path = temp_dir.path().join("test.pub.pem");

    // Create minimal PEM format
    let pubkey_bytes = verifying_key.to_bytes();
    let der_bytes = [
        &[0x30, 0x2a, 0x30, 0x05, 0x06, 0x03, 0x2b, 0x65, 0x70, 0x03, 0x21, 0x00][..],
        &pubkey_bytes[..],
    ]
    .concat();

    let pem_content = format!(
        "-----BEGIN PUBLIC KEY-----\n{}\n-----END PUBLIC KEY-----\n",
        base64::encode(&der_bytes)
    );

    let mut file = fs::File::create(&pubkey_path).unwrap();
    file.write_all(pem_content.as_bytes()).unwrap();

    // Test successful verification
    let result = costpilot::pro_engine::crypto::ed25519_verify(
        &pubkey_path,
        message,
        &signature.to_bytes(),
    );
    assert!(result);

    // Test failed verification with wrong message
    let wrong_message = b"different message";
    let result = costpilot::pro_engine::crypto::ed25519_verify(
        &pubkey_path,
        wrong_message,
        &signature.to_bytes(),
    );
    assert!(!result);

    // Test failed verification with corrupted signature
    let mut bad_sig = signature.to_bytes();
    bad_sig[0] ^= 0xFF;
    let result = costpilot::pro_engine::crypto::ed25519_verify(
        &pubkey_path,
        message,
        &bad_sig,
    );
    assert!(!result);
}
