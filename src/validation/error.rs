// Validation errors and result types

use std::fmt;
use serde::{Deserialize, Serialize};

/// Result type for validation operations
pub type ValidationResult<T> = Result<T, ValidationError>;

/// Validation error with context and remediation hints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    pub message: String,
    pub field: Option<String>,
    pub line: Option<usize>,
    pub column: Option<usize>,
    pub hint: Option<String>,
    pub error_code: Option<String>,
}

impl ValidationError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            field: None,
            line: None,
            column: None,
            hint: None,
            error_code: None,
        }
    }

    pub fn with_field(mut self, field: impl Into<String>) -> Self {
        self.field = Some(field.into());
        self
    }

    pub fn with_line(mut self, line: usize) -> Self {
        self.line = Some(line);
        self
    }

    pub fn with_column(mut self, column: usize) -> Self {
        self.column = Some(column);
        self
    }

    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }

    pub fn with_error_code(mut self, code: impl Into<String>) -> Self {
        self.error_code = Some(code.into());
        self
    }

    pub fn format(&self) -> String {
        use colored::Colorize;

        let mut output = String::new();

        // Error code and message
        if let Some(code) = &self.error_code {
            output.push_str(&format!("  {} {}: {}\n", "❌".red(), code.red().bold(), self.message));
        } else {
            output.push_str(&format!("  {} {}\n", "❌".red(), self.message));
        }

        // Location
        if let Some(field) = &self.field {
            output.push_str(&format!("     Field: {}\n", field.yellow()));
        }
        if let Some(line) = self.line {
            if let Some(column) = self.column {
                output.push_str(&format!("     Location: line {}, column {}\n", line, column));
            } else {
                output.push_str(&format!("     Location: line {}\n", line));
            }
        }

        // Hint
        if let Some(hint) = &self.hint {
            output.push_str(&format!("     💡 Hint: {}\n", hint.cyan()));
        }

        output
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ValidationError {}

impl From<std::io::Error> for ValidationError {
    fn from(err: std::io::Error) -> Self {
        ValidationError::new(format!("IO error: {}", err))
            .with_error_code("E500")
    }
}

impl From<serde_json::Error> for ValidationError {
    fn from(err: serde_json::Error) -> Self {
        ValidationError::new(format!("JSON parsing error: {}", err))
            .with_error_code("E501")
            .with_hint("Ensure the file is valid JSON format")
    }
}

impl From<serde_yaml::Error> for ValidationError {
    fn from(err: serde_yaml::Error) -> Self {
        let message = format!("YAML parsing error: {}", err);
        let mut error = ValidationError::new(message)
            .with_error_code("E502")
            .with_hint("Ensure the file is valid YAML format");

        // Try to extract location info
        if let Some(location) = err.location() {
            error = error.with_line(location.line()).with_column(location.column());
        }

        error
    }
}

/// Validation warning (non-blocking)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationWarning {
    pub message: String,
    pub field: Option<String>,
    pub suggestion: Option<String>,
    pub warning_code: Option<String>,
}

impl ValidationWarning {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            field: None,
            suggestion: None,
            warning_code: None,
        }
    }

    pub fn with_field(mut self, field: impl Into<String>) -> Self {
        self.field = Some(field.into());
        self
    }

    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }

    pub fn with_warning_code(mut self, code: impl Into<String>) -> Self {
        self.warning_code = Some(code.into());
        self
    }

    pub fn format(&self) -> String {
        use colored::Colorize;

        let mut output = String::new();

        // Warning code and message
        if let Some(code) = &self.warning_code {
            output.push_str(&format!("  {} {}: {}\n", "⚠️ ".yellow(), code.yellow().bold(), self.message));
        } else {
            output.push_str(&format!("  {} {}\n", "⚠️ ".yellow(), self.message));
        }

        // Field
        if let Some(field) = &self.field {
            output.push_str(&format!("     Field: {}\n", field.yellow()));
        }

        // Suggestion
        if let Some(suggestion) = &self.suggestion {
            output.push_str(&format!("     💡 Suggestion: {}\n", suggestion.cyan()));
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_error_builder() {
        let error = ValidationError::new("Missing field")
            .with_field("budget")
            .with_line(10)
            .with_column(5)
            .with_hint("Add a 'budget' field to the configuration")
            .with_error_code("E001");

        assert_eq!(error.message, "Missing field");
        assert_eq!(error.field, Some("budget".to_string()));
        assert_eq!(error.line, Some(10));
        assert_eq!(error.column, Some(5));
        assert_eq!(error.error_code, Some("E001".to_string()));
    }

    #[test]
    fn test_validation_warning_builder() {
        let warning = ValidationWarning::new("Deprecated field")
            .with_field("old_config")
            .with_suggestion("Use 'new_config' instead")
            .with_warning_code("W001");

        assert_eq!(warning.message, "Deprecated field");
        assert_eq!(warning.field, Some("old_config".to_string()));
        assert_eq!(warning.warning_code, Some("W001".to_string()));
    }
}
