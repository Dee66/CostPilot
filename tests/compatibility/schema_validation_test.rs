// Canonical output schema validation tests

use serde_json::Value;
use std::fs;
use std::path::PathBuf;

#[test]
fn test_prediction_schema_exists() {
    let schema_path = PathBuf::from("tests/golden/schemas/prediction_output.schema.json");
    assert!(schema_path.exists(), "Prediction output schema must exist");
    
    // Verify it's valid JSON
    let content = fs::read_to_string(&schema_path).expect("Failed to read schema");
    let _schema: Value = serde_json::from_str(&content)
        .expect("Schema must be valid JSON");
}

#[test]
fn test_detection_schema_exists() {
    let schema_path = PathBuf::from("tests/golden/schemas/detection_output.schema.json");
    assert!(schema_path.exists(), "Detection output schema must exist");
    
    let content = fs::read_to_string(&schema_path).expect("Failed to read schema");
    let _schema: Value = serde_json::from_str(&content)
        .expect("Schema must be valid JSON");
}

#[test]
fn test_mapping_schema_exists() {
    let schema_path = PathBuf::from("tests/golden/schemas/mapping_output.schema.json");
    assert!(schema_path.exists(), "Mapping output schema must exist");
    
    let content = fs::read_to_string(&schema_path).expect("Failed to read schema");
    let _schema: Value = serde_json::from_str(&content)
        .expect("Schema must be valid JSON");
}

#[test]
fn test_golden_prediction_matches_schema() {
    // Test that golden prediction output matches the schema structure
    let golden_path = PathBuf::from("test/golden/predict_output.json");
    if !golden_path.exists() {
        eprintln!("⚠️  Warning: Golden prediction output not found, skipping validation");
        return;
    }
    
    let content = fs::read_to_string(&golden_path).expect("Failed to read golden output");
    let output: Value = serde_json::from_str(&content)
        .expect("Golden output must be valid JSON");
    
    // Verify required top-level fields
    assert!(output.get("estimates").is_some(), "Output must have 'estimates' field");
    assert!(output.get("total").is_some(), "Output must have 'total' field");
    assert!(output.get("metadata").is_some(), "Output must have 'metadata' field");
    
    // Verify metadata structure
    let metadata = output.get("metadata").expect("metadata must exist");
    assert!(metadata.get("version").is_some(), "metadata must have version");
    assert!(metadata.get("timestamp").is_some(), "metadata must have timestamp");
    assert!(metadata.get("resource_count").is_some(), "metadata must have resource_count");
}

#[test]
fn test_golden_mapping_matches_schema() {
    // Test that golden mapping output matches the schema structure
    let golden_path = PathBuf::from("test/golden/mapping_graph.json");
    if !golden_path.exists() {
        eprintln!("⚠️  Warning: Golden mapping output not found, skipping validation");
        return;
    }
    
    let content = fs::read_to_string(&golden_path).expect("Failed to read golden output");
    let output: Value = serde_json::from_str(&content)
        .expect("Golden output must be valid JSON");
    
    // Verify required top-level fields
    assert!(output.get("nodes").is_some(), "Output must have 'nodes' field");
    assert!(output.get("edges").is_some(), "Output must have 'edges' field");
    assert!(output.get("metadata").is_some(), "Output must have 'metadata' field");
}

#[test]
fn test_schema_defines_required_fields() {
    let schema_path = PathBuf::from("tests/golden/schemas/prediction_output.schema.json");
    let content = fs::read_to_string(&schema_path).expect("Failed to read schema");
    let schema: Value = serde_json::from_str(&content).expect("Schema must be valid JSON");
    
    // Verify schema has required field list
    let required = schema.get("required").expect("Schema must have 'required' field");
    assert!(required.is_array(), "'required' must be an array");
    
    let required_array = required.as_array().unwrap();
    assert!(!required_array.is_empty(), "'required' array must not be empty");
    
    // Verify key required fields are present
    let required_strs: Vec<&str> = required_array
        .iter()
        .filter_map(|v| v.as_str())
        .collect();
    
    assert!(required_strs.contains(&"estimates"), "Schema must require 'estimates'");
    assert!(required_strs.contains(&"total"), "Schema must require 'total'");
    assert!(required_strs.contains(&"metadata"), "Schema must require 'metadata'");
}

