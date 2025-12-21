// Cryptographic operations for ProEngine loading

#[cfg(not(target_arch = "wasm32"))]
use ring::{aead, hkdf as ring_hkdf, signature};

#[cfg(not(target_arch = "wasm32"))]
use hex;

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce as AesNonce,
};
use base64::{Engine as _, engine::general_purpose};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use hkdf::SimpleHkdf;
use sha2::Sha256;
use std::fs;
use std::path::Path;

/// Derives a 32-byte AES key using HKDF-SHA256
pub fn hkdf_derive(key_material: &str, salt: &[u8], info: &[u8]) -> [u8; 32] {
    let hk = SimpleHkdf::<Sha256>::new(Some(salt), key_material.as_bytes());
    let mut okm = [0u8; 32];
    hk.expand(info, &mut okm)
        .expect("HKDF expand failed (invalid length)");
    okm
}

/// Decrypts AES-GCM ciphertext
/// Expects: key (32 bytes), nonce (12 bytes), ciphertext with tag appended
pub fn aes_gcm_decrypt(key: &[u8; 32], nonce: &[u8; 12], ciphertext: &[u8]) -> Result<Vec<u8>, String> {
    let cipher = Aes256Gcm::new(key.into());
    let nonce = AesNonce::from_slice(nonce);

    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| "AES-GCM decryption failed".to_string())
}

/// Verifies Ed25519 signature
pub fn ed25519_verify(pubkey_pem: &Path, message: &[u8], sig: &[u8]) -> bool {
    // Read public key PEM
    let pem_data = match fs::read_to_string(pubkey_pem) {
        Ok(d) => d,
        Err(_) => return false,
    };

    // Extract public key bytes from PEM
    let pubkey_bytes = match extract_ed25519_pubkey_from_pem(&pem_data) {
        Some(b) => b,
        None => return false,
    };

    // Parse verifying key
    let verifying_key = match VerifyingKey::from_bytes(&pubkey_bytes) {
        Ok(k) => k,
        Err(_) => return false,
    };

    // Parse signature
    let signature = match Signature::from_slice(sig) {
        Ok(s) => s,
        Err(_) => return false,
    };

    // Verify
    verifying_key.verify(message, &signature).is_ok()
}

/// Extracts Ed25519 public key bytes from PEM format
fn extract_ed25519_pubkey_from_pem(pem: &str) -> Option<[u8; 32]> {
    // Simple PEM parsing - extract base64 between BEGIN/END PUBLIC KEY
    let lines: Vec<&str> = pem.lines().collect();
    let mut base64_data = String::new();
    let mut in_key = false;

    for line in lines {
        if line.contains("BEGIN PUBLIC KEY") {
            in_key = true;
            continue;
        }
        if line.contains("END PUBLIC KEY") {
            break;
        }
        if in_key && !line.trim().is_empty() {
            base64_data.push_str(line.trim());
        }
    }

    // Decode base64
    let der = general_purpose::STANDARD.decode(&base64_data).ok()?;

    // Ed25519 public key in SubjectPublicKeyInfo DER:
    // Last 32 bytes are the raw public key
    if der.len() < 32 {
        return None;
    }

    let mut key_bytes = [0u8; 32];
    key_bytes.copy_from_slice(&der[der.len() - 32..]);
    Some(key_bytes)
}

/// Derive AES-GCM key from license key using HKDF-SHA256 (legacy ring implementation)
#[cfg(not(target_arch = "wasm32"))]
pub fn derive_key(raw_key: &str) -> [u8; 32] {
    let salt = ring_hkdf::Salt::new(ring_hkdf::HKDF_SHA256, b"costpilot-pro-v1");
    let prk = salt.extract(raw_key.as_bytes());

    let mut key = [0u8; 32];
    prk.expand(&[b"aes-gcm-key"], ring_hkdf::HKDF_SHA256)
        .expect("HKDF expand failed")
        .fill(&mut key)
        .expect("HKDF fill failed");

    key
}

/// Decrypt AES-GCM ciphertext
/// Format: nonce (12 bytes) || ciphertext || tag (16 bytes)
#[cfg(not(target_arch = "wasm32"))]
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
#[cfg(not(target_arch = "wasm32"))]
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
#[cfg(not(target_arch = "wasm32"))]
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
