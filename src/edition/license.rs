use serde::{Deserialize, Serialize};
use std::path::Path;

/// License information for Premium edition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct License {
    pub key: String,
    pub email: String,
    pub expires: String,
    pub signature: String,
}

impl License {
    /// Load license from JSON file
    pub fn load_from_file(path: &Path) -> Result<Option<Self>, LicenseError> {
        if !path.exists() {
            return Ok(None);
        }

        let content =
            std::fs::read_to_string(path).map_err(|e| LicenseError::IoError(e.to_string()))?;

        let license: License =
            serde_json::from_str(&content).map_err(|e| LicenseError::ParseError(e.to_string()))?;

        Ok(Some(license))
    }

    /// Verify license signature
    pub fn verify_signature(&self) -> bool {
        use ed25519_dalek::{Signature, Verifier, VerifyingKey};

        // Get the embedded public key
        let public_key_bytes = crate::LICENSE_PUBLIC_KEY;
        let public_key = VerifyingKey::from_bytes(public_key_bytes.try_into().unwrap()).unwrap();

        // Create signature from the stored bytes
        let sig_bytes: [u8; 64] = match self.signature.as_bytes().try_into() {
            Ok(bytes) => bytes,
            Err(_) => return false,
        };
        let signature = Signature::from_bytes(&sig_bytes);

        // Data to verify: email + license_key + expires
        let data = format!("{}{}{}", self.email, self.license_key, self.expires);

        // Verify the signature
        public_key.verify(data.as_bytes(), &signature).is_ok()
    }
}

/// License loading errors
#[derive(Debug)]
pub enum LicenseError {
    IoError(String),
    ParseError(String),
    InvalidSignature,
}

impl std::fmt::Display for LicenseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LicenseError::IoError(msg) => write!(f, "License I/O error: {}", msg),
            LicenseError::ParseError(msg) => write!(f, "License parse error: {}", msg),
            LicenseError::InvalidSignature => write!(f, "Invalid license signature"),
        }
    }
}

impl std::error::Error for LicenseError {}
