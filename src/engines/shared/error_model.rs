// Error model with stable error IDs and categorization

use std::fmt;
use serde::{Serialize, Deserialize};

/// Error categories for CostPilot
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ErrorCategory {
    /// Invalid input format (e.g., malformed JSON)
    InvalidInput,
    /// Parser failed to process IaC
    ParseError,
    /// Prediction engine error
    PredictionError,
    /// Policy violation
    PolicyViolation,
    /// SLO breach
    SLOBreach,
    /// Drift detected
    DriftDetected,
    /// Internal engine error
    InternalError,
    /// Configuration error
    ConfigError,
    /// File system error
    FileSystemError,
}

/// Stable error with ID and remediation hints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostPilotError {
    /// Stable error identifier
    pub id: String,
    /// Error category
    pub category: ErrorCategory,
    /// Human-readable message
    pub message: String,
    /// Remediation hint
    pub hint: Option<String>,
    /// Context data
    pub context: Option<serde_json::Value>,
}

impl CostPilotError {
    pub fn new(id: impl Into<String>, category: ErrorCategory, message: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            category,
            message: message.into(),
            hint: None,
            context: None,
        }
    }

    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }

    pub fn with_context(mut self, context: serde_json::Value) -> Self {
        self.context = Some(context);
        self
    }
}

impl fmt::Display for CostPilotError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.id, self.message)?;
        if let Some(hint) = &self.hint {
            write!(f, "\n  Hint: {}", hint)?;
        }
        Ok(())
    }
}

impl std::error::Error for CostPilotError {}

/// Type alias for Results
pub type Result<T> = std::result::Result<T, CostPilotError>;
