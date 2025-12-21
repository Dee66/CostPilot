#![cfg(not(target_arch = "wasm32"))]

// Output validation module for CostPilot JSON outputs
//
// This module validates that all CostPilot outputs conform to their canonical schemas:
// - detection output
// - prediction output
// - mapping output
// - explain output (future)
// - trend output (future)
//
// Validation is strict: fails hard on missing required fields, unknown fields, wrong types.

use crate::engines::shared::error_model::{CostPilotError, ErrorCategory, Result};
use jsonschema::{Draft, JSONSchema};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;

/// Output types that can be validated
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OutputType {
    Detection,
    Prediction,
    Mapping,
    Explain,
    Trend,
}

/// Output validator for JSON schema validation
pub struct OutputValidator {
    schemas: HashMap<OutputType, JSONSchema>,
}

impl OutputValidator {
    /// Create a new output validator with all schemas loaded
    pub fn new() -> Result<Self> {
        let mut schemas = HashMap::new();

        // Load detection schema
        let detection_schema = Self::load_schema("tests/golden/schemas/detection_output.schema.json")?;
        schemas.insert(OutputType::Detection, detection_schema);

        // Load prediction schema
        let prediction_schema = Self::load_schema("tests/golden/schemas/prediction_output.schema.json")?;
        schemas.insert(OutputType::Prediction, prediction_schema);

        // Load mapping schema
        let mapping_schema = Self::load_schema("tests/golden/schemas/mapping_output.schema.json")?;
        schemas.insert(OutputType::Mapping, mapping_schema);

        // Note: explain and trend schemas don't exist yet
        // schemas.insert(OutputType::Explain, Self::load_schema("tests/golden/schemas/explain_output.schema.json")?);
        // schemas.insert(OutputType::Trend, Self::load_schema("tests/golden/schemas/trend_output.schema.json")?);

        Ok(Self { schemas })
    }

    /// Validate JSON output against its schema
    pub fn validate(&self, output_type: OutputType, json_str: &str) -> Result<()> {
        // Parse JSON first
        let json_value: Value = serde_json::from_str(json_str).map_err(|e| {
            CostPilotError::new(
                "SCHEMA_001",
                ErrorCategory::ValidationError,
                format!("Output is not valid JSON: {}", e),
            )
        })?;

        // Get the schema
        let schema = self.schemas.get(&output_type).ok_or_else(|| {
            CostPilotError::new(
                "SCHEMA_002",
                ErrorCategory::ValidationError,
                format!("No schema available for output type {:?}", output_type),
            )
        })?;

        // Validate against schema
        let result = schema.validate(&json_value);
        if let Err(errors) = result {
            let error_messages: Vec<String> = errors.map(|e| e.to_string()).collect();
            return Err(CostPilotError::new(
                "SCHEMA_003",
                ErrorCategory::ValidationError,
                format!(
                    "Output does not conform to schema for {:?}: {}",
                    output_type,
                    error_messages.join("; ")
                ),
            ));
        }

        Ok(())
    }

    /// Load and compile a JSON schema from file
    fn load_schema(path: &str) -> Result<JSONSchema> {
        let schema_content = fs::read_to_string(path).map_err(|e| {
            CostPilotError::new(
                "SCHEMA_004",
                ErrorCategory::FileSystemError,
                format!("Failed to read schema file {}: {}", path, e),
            )
        })?;

        let schema_value: Value = serde_json::from_str(&schema_content).map_err(|e| {
            CostPilotError::new(
                "SCHEMA_005",
                ErrorCategory::ValidationError,
                format!("Schema file {} is not valid JSON: {}", path, e),
            )
        })?;

        let compiled = JSONSchema::options()
            .with_draft(Draft::Draft7)
            .compile(&schema_value)
            .map_err(|e| {
                CostPilotError::new(
                    "SCHEMA_006",
                    ErrorCategory::ValidationError,
                    format!("Schema file {} cannot be compiled: {}", path, e),
                )
            })?;

        Ok(compiled)
    }
}

impl Default for OutputValidator {
    fn default() -> Self {
        Self::new().expect("Failed to create output validator")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_validator_creation() {
        let validator = OutputValidator::new();
        assert!(validator.is_ok());
    }

    #[test]
    fn test_valid_detection_output() {
        let validator = OutputValidator::new().unwrap();

        // This is a minimal valid detection output
        let valid_output = r#"{
            "detections": [],
            "summary": {
                "total_count": 0,
                "by_severity": {}
            },
            "metadata": {
                "version": "1.0.0",
                "timestamp": "2023-01-01T00:00:00Z"
            }
        }"#;

        let result = validator.validate(OutputType::Detection, valid_output);
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_detection_output_missing_required_field() {
        let validator = OutputValidator::new().unwrap();

        // Missing required "metadata" field
        let invalid_output = r#"{
            "detections": [],
            "summary": {
                "total_count": 0,
                "by_severity": {}
            }
        }"#;

        let result = validator.validate(OutputType::Detection, invalid_output);
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.category, ErrorCategory::ValidationError);
        assert!(error.message.contains("does not conform to schema"));
    }

    #[test]
    fn test_invalid_detection_output_wrong_type() {
        let validator = OutputValidator::new().unwrap();

        // "total_count" should be integer, not string
        let invalid_output = r#"{
            "detections": [],
            "summary": {
                "total_count": "zero",
                "by_severity": {}
            },
            "metadata": {
                "version": "1.0.0",
                "timestamp": "2023-01-01T00:00:00Z"
            }
        }"#;

        let result = validator.validate(OutputType::Detection, invalid_output);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_json() {
        let validator = OutputValidator::new().unwrap();

        let invalid_json = r#"{"detections": [}"#; // Invalid JSON

        let result = validator.validate(OutputType::Detection, invalid_json);
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("not valid JSON"));
    }
}
