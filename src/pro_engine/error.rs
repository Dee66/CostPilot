use std::fmt;

#[derive(Debug)]
pub enum ProEngineLoadError {
    MissingFile(String),
    SignatureMismatch,
    DecryptionFailed,
    InvalidWasm,
    IncompatibleInterface { expected: String, found: String },
    NotImplemented,
}

impl fmt::Display for ProEngineLoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingFile(path) => write!(f, "ProEngine WASM file not found: {}", path),
            Self::SignatureMismatch => write!(f, "ProEngine WASM signature verification failed"),
            Self::DecryptionFailed => write!(f, "Failed to decrypt ProEngine WASM"),
            Self::InvalidWasm => write!(f, "Invalid WASM format in ProEngine binary"),
            Self::IncompatibleInterface { expected, found } => {
                write!(
                    f,
                    "ProEngine interface version mismatch: expected {}, found {}",
                    expected, found
                )
            }
            Self::NotImplemented => write!(f, "ProEngine WASM instantiation not yet implemented"),
        }
    }
}

impl std::error::Error for ProEngineLoadError {}
