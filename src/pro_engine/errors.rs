// ProEngine error types

use std::fmt;

#[derive(Debug)]
pub enum ProEngineError {
    IoError(std::io::Error),
    InvalidLicense,
    DecryptionFailed,
    SignatureFailed,
    MissingEngine,
    UnsupportedPlatform,
}

impl fmt::Display for ProEngineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProEngineError::IoError(e) => write!(f, "IO error: {}", e),
            ProEngineError::InvalidLicense => write!(f, "Invalid license"),
            ProEngineError::DecryptionFailed => write!(f, "Decryption failed"),
            ProEngineError::SignatureFailed => write!(f, "Signature verification failed"),
            ProEngineError::MissingEngine => write!(f, "Pro engine not found"),
            ProEngineError::UnsupportedPlatform => write!(f, "Unsupported platform"),
        }
    }
}

impl std::error::Error for ProEngineError {}

impl From<std::io::Error> for ProEngineError {
    fn from(err: std::io::Error) -> Self {
        ProEngineError::IoError(err)
    }
}
