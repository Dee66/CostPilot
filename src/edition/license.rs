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

    /// Verify license signature (stub implementation)
    pub fn verify_signature(&self) -> bool {
        // Stub: return true for now
        true
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
