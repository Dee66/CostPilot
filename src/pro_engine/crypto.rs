// Cryptographic operations for ProEngine loading

use ring::{aead, hkdf, signature};

/// Derive AES-GCM key from license key using HKDF-SHA256
pub fn derive_key(raw_key: &str) -> [u8; 32] {
    let salt = hkdf::Salt::new(hkdf::HKDF_SHA256, b"costpilot-pro-v1");
    let prk = salt.extract(raw_key.as_bytes());

    let mut key = [0u8; 32];
    prk.expand(&[b"aes-gcm-key"], hkdf::HKDF_SHA256)
        .expect("HKDF expand failed")
        .fill(&mut key)
        .expect("HKDF fill failed");

    key
}

/// Decrypt AES-GCM ciphertext
/// Format: nonce (12 bytes) || ciphertext || tag (16 bytes)
pub fn decrypt_aes_gcm(ct: &[u8], key: &[u8; 32]) -> Result<Vec<u8>, String> {
    if ct.len() < 28 {
        return Err("Ciphertext too short".to_string());
    }

    let nonce = &ct[0..12];
    let ciphertext_and_tag = &ct[12..];

    let unbound_key = aead::UnboundKey::new(&aead::AES_256_GCM, key).map_err(|_| "Invalid key")?;
    let key = aead::LessSafeKey::new(unbound_key);

    let nonce = aead::Nonce::try_assume_unique_for_key(nonce).map_err(|_| "Invalid nonce")?;

    let mut buffer = ciphertext_and_tag.to_vec();
    let plaintext_len = key
        .open_in_place(nonce, aead::Aad::empty(), &mut buffer)
        .map_err(|_| "Decryption failed")?
        .len();

    buffer.truncate(plaintext_len);
    Ok(buffer)
}

/// Verify WASM signature using Ed25519
pub fn verify_wasm_signature(wasm: &[u8], sig: &[u8]) -> Result<(), String> {
    // Hardcoded public key (in production, this would be embedded in binary)
    const PUBLIC_KEY: &[u8] = &[
        // Placeholder: 32-byte Ed25519 public key
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0,
    ];

    let public_key = signature::UnparsedPublicKey::new(&signature::ED25519, PUBLIC_KEY);
    public_key
        .verify(wasm, sig)
        .map_err(|_| "WASM signature verification failed".to_string())
}

/// Verify license signature
pub fn verify_license_signature(lic: &super::license::License) -> Result<(), String> {
    // Construct canonical signing message
    let message = format!("{}|{}|{}", lic.email, lic.license_key, lic.expires);

    // Decode signature from hex
    let sig_bytes = hex::decode(&lic.signature).map_err(|_| "Invalid signature format")?;

    // Hardcoded public key
    const PUBLIC_KEY: &[u8] = &[
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0,
    ];

    let public_key = signature::UnparsedPublicKey::new(&signature::ED25519, PUBLIC_KEY);
    public_key
        .verify(message.as_bytes(), &sig_bytes)
        .map_err(|_| "License signature verification failed".to_string())
}
