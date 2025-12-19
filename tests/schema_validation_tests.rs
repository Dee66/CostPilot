// JSON Schema validation tests for CostPilot outputs
// These tests validate that JSON schemas are well-formed and can be compiled

use jsonschema::{Draft, JSONSchema};
use serde_json::Value;
use std::fs;

#[cfg(test)]
mod compatibility;

#[test]
fn test_detection_schema_is_valid() {
    let schema_content = fs::read_to_string("tests/golden/schemas/detection_output.schema.json")
        .expect("Detection schema file not found");
    let schema_json: Value = serde_json::from_str(&schema_content)
        .expect("Detection schema must be valid JSON");

    // Test that the schema can be compiled
    let compiled = JSONSchema::options()
        .with_draft(Draft::Draft7)
        .compile(&schema_json);
    assert!(compiled.is_ok(), "Detection schema must compile successfully");
}

#[test]
fn test_prediction_schema_is_valid() {
    let schema_content = fs::read_to_string("tests/golden/schemas/prediction_output.schema.json")
        .expect("Prediction schema file not found");
    let schema_json: Value = serde_json::from_str(&schema_content)
        .expect("Prediction schema must be valid JSON");

    // Test that the schema can be compiled
    let compiled = JSONSchema::options()
        .with_draft(Draft::Draft7)
        .compile(&schema_json);
    assert!(compiled.is_ok(), "Prediction schema must compile successfully");
}

#[test]
fn test_mapping_schema_is_valid() {
    let schema_content = fs::read_to_string("tests/golden/schemas/mapping_output.schema.json")
        .expect("Mapping schema file not found");
    let schema_json: Value = serde_json::from_str(&schema_content)
        .expect("Mapping schema must be valid JSON");

    // Test that the schema can be compiled
    let compiled = JSONSchema::options()
        .with_draft(Draft::Draft7)
        .compile(&schema_json);
    assert!(compiled.is_ok(), "Mapping schema must compile successfully");
}

// Note: explain_output.schema.json and trend_output.schema.json do not exist yet
// These tests will be added when the schemas are created