#[test]
fn test_all_schemas_have_definitions() {
    let schema_files = vec![
        "tests/golden/schemas/prediction_output.schema.json",
        "tests/golden/schemas/detection_output.schema.json",
        "tests/golden/schemas/mapping_output.schema.json",
    ];
    
    for schema_file in schema_files {
        let schema_path = PathBuf::from(schema_file);
        let content = fs::read_to_string(&schema_path)
            .expect(&format!("Failed to read schema: {}", schema_file));
        let schema: Value = serde_json::from_str(&content)
            .expect(&format!("Schema must be valid JSON: {}", schema_file));
        
        // Verify schema has definitions section
        assert!(schema.get("definitions").is_some(), 
            "Schema {} must have 'definitions' section", schema_file);
        
        let definitions = schema.get("definitions").unwrap();
        assert!(definitions.is_object(), 
            "Schema {} 'definitions' must be an object", schema_file);
        assert!(!definitions.as_object().unwrap().is_empty(), 
            "Schema {} 'definitions' must not be empty", schema_file);
    }
}

#[test]
fn test_schemas_enforce_determinism() {
    // Test that schemas enforce deterministic properties
    let schema_path = PathBuf::from("tests/golden/schemas/prediction_output.schema.json");
    let content = fs::read_to_string(&schema_path).expect("Failed to read schema");
    let schema: Value = serde_json::from_str(&content).expect("Schema must be valid JSON");
    
    // Check CostEstimate definition has confidence bounds
    let definitions = schema.get("definitions").expect("Must have definitions");
    let cost_estimate = definitions.get("CostEstimate").expect("Must define CostEstimate");
    let properties = cost_estimate.get("properties").expect("CostEstimate must have properties");
    
    // Verify confidence_score has min/max constraints
    let confidence = properties.get("confidence_score").expect("Must have confidence_score");
    assert!(confidence.get("minimum").is_some(), "confidence_score must have minimum");
    assert!(confidence.get("maximum").is_some(), "confidence_score must have maximum");
    
    let min = confidence.get("minimum").and_then(|v| v.as_f64()).unwrap();
    let max = confidence.get("maximum").and_then(|v| v.as_f64()).unwrap();
    assert_eq!(min, 0.0, "confidence_score minimum must be 0.0");
    assert_eq!(max, 1.0, "confidence_score maximum must be 1.0");
}

#[test]
fn test_schemas_validate_cost_non_negative() {
    // Test that cost fields require non-negative values
    let schema_path = PathBuf::from("tests/golden/schemas/prediction_output.schema.json");
    let content = fs::read_to_string(&schema_path).expect("Failed to read schema");
    let schema: Value = serde_json::from_str(&content).expect("Schema must be valid JSON");
    
    let definitions = schema.get("definitions").expect("Must have definitions");
    let cost_estimate = definitions.get("CostEstimate").expect("Must define CostEstimate");
    let properties = cost_estimate.get("properties").expect("CostEstimate must have properties");
    
    // Verify all cost fields have minimum: 0
    let cost_fields = vec!["monthly_cost", "prediction_interval_low", "prediction_interval_high"];
    
    for field in cost_fields {
        let field_def = properties.get(field).expect(&format!("Must have {}", field));
        let minimum = field_def.get("minimum")
            .and_then(|v| v.as_f64())
            .expect(&format!("{} must have minimum constraint", field));
        assert_eq!(minimum, 0.0, "{} must have minimum of 0.0", field);
    }
}
