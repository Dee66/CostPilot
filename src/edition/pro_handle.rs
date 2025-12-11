use crate::pro_engine::{ProEngineExecutor, ProEngineRequest, ProEngineResponse};
use std::path::PathBuf;

/// Handle to Premium engine with execution capability
#[derive(Clone)]
pub struct ProEngineHandle {
    pub path: PathBuf,
    pub decrypted_wasm: Option<Vec<u8>>,
    // Executor wrapped in Arc for thread-safe cloning
    executor: Option<std::sync::Arc<dyn ProEngineExecutor + Send + Sync>>,
}

impl ProEngineHandle {
    /// Create handle with executor
    pub fn with_executor(
        path: PathBuf,
        decrypted_wasm: Option<Vec<u8>>,
        executor: Box<dyn ProEngineExecutor + Send + Sync>,
    ) -> Self {
        Self {
            path,
            decrypted_wasm,
            executor: Some(std::sync::Arc::from(executor)),
        }
    }

    /// Create stub handle without executor (for testing/stub loading)
    pub fn stub(path: PathBuf) -> Self {
        Self {
            path,
            decrypted_wasm: None,
            executor: None,
        }
    }

    /// Execute scan request (wrapper for execute)
    pub fn scan(&self, input: &[u8]) -> Result<Vec<u8>, String> {
        let _input_str =
            std::str::from_utf8(input).map_err(|e| format!("Invalid UTF-8 input: {}", e))?;
        let req = ProEngineRequest::Predict {
            changes: vec![], // Placeholder - actual parsing done in WASM
        };
        let resp = self.execute(req)?;
        match resp {
            ProEngineResponse::Predict(estimates) => {
                // Serialize estimates back to bytes
                let json = serde_json::to_string(&estimates)
                    .map_err(|e| format!("Serialization error: {}", e))?;
                Ok(json.into_bytes())
            }
            _ => Err("Unexpected response type".to_string()),
        }
    }

    /// Execute predict request (wrapper for execute)
    pub fn predict(&self, input: &[u8]) -> Result<Vec<u8>, String> {
        let _input_str =
            std::str::from_utf8(input).map_err(|e| format!("Invalid UTF-8 input: {}", e))?;
        let req = ProEngineRequest::Predict {
            changes: vec![], // Placeholder - actual parsing done in WASM
        };
        let resp = self.execute(req)?;
        match resp {
            ProEngineResponse::Predict(estimates) => {
                // Serialize estimates back to bytes
                let json = serde_json::to_string(&estimates)
                    .map_err(|e| format!("Serialization error: {}", e))?;
                Ok(json.into_bytes())
            }
            _ => Err("Unexpected response type".to_string()),
        }
    }

    /// Execute a ProEngine request
    pub fn execute(&self, req: ProEngineRequest) -> Result<ProEngineResponse, String> {
        self.executor
            .as_ref()
            .ok_or_else(|| "ProEngine executor not initialized".to_string())?
            .execute(req)
    }
}

/// Pro engine loading errors
#[derive(Debug)]
pub enum ProEngineError {
    NotFound,
    LoadError(String),
    InvalidFormat,
}

impl std::fmt::Display for ProEngineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProEngineError::NotFound => write!(f, "Pro engine not found"),
            ProEngineError::LoadError(msg) => write!(f, "Pro engine load error: {}", msg),
            ProEngineError::InvalidFormat => write!(f, "Invalid Pro engine format"),
        }
    }
}

impl std::error::Error for ProEngineError {}
