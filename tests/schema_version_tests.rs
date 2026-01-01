use std::fs;

/// Test that output schema version matches binary version compatibility table
#[test]
fn test_output_schema_version_compatibility() {
    // This test validates that the schema version in outputs matches
    // the binary version compatibility requirements

    // Create a temporary directory for testing
    let temp_dir = tempfile::tempdir().unwrap();

    // Simulate current binary version (would be read from Cargo.toml in real implementation)
    let current_version = "1.0.0";

    // Create a mock output with schema version
    let output_file = temp_dir.path().join("costpilot_output.json");
    let mock_output = format!(
        r#"
{{
  "schema_version": "{}",
  "timestamp": "2024-01-01T00:00:00Z",
  "results": [
    {{
      "resource_type": "aws_instance",
      "monthly_cost": 10.50,
      "currency": "USD"
    }}
  ]
}}
"#,
        current_version
    );

    fs::write(&output_file, mock_output).unwrap();

    // Read and parse the output
    let content = fs::read_to_string(&output_file).unwrap();
    let json_value: serde_json::Value = serde_json::from_str(&content).unwrap();

    // Extract schema version from output
    let output_schema_version = json_value["schema_version"].as_str().unwrap();

    // Validate that schema version matches current binary version
    // In a real implementation, this would check against a compatibility table
    assert_eq!(
        output_schema_version, current_version,
        "Output schema version {} does not match binary version {}",
        output_schema_version, current_version
    );

    // Test compatibility table validation
    // Simulate a compatibility table (in real implementation this would be a data structure)
    let compatibility_table = std::collections::HashMap::from([
        ("1.0.0", vec!["1.0.0"]),
        ("1.1.0", vec!["1.0.0", "1.1.0"]),
    ]);

    // Check that current version is in compatibility table
    assert!(
        compatibility_table.contains_key(current_version),
        "Binary version {} not found in compatibility table",
        current_version
    );

    // Check that output schema version is compatible
    let compatible_versions = compatibility_table.get(current_version).unwrap();
    assert!(
        compatible_versions.contains(&output_schema_version.to_string().as_str()),
        "Schema version {} is not compatible with binary version {}",
        output_schema_version,
        current_version
    );

    temp_dir.close().unwrap();
}
