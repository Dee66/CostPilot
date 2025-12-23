// ProEngine error types

use std::fmt;

#[derive(Debug)]
pub enum ProEngineError {
    IoError(std::io::Error),
    InvalidLicense,
    LicenseInvalid,
    LicenseExpired,
    DecryptionFailed,
    DecryptFailed,
    SignatureFailed,
    MissingEngine,
    UnsupportedPlatform,
    LoadFailed(String),
}

impl fmt::Display for ProEngineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProEngineError::IoError(e) => write!(f, "IO error: {}", e),
            ProEngineError::InvalidLicense => write!(f, "Invalid license"),
            ProEngineError::LicenseInvalid => write!(f, "Invalid license"),
            ProEngineError::LicenseExpired => write!(f, "License expired"),
            ProEngineError::DecryptionFailed => write!(f, "Decryption failed"),
            ProEngineError::DecryptFailed => write!(f, "Decryption failed"),
            ProEngineError::SignatureFailed => write!(f, "Signature verification failed"),
            ProEngineError::MissingEngine => write!(f, "Pro engine not found"),
            ProEngineError::UnsupportedPlatform => write!(f, "Unsupported platform"),
            ProEngineError::LoadFailed(msg) => write!(f, "Load failed: {}", msg),
        }
    }
}

impl std::error::Error for ProEngineError {}

impl From<std::io::Error> for ProEngineError {
    fn from(err: std::io::Error) -> Self {
        ProEngineError::IoError(err)
    }
}
