// ProEngine WASM loader with encrypted bundle decryption and signature verification

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use chrono::{DateTime, Utc};
use ed25519_dalek::{Signature, VerifyingKey};
use hkdf::Hkdf;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::fs;
use std::path::Path;
use zeroize::Zeroize;

#[derive(Debug)]
pub enum LoaderError {
    Io(String),
    SignatureInvalid,
    DecryptionFailed,
    MissingKeyMaterial,
    BindingMismatch,
    InvalidFormat,
    IntegrityFailure,
}

impl std::fmt::Display for LoaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoaderError::Io(e) => write!(f, "I/O error: {}", e),
            LoaderError::SignatureInvalid => write!(f, "Signature validation failed"),
            LoaderError::DecryptionFailed => write!(f, "Decryption failed"),
            LoaderError::MissingKeyMaterial => write!(f, "Missing key material"),
            LoaderError::BindingMismatch => write!(f, "Machine binding mismatch"),
            LoaderError::InvalidFormat => write!(f, "Invalid bundle format"),
            LoaderError::IntegrityFailure => write!(f, "Integrity check failed"),
        }
    }
}

impl std::error::Error for LoaderError {}

impl From<std::io::Error> for LoaderError {
    fn from(e: std::io::Error) -> Self {
        LoaderError::Io(e.to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseInfo {
    pub license_key: String,
    pub subject: String,
    pub expires: Option<DateTime<Utc>>,
    pub machine_binding: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BundleMetadata {
    #[serde(default)]
    salt: Option<String>,
    alg: String,
    #[serde(default)]
    heuristics_version: Option<String>,
}

pub struct EncryptedBundle {
    pub ciphertext: Vec<u8>,
    pub nonce: [u8; 12],
    pub signature: Vec<u8>,
    pub metadata: serde_json::Value,
    metadata_bytes: Vec<u8>,
}

impl EncryptedBundle {
    pub fn get_metadata_bytes(&self) -> &Vec<u8> {
        &self.metadata_bytes
    }
}

/// Parse encrypted bundle from binary format:
/// [4-byte BE length][metadata JSON][12-byte nonce][ciphertext][64-byte signature]
pub fn parse_bundle(bytes: &[u8]) -> Result<EncryptedBundle, LoaderError> {
    // Minimal, robust parsing logic.
    if bytes.len() < 4 + 12 + 64 {
        return Err(LoaderError::InvalidFormat);
    }

    let metadata_len = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as usize;
    let metadata_start = 4usize;
    let metadata_end = metadata_start.checked_add(metadata_len).ok_or(LoaderError::InvalidFormat)?;

    // signature occupies the last 64 bytes
    let signature_len = 64usize;
    if bytes.len() < signature_len {
        return Err(LoaderError::InvalidFormat);
    }
    let signature_start = bytes.len() - signature_len;

    // nonce must be directly after metadata and before ciphertext
    let nonce_start = metadata_end;
    let nonce_end = nonce_start.checked_add(12).ok_or(LoaderError::InvalidFormat)?;

    // validate ranges
    if metadata_end > bytes.len() || nonce_end > signature_start || nonce_end >= signature_start {
        return Err(LoaderError::InvalidFormat);
    }

    let metadata_bytes = bytes[metadata_start..metadata_end].to_vec();
    let metadata: serde_json::Value = serde_json::from_slice(&metadata_bytes).map_err(|_| LoaderError::InvalidFormat)?;

    let nonce: [u8; 12] = bytes[nonce_start..nonce_end].try_into().map_err(|_| LoaderError::InvalidFormat)?;

    let ciphertext = bytes[nonce_end..signature_start].to_vec();
    if ciphertext.is_empty() {
        return Err(LoaderError::InvalidFormat);
    }

    let signature = bytes[signature_start..].to_vec();
    if signature.len() != signature_len {
        return Err(LoaderError::InvalidFormat);
    }

    Ok(EncryptedBundle {
        ciphertext,
        nonce,
        signature,
        metadata,
        metadata_bytes,
    })
}

/// Verify Ed25519 signature over signed region (metadata + nonce + ciphertext)
pub fn verify_signature(bundle: &EncryptedBundle, public_key: &[u8]) -> Result<(), LoaderError> {
    if public_key.len() != 32 {
        return Err(LoaderError::SignatureInvalid);
    }

    if bundle.signature.len() != 64 {
        return Err(LoaderError::SignatureInvalid);
    }

    let mut signed_data = Vec::with_capacity(
        bundle.metadata_bytes.len() + bundle.nonce.len() + bundle.ciphertext.len(),
    );
    signed_data.extend_from_slice(&bundle.metadata_bytes);
    signed_data.extend_from_slice(&bundle.nonce);
    signed_data.extend_from_slice(&bundle.ciphertext);

    let verifying_key = VerifyingKey::from_bytes(
        public_key
            .try_into()
            .map_err(|_| LoaderError::SignatureInvalid)?,
    )
    .map_err(|_| LoaderError::SignatureInvalid)?;

    let signature = Signature::from_slice(&bundle.signature).map_err(|_| LoaderError::SignatureInvalid)?;

    verifying_key
        .verify_strict(&signed_data, &signature)
        .map_err(|_| LoaderError::SignatureInvalid)?;

    Ok(())
}

/// Derive AES-256 decryption key from license info using HKDF-SHA256
pub fn derive_decryption_key(license: &LicenseInfo, salt: &[u8]) -> Result<[u8; 32], LoaderError> {
    if license.license_key.is_empty() {
        return Err(LoaderError::MissingKeyMaterial);
    }

    let mut ikm = Vec::new();
    ikm.extend_from_slice(license.license_key.as_bytes());
    ikm.extend_from_slice(license.subject.as_bytes());
    if let Some(ref binding) = license.machine_binding {
        ikm.extend_from_slice(binding.as_bytes());
    }

    let hk = Hkdf::<Sha256>::new(Some(salt), &ikm);
    let mut okm = [0u8; 32];
    hk.expand(b"costpilot-pro-engine-v1", &mut okm)
        .map_err(|_| LoaderError::MissingKeyMaterial)?;

    ikm.zeroize();

    Ok(okm)
}

/// Decrypt AES-GCM bundle using derived key
pub fn decrypt_bundle(bundle: &EncryptedBundle, key: &[u8; 32]) -> Result<Vec<u8>, LoaderError> {
    let cipher = Aes256Gcm::new(key.into());
    let nonce = Nonce::from_slice(&bundle.nonce);

    let aad = &bundle.metadata_bytes;

    let plaintext = cipher
        .decrypt(
            nonce,
            aes_gcm::aead::Payload {
                msg: &bundle.ciphertext,
                aad,
            },
        )
        .map_err(|_| LoaderError::DecryptionFailed)?;

    Ok(plaintext)
}

/// Load and decrypt Pro Engine WASM from file
pub fn load_pro_engine_from_file(
    path: &Path,
    license: &LicenseInfo,
    public_key: &[u8],
) -> Result<Vec<u8>, LoaderError> {
    let bundle_bytes = fs::read(path)?;

    let bundle = parse_bundle(&bundle_bytes)?;

    verify_signature(&bundle, public_key)?;

    let metadata: BundleMetadata =
        serde_json::from_value(bundle.metadata.clone()).map_err(|_| LoaderError::InvalidFormat)?;
    let salt = metadata
        .salt
        .as_ref()
        .map(|s| s.as_bytes())
        .unwrap_or(b"costpilot-pro-v1");

    let mut key = derive_decryption_key(license, salt)?;

    let plaintext = decrypt_bundle(&bundle, &key)?;

    key.zeroize();

    if plaintext.len() < 4 || &plaintext[0..4] != b"\0asm" {
        return Err(LoaderError::IntegrityFailure);
    }

    Ok(plaintext)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::path::PathBuf;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_bundle_valid() {
        // Create a valid bundle: 4-byte len + metadata + 12-byte nonce + ciphertext + 64-byte sig
        let metadata = r#"{"alg":"AES256-GCM","salt":"test-salt"}"#;
        let metadata_bytes = metadata.as_bytes();
        let metadata_len = (metadata_bytes.len() as u32).to_be_bytes();

        let nonce = [1u8; 12];
        let ciphertext = b"encrypted data";
        let signature = [2u8; 64];

        let mut bundle_bytes = Vec::new();
        bundle_bytes.extend_from_slice(&metadata_len);
        bundle_bytes.extend_from_slice(metadata_bytes);
        bundle_bytes.extend_from_slice(&nonce);
        bundle_bytes.extend_from_slice(ciphertext);
        bundle_bytes.extend_from_slice(&signature);

        let bundle = parse_bundle(&bundle_bytes).unwrap();
        assert_eq!(bundle.ciphertext, ciphertext);
        assert_eq!(bundle.nonce, nonce);
        assert_eq!(bundle.signature, signature);
    }

    #[test]
    fn test_parse_bundle_too_short() {
        let short_bytes = vec![0u8; 10];
        assert!(matches!(
            parse_bundle(&short_bytes),
            Err(LoaderError::InvalidFormat)
        ));
    }

    #[test]
    fn test_parse_bundle_invalid_metadata_len() {
        let mut bytes = vec![0u8; 100];
        // Set metadata len to larger than available
        bytes[0..4].copy_from_slice(&(100u32.to_be_bytes()));
        assert!(matches!(
            parse_bundle(&bytes),
            Err(LoaderError::InvalidFormat)
        ));
    }

    #[test]
    fn test_parse_bundle_invalid_json() {
        let invalid_json = b"invalid json";
        let metadata_len = (invalid_json.len() as u32).to_be_bytes();
        let nonce = [0u8; 12];
        let ciphertext = b"data";
        let signature = [0u8; 64];

        let mut bundle_bytes = Vec::new();
        bundle_bytes.extend_from_slice(&metadata_len);
        bundle_bytes.extend_from_slice(invalid_json);
        bundle_bytes.extend_from_slice(&nonce);
        bundle_bytes.extend_from_slice(ciphertext);
        bundle_bytes.extend_from_slice(&signature);

        assert!(matches!(
            parse_bundle(&bundle_bytes),
            Err(LoaderError::InvalidFormat)
        ));
    }

    #[test]
    fn test_verify_signature_valid() {
        use ed25519_dalek::Signer;

        // Use a fixed key for deterministic test
        let secret_key_bytes = [1u8; 32];
        let secret_key = ed25519_dalek::SecretKey::from(secret_key_bytes);
        let signing_key = ed25519_dalek::SigningKey::from(&secret_key);
        let public_key = ed25519_dalek::VerifyingKey::from(&signing_key);
        let keypair_bytes = [secret_key_bytes, public_key.to_bytes()].concat();
        let signing_key =
            ed25519_dalek::SigningKey::from_keypair_bytes(&keypair_bytes.try_into().unwrap())
                .unwrap();

        let data = b"test data";
        let mut signed_data = Vec::new();
        signed_data.extend_from_slice(b"{}"); // metadata_bytes
        signed_data.extend_from_slice(&[0u8; 12]); // nonce
        signed_data.extend_from_slice(data);

        let signature = signing_key.sign(&signed_data);

        let bundle = EncryptedBundle {
            ciphertext: data.to_vec(),
            nonce: [0u8; 12],
            signature: signature.to_bytes().to_vec(),
            metadata: serde_json::json!({"test": "value"}),
            metadata_bytes: b"{}".to_vec(),
        };

        assert!(verify_signature(&bundle, &public_key.to_bytes()).is_ok());
    }

    #[test]
    fn test_verify_signature_invalid_key_length() {
        let bundle = EncryptedBundle {
            ciphertext: vec![],
            nonce: [0u8; 12],
            signature: vec![0u8; 64],
            metadata: serde_json::json!({}),
            metadata_bytes: vec![],
        };

        assert!(matches!(
            verify_signature(&bundle, &[0u8; 31]),
            Err(LoaderError::SignatureInvalid)
        ));
    }

    #[test]
    fn test_verify_signature_invalid_signature() {
        let bundle = EncryptedBundle {
            ciphertext: vec![],
            nonce: [0u8; 12],
            signature: vec![0u8; 64], // Invalid signature
            metadata: serde_json::json!({}),
            metadata_bytes: vec![],
        };

        let key = [0u8; 32];
        assert!(matches!(
            verify_signature(&bundle, &key),
            Err(LoaderError::SignatureInvalid)
        ));
    }

    #[test]
    fn test_derive_decryption_key_valid() {
        let license = LicenseInfo {
            license_key: "test-key".to_string(),
            subject: "test-subject".to_string(),
            expires: None,
            machine_binding: Some("test-binding".to_string()),
        };

        let salt = b"test-salt";
        let key = derive_decryption_key(&license, salt).unwrap();
        assert_eq!(key.len(), 32);
    }

    #[test]
    fn test_derive_decryption_key_empty_license() {
        let license = LicenseInfo {
            license_key: "".to_string(),
            subject: "subject".to_string(),
            expires: None,
            machine_binding: None,
        };

        let salt = b"salt";
        assert!(matches!(
            derive_decryption_key(&license, salt),
            Err(LoaderError::MissingKeyMaterial)
        ));
    }

    #[test]
    fn test_decrypt_bundle_valid() {
        // For testing decryption, we need to encrypt some data first.
        use aes_gcm::aead::Aead;

        let key = [1u8; 32];
        let cipher = Aes256Gcm::new(&key.into());
        let nonce = Nonce::from_slice(&[2u8; 12]);
        let aad = b"metadata";
        let plaintext = b"test plaintext";

        let ciphertext = cipher
            .encrypt(
                nonce,
                aes_gcm::aead::Payload {
                    msg: plaintext,
                    aad,
                },
            )
            .unwrap();

        let bundle = EncryptedBundle {
            ciphertext,
            nonce: (*nonce).into(),
            signature: vec![], // Not used in decrypt
            metadata: serde_json::json!({}),
            metadata_bytes: aad.to_vec(),
        };

        let decrypted = decrypt_bundle(&bundle, &key).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_decrypt_bundle_invalid_key() {
        let bundle = EncryptedBundle {
            ciphertext: vec![0u8; 16],
            nonce: [0u8; 12],
            signature: vec![],
            metadata: serde_json::json!({}),
            metadata_bytes: vec![],
        };

        let wrong_key = [0u8; 32];
        assert!(matches!(
            decrypt_bundle(&bundle, &wrong_key),
            Err(LoaderError::DecryptionFailed)
        ));
    }

    #[test]
    fn test_load_pro_engine_from_file_nonexistent() {
        let license = LicenseInfo {
            license_key: "key".to_string(),
            subject: "subject".to_string(),
            expires: None,
            machine_binding: None,
        };

        let path = PathBuf::from("nonexistent.wasm");
        let key = [0u8; 32];

        assert!(matches!(
            load_pro_engine_from_file(&path, &license, &key),
            Err(LoaderError::Io(_))
        ));
    }

    #[test]
    fn test_load_pro_engine_from_file_invalid_wasm() {
        let license = LicenseInfo {
            license_key: "key".to_string(),
            subject: "subject".to_string(),
            expires: None,
            machine_binding: None,
        };

        // Create a temp file with invalid data
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"not a wasm file").unwrap();
        temp_file.flush().unwrap();

        let key = [0u8; 32];

        // This will fail at parse_bundle since it's not a valid bundle
        assert!(matches!(
            load_pro_engine_from_file(temp_file.path(), &license, &key),
            Err(LoaderError::InvalidFormat)
        ));
    }
}
