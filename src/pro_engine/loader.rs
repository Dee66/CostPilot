// ProEngine WASM loader with decryption and WIT bindings

use super::error::ProEngineLoadError;
use super::errors::ProEngineError;
use super::ProEngineHandle;
use crate::edition::license::License;
use crate::edition::EditionContext;
use aes_gcm::aead::Aead;
use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

pub struct ProEngineLoader;

impl ProEngineLoader {
    /// Load encrypted WASM engine and decrypt it
    pub fn load(path: &Path, license: &License) -> Result<ProEngineHandle, ProEngineError> {
        // Read encrypted WASM bytes
        let ciphertext = fs::read(path)?;

        // Derive AES-GCM key from license.key using HKDF-SHA256 (stub)
        let _encryption_key = Self::derive_key(&license.key);

        // Decrypt the buffer (stub: plaintext = ciphertext for now)
        let _plaintext = Self::decrypt_stub(&ciphertext, &_encryption_key)?;

        // Verify signature via license (stub: always true)
        if !license.verify_signature() {
            return Err(ProEngineError::SignatureFailed);
        }

        // Return stub handle with decrypted bytes (no executor yet)
        Ok(ProEngineHandle::stub())
    }

    /// Stub: derive key from license.key using HKDF-SHA256
    fn derive_key(license_key: &str) -> Vec<u8> {
        // Placeholder: return license_key as bytes
        // Real implementation: use HKDF with SHA256
        license_key.as_bytes().to_vec()
    }

    /// Stub: decrypt ciphertext (currently returns plaintext = ciphertext)
    fn decrypt_stub(ciphertext: &[u8], _key: &[u8]) -> Result<Vec<u8>, ProEngineError> {
        // Placeholder: no actual decryption
        // Real implementation: AES-GCM decryption
        Ok(ciphertext.to_vec())
    }
}

/// WIT-based WASM loader using AES-GCM decryption
pub fn load_pro_engine_wit(edition: &EditionContext) -> Result<Option<ProEngineHandle>> {
    if edition.is_free() {
        return Ok(None);
    }

    // Attempt to load encrypted WASM
    let wasm_path = edition.paths.pro_wasm_path();
    let encrypted = match fs::read(&wasm_path) {
        Ok(data) => data,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Err(ProEngineLoadError::MissingFile(wasm_path.display().to_string()).into());
        }
        Err(e) => return Err(e).context("Failed to read ProEngine WASM"),
    };

    // Derive encryption key from license
    let key = edition
        .derive_key()
        .context("Failed to derive encryption key")?;

    // Derive deterministic nonce
    let nonce_bytes = edition.derive_nonce().context("Failed to derive nonce")?;

    // Decrypt WASM
    let cipher = Aes256Gcm::new_from_slice(&key).context("Failed to initialize cipher")?;
    let nonce = Nonce::from_slice(&nonce_bytes);

    let decrypted = cipher
        .decrypt(nonce, encrypted.as_ref())
        .map_err(|_| ProEngineLoadError::DecryptionFailed)?;

    // Validate WASM magic bytes
    if decrypted.len() < 4 || &decrypted[0..4] != b"\0asm" {
        return Err(ProEngineLoadError::InvalidWasm.into());
    }

    // TODO: Instantiate WASM module using wit-bindgen runtime
    // This will be implemented once wit-bindgen integration is complete
    // For now, return placeholder
    Err(ProEngineLoadError::NotImplemented.into())
}
