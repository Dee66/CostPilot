// Validation module for configuration files and schemas
//
// This module provides comprehensive validation for all CostPilot configuration files:
// - costpilot.yaml (main configuration)
// - policy files (YAML/JSON)
// - baselines.json (cost baselines)
// - slo.yaml (SLO definitions)
//
// Each validator provides:
// - Schema validation
// - Semantic validation
// - Helpful error messages with remediation hints
// - File path and line number tracking for errors

pub mod baselines;
pub mod config;
pub mod error;
#[cfg(not(target_arch = "wasm32"))]
pub mod output;
pub mod policy;
pub mod slo;

pub use baselines::BaselinesValidator;
pub use config::ConfigValidator;
pub use error::{ValidationError, ValidationResult, ValidationWarning};
#[cfg(not(target_arch = "wasm32"))]
pub use output::OutputValidator;
pub use policy::PolicyValidator;
pub use slo::SloValidator;

use serde::{Deserialize, Serialize};
use std::path::Path;

/// Validation severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    Error,   // Blocks execution
    Warning, // Suggests improvement
    Info,    // Informational only
}

/// Validation report containing all errors and warnings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationReport {
    pub file_path: String,
    pub file_type: FileType,
    pub is_valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
}

impl ValidationReport {
    pub fn new(file_path: impl AsRef<Path>, file_type: FileType) -> Self {
        Self {
            file_path: file_path.as_ref().to_string_lossy().to_string(),
            file_type,
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    pub fn add_error(&mut self, error: ValidationError) {
        self.is_valid = false;
        self.errors.push(error);
    }

    pub fn add_warning(&mut self, warning: ValidationWarning) {
        self.warnings.push(warning);
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn error_count(&self) -> usize {
        self.errors.len()
    }

    pub fn warning_count(&self) -> usize {
        self.warnings.len()
    }

    /// Format the report as human-readable text
    pub fn format_text(&self) -> String {
        use colored::Colorize;

        let mut output = String::new();

        // Header
        output.push_str(&format!(
            "📋 Validation Report: {}\n",
            self.file_path.bright_cyan()
        ));
        output.push_str(&format!("   Type: {:?}\n\n", self.file_type));

        // Status
        if self.is_valid {
            output.push_str(&format!("✅ {} Valid\n\n", "Status:".bold()));
        } else {
            output.push_str(&format!("❌ {} Invalid\n\n", "Status:".bold()));
        }

        // Errors
        if !self.errors.is_empty() {
            output.push_str(&format!(
                "🔴 {} Errors ({})\n",
                "Validation".bold(),
                self.errors.len()
            ));
            for error in &self.errors {
                output.push_str(&format!("\n{}\n", error.format()));
            }
            output.push('\n');
        }

        // Warnings
        if !self.warnings.is_empty() {
            output.push_str(&format!(
                "🟡 {} Warnings ({})\n",
                "Validation".bold(),
                self.warnings.len()
            ));
            for warning in &self.warnings {
                output.push_str(&format!("\n{}\n", warning.format()));
            }
            output.push('\n');
        }

        // Summary
        if self.is_valid && self.warnings.is_empty() {
            output.push_str("✨ No issues found!\n");
        } else if self.is_valid {
            output.push_str(&format!(
                "⚠️  File is valid but has {} warning(s). Consider addressing them.\n",
                self.warnings.len()
            ));
        } else {
            output.push_str(&format!(
                "❌ File has {} error(s). Fix these before using the configuration.\n",
                self.errors.len()
            ));
        }

        output
    }

    /// Format the report as JSON
    pub fn format_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_default()
    }
}

/// File types supported by validation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileType {
    Config,
    Policy,
    Baselines,
    Slo,
}

/// Validate any supported configuration file
pub fn validate_file(path: impl AsRef<Path>) -> ValidationResult<ValidationReport> {
    let path = path.as_ref();

    // Check if file exists first
    if !path.exists() {
        return Err(ValidationError::new(format!("No such file: {}", path.display())));
    }

    // Detect file type from name/extension
    let file_type = detect_file_type(path)?;

    match file_type {
        FileType::Config => ConfigValidator::validate_file(path),
        FileType::Policy => PolicyValidator::validate_file(path),
        FileType::Baselines => BaselinesValidator::validate_file(path),
        FileType::Slo => SloValidator::validate_file(path),
    }
}

/// Detect file type from path
fn detect_file_type(path: &Path) -> ValidationResult<FileType> {
    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| ValidationError::new("Invalid file path"))?;

    if file_name == "costpilot.yaml"
        || file_name == "costpilot.yml"
        || file_name == ".costpilot.yaml"
    {
        Ok(FileType::Config)
    } else if file_name == "baselines.json" {
        Ok(FileType::Baselines)
    } else if file_name.starts_with("slo")
        && (file_name.ends_with(".yaml") || file_name.ends_with(".yml"))
    {
        Ok(FileType::Slo)
    } else if file_name.ends_with(".yaml") || file_name.ends_with(".yml") {
        // Assume policy file
        Ok(FileType::Policy)
    } else {
        Err(ValidationError::new(format!(
            "Could not detect file type for: {}. Supported: costpilot.yaml, baselines.json, slo.yaml, *.yaml (policies)",
            file_name
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_detect_file_type() {
        assert_eq!(
            detect_file_type(&PathBuf::from("costpilot.yaml")).unwrap(),
            FileType::Config
        );
        assert_eq!(
            detect_file_type(&PathBuf::from(".costpilot.yaml")).unwrap(),
            FileType::Config
        );
        assert_eq!(
            detect_file_type(&PathBuf::from("baselines.json")).unwrap(),
            FileType::Baselines
        );
        assert_eq!(
            detect_file_type(&PathBuf::from("slo.yaml")).unwrap(),
            FileType::Slo
        );
        assert_eq!(
            detect_file_type(&PathBuf::from("my-policy.yaml")).unwrap(),
            FileType::Policy
        );
    }

    #[test]
    fn test_validation_report() {
        let mut report = ValidationReport::new("test.yaml", FileType::Config);
        assert!(report.is_valid);
        assert_eq!(report.error_count(), 0);

        report.add_error(ValidationError::new("Test error"));
        assert!(!report.is_valid);
        assert_eq!(report.error_count(), 1);
    }
}
