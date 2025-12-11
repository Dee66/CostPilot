// Error model with stable error IDs and categorization

use serde::{Deserialize, Serialize};
use std::fmt;

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
    /// Performance budget timeout
    Timeout,
    /// Circuit breaker triggered
    CircuitBreaker,
    /// Validation error
    ValidationError,
    /// IO error
    IoError,
    /// Resource not found
    NotFound,
    /// Security violation
    SecurityViolation,
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

    /// Create a validation error
    pub fn validation_error(message: impl Into<String>) -> Self {
        Self::new("E_VALIDATION", ErrorCategory::ValidationError, message)
    }

    /// Create an IO error
    pub fn io_error(message: impl Into<String>) -> Self {
        Self::new("E_IO", ErrorCategory::IoError, message)
    }

    /// Create a serialization error
    pub fn serialization_error(message: impl Into<String>) -> Self {
        Self::new("E_SERIALIZATION", ErrorCategory::InternalError, message)
    }

    /// Create a parse error
    pub fn parse_error(message: impl Into<String>) -> Self {
        Self::new("E_PARSE", ErrorCategory::ParseError, message)
    }

    /// Create a file not found error
    pub fn file_not_found(message: impl Into<String>) -> Self {
        Self::new("E_FILE_NOT_FOUND", ErrorCategory::NotFound, message)
    }

    /// Create a generation error
    pub fn generation_error(message: impl Into<String>) -> Self {
        Self::new("E_GENERATION", ErrorCategory::InternalError, message)
    }

    /// Create an invalid JSON error
    pub fn invalid_json(message: impl Into<String>) -> Self {
        Self::new("E_INVALID_JSON", ErrorCategory::InvalidInput, message)
            .with_hint("Check JSON syntax and structure")
    }

    /// Create an upgrade required error
    pub fn upgrade_required(message: impl Into<String>) -> Self {
        Self::new(
            "E_UPGRADE_REQUIRED",
            ErrorCategory::ValidationError,
            message,
        )
        .with_hint("This feature requires CostPilot Premium. Visit https://costpilot.dev/upgrade")
    }

    /// Create a policy violation error
    pub fn policy_violation(policy_id: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(
            format!("E_POLICY_{}", policy_id.into()),
            ErrorCategory::PolicyViolation,
            message,
        )
        .with_hint("Review policy configuration and resource settings")
    }

    /// Create an SLO breach error
    pub fn slo_breach(slo_name: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(
            format!("E_SLO_{}", slo_name.into().to_uppercase().replace(' ', "_")),
            ErrorCategory::SLOBreach,
            message,
        )
        .with_hint("Review SLO thresholds and current metrics")
    }

    /// Create a prediction error
    pub fn prediction_error(message: impl Into<String>) -> Self {
        Self::new("E_PREDICTION", ErrorCategory::PredictionError, message)
            .with_hint("Check resource configuration and heuristics")
    }

    /// Create a configuration error
    pub fn config_error(message: impl Into<String>) -> Self {
        Self::new("E_CONFIG", ErrorCategory::ConfigError, message)
            .with_hint("Verify configuration file syntax and required fields")
    }

    /// Create a security violation error
    pub fn security_violation(message: impl Into<String>) -> Self {
        Self::new("E_SECURITY", ErrorCategory::SecurityViolation, message)
            .with_hint("Review security policies and access controls")
    }

    /// Generate remediation hint based on error category and message
    pub fn generate_hint(&self) -> String {
        if let Some(hint) = &self.hint {
            return hint.clone();
        }

        match self.category {
            ErrorCategory::InvalidInput => {
                "Check input format and ensure it's valid JSON/YAML".to_string()
            }
            ErrorCategory::ParseError => {
                "Verify the file is a valid Terraform/CDK/CloudFormation document".to_string()
            }
            ErrorCategory::PredictionError => {
                "Check resource configuration and ensure heuristics are up to date".to_string()
            }
            ErrorCategory::PolicyViolation => {
                "Review policy rules and adjust resource configuration".to_string()
            }
            ErrorCategory::SLOBreach => {
                "Check SLO thresholds and current metrics, consider adjusting targets".to_string()
            }
            ErrorCategory::DriftDetected => {
                "Review drift and use autofix to reconcile state".to_string()
            }
            ErrorCategory::InternalError => {
                "This is an internal error, please report it with context".to_string()
            }
            ErrorCategory::ConfigError => {
                "Check configuration file syntax and required fields".to_string()
            }
            ErrorCategory::FileSystemError => "Verify file permissions and disk space".to_string(),
            ErrorCategory::Timeout => {
                "Operation exceeded time budget, try reducing input size or complexity".to_string()
            }
            ErrorCategory::CircuitBreaker => {
                "Service is unavailable due to repeated failures, retry later".to_string()
            }
            ErrorCategory::ValidationError => {
                "Input validation failed, check data types and constraints".to_string()
            }
            ErrorCategory::IoError => "Check file permissions, paths, and disk space".to_string(),
            ErrorCategory::NotFound => "Verify the resource or file path exists".to_string(),
            ErrorCategory::SecurityViolation => {
                "Review security policies and access controls".to_string()
            }
        }
    }

    /// Convert to machine-readable format (JSON)
    pub fn to_machine_format(&self) -> String {
        serde_json::to_string(self)
            .unwrap_or_else(|_| format!(r#"{{"id":"{}","message":"{}"}}"#, self.id, self.message))
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

/// Map error category to stable error ID prefix
pub fn map_category_to_id(category: &ErrorCategory) -> &'static str {
    match category {
        ErrorCategory::InvalidInput => "E_INVALID_INPUT",
        ErrorCategory::ParseError => "E_PARSE",
        ErrorCategory::PredictionError => "E_PREDICTION",
        ErrorCategory::PolicyViolation => "E_POLICY",
        ErrorCategory::SLOBreach => "E_SLO",
        ErrorCategory::DriftDetected => "E_DRIFT",
        ErrorCategory::InternalError => "E_INTERNAL",
        ErrorCategory::ConfigError => "E_CONFIG",
        ErrorCategory::FileSystemError => "E_FS",
        ErrorCategory::Timeout => "E_TIMEOUT",
        ErrorCategory::CircuitBreaker => "E_CIRCUIT_BREAKER",
        ErrorCategory::ValidationError => "E_VALIDATION",
        ErrorCategory::IoError => "E_IO",
        ErrorCategory::NotFound => "E_NOT_FOUND",
        ErrorCategory::SecurityViolation => "E_SECURITY",
    }
}
