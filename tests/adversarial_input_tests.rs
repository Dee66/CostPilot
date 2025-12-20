use costpilot::engines::detection::DetectionEngine;
use costpilot::engines::shared::error_model::ErrorCategory;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

/// Adversarial input testing - ensure CostPilot fails safely under hostile conditions
#[cfg(test)]
mod adversarial_input_tests {
    use super::*;

    fn create_temp_file(content: &str) -> Result<String, Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let file_path = temp_dir.path().join("test_plan.json");
        fs::write(&file_path, content)?;
        Ok(file_path.to_string_lossy().to_string())
    }

    #[test]
    fn test_extreme_json_nesting() {
        let engine = DetectionEngine::new();

        // Create deeply nested JSON that could cause stack overflow
        let mut nested_json = r#"{"planned_values": {"root_module": {"resources": ["#.to_string();
        for i in 0..1000 {
            nested_json.push_str(&format!(r#"{{"address": "test{}","values": {{"nested": "#, i));
        }
        nested_json.push_str(r#""deeply_nested_value""#);
        for _ in 0..1000 {
            nested_json.push_str("}}");
        }
        nested_json.push_str(r#"]}}}"#);

        let result = engine.detect_from_terraform_json(&nested_json);

        // Should not crash, should return an error
        assert!(result.is_err(), "Should handle extreme nesting gracefully");

        // Error should be structured and informative
        if let Err(e) = result {
            assert!(!e.message.is_empty(), "Error should have a descriptive message");
            // Could be parsing error or recursion limit error
            assert!(matches!(e.category,
                ErrorCategory::ParseError | ErrorCategory::ValidationError | ErrorCategory::InvalidInput),
                "Error should be in expected categories, got {:?}", e.category);
        }
    }

    #[test]
    fn test_extremely_large_json_string() {
        let engine = DetectionEngine::new();

        // Create a JSON with an extremely large string value
        let large_string = "x".repeat(10_000_000); // 10MB string
        let json = format!(r#"
        {{
            "planned_values": {{
                "root_module": {{
                    "resources": [
                        {{
                            "address": "test.large_string",
                            "values": {{
                                "large_field": "{}"
                            }}
                        }}
                    ]
                }}
            }}
        }}"#, large_string);

        // Should handle large strings without crashing
        let result = engine.detect_from_terraform_json(&json);

        // May succeed or fail, but should not crash
        match result {
            Ok(resources) => {
                // If it succeeds, should have processed the resource
                assert!(!resources.is_empty(), "Should have found at least one resource");
            }
            Err(e) => {
                // If it fails, should be a structured error
                assert!(!e.message.is_empty(), "Error should have a descriptive message");
                assert!(matches!(e.category,
                    ErrorCategory::ParseError | ErrorCategory::ValidationError | ErrorCategory::InvalidInput),
                    "Error should be in expected categories, got {:?}", e.category);
            }
        }
    }

    #[test]
    fn test_invalid_json() {
        let engine = DetectionEngine::new();

        let invalid_jsons = vec![
            "",  // Empty string
            "{", // Unclosed brace
            r#"{"planned_values": "missing_brace""#, // Missing closing brace
            r#"{"planned_values": {"root_module": {"resources": [{}]}}"#, // Missing comma
            r#"{"planned_values": null, "extra": "data""#, // Invalid structure
            "not json at all", // Not JSON
            r#"{"planned_values": {"root_module": {"resources": [{"address": "test", "values": null}]}}}"#, // Extra closing brace
        ];

        for (i, invalid_json) in invalid_jsons.iter().enumerate() {
            let result = engine.detect_from_terraform_json(invalid_json);
            assert!(result.is_err(), "Invalid JSON {} should fail: {}", i, invalid_json);

            if let Err(e) = result {
                assert!(!e.message.is_empty(), "Error should have descriptive message for case {}", i);
                assert_eq!(e.category, ErrorCategory::ParseError,
                    "Invalid JSON should result in ParseError, got {:?} for case {}", e.category, i);
            }
        }
    }

    #[test]
    fn test_malformed_terraform_plan() {
        let engine = DetectionEngine::new();

        let malformed_plans = vec![
            // Missing planned_values
            r#"{"configuration": {"root_module": {}}}"#,
            // Empty resources array
            r#"{"planned_values": {"root_module": {"resources": []}}}"#,
            // Resource without address
            r#"{"planned_values": {"root_module": {"resources": [{"values": {}}]}}}"#,
            // Resource with invalid address
            r#"{"planned_values": {"root_module": {"resources": [{"address": "", "values": {}}]}}}"#,
            // Nested resources (should be flat array)
            r#"{"planned_values": {"root_module": {"resources": [{"address": "test", "values": {}, "resources": []}]}}}"#,
        ];

        for (i, malformed_plan) in malformed_plans.iter().enumerate() {
            let result = engine.detect_from_terraform_json(malformed_plan);

            // May succeed or fail depending on how permissive the parser is
            // But should never crash
            match result {
                Ok(resources) => {
                    // If it succeeds, resources should be valid
                    for resource in &resources {
                        assert!(!resource.resource_id.is_empty(), "Resource ID should not be empty in case {}", i);
                        assert!(!resource.resource_type.is_empty(), "Resource type should not be empty in case {}", i);
                    }
                }
                Err(e) => {
                    // If it fails, should be structured error
                    assert!(!e.message.is_empty(), "Error should have message for case {}", i);
                    assert!(matches!(e.category,
                        ErrorCategory::ParseError | ErrorCategory::ValidationError),
                        "Error should be parse/validation error, got {:?} for case {}", e.category, i);
                }
            }
        }
    }

    #[test]
    fn test_corrupted_binary_data() {
        let engine = DetectionEngine::new();

        // Test with binary data that could be interpreted as JSON
        let binary_data = vec![
            vec![0x00, 0x01, 0x02, 0x03], // Null bytes
            vec![0xFF, 0xFE, 0xFD, 0xFC], // High bytes
            (0..1000).map(|i| (i % 256) as u8).collect(), // Pattern
            vec![0x7B, 0x22, 0xFF, 0x22, 0x7D], // Invalid UTF-8 in JSON-like structure
        ];

        for (i, data) in binary_data.iter().enumerate() {
            let json_str = String::from_utf8_lossy(data);
            let result = engine.detect_from_terraform_json(&json_str);

            // Should fail gracefully, not crash
            assert!(result.is_err(), "Corrupted data {} should fail", i);

            if let Err(e) = result {
                assert!(!e.message.is_empty(), "Error should have message for corrupted data {}", i);
                assert_eq!(e.category, ErrorCategory::ParseError,
                    "Corrupted data should result in ParseError, got {:?} for case {}", e.category, i);
            }
        }
    }

    #[test]
    fn test_extreme_array_sizes() {
        let engine = DetectionEngine::new();

        // Test with extremely large arrays
        let mut large_array = r#"{"planned_values": {"root_module": {"resources": ["#.to_string();
        for i in 0..10000 {
            if i > 0 { large_array.push(','); }
            large_array.push_str(&format!(r#"{{"address": "test{}","values": {{"instance_type": "t2.micro"}}}}"#, i));
        }
        large_array.push_str(r#"]}}}"#);

        let result = engine.detect_from_terraform_json(&large_array);

        // Should handle large arrays without crashing
        match result {
            Ok(resources) => {
                assert_eq!(resources.len(), 10000, "Should process all 10000 resources");
                for (i, resource) in resources.iter().enumerate() {
                    assert_eq!(resource.resource_id, format!("test{}", i), "Resource {} should have correct ID", i);
                }
            }
            Err(e) => {
                // If it fails due to memory limits, that's acceptable
                assert!(!e.message.is_empty(), "Error should have message");
                assert!(matches!(e.category,
                    ErrorCategory::ParseError | ErrorCategory::InvalidInput | ErrorCategory::ValidationError),
                    "Error should be in expected categories, got {:?}", e.category);
            }
        }
    }

    #[test]
    fn test_path_traversal_attempts() {
        let engine = DetectionEngine::new();

        // Test file path with path traversal attempts
        let traversal_paths = vec![
            "../../../etc/passwd",
            "..\\..\\..\\windows\\system32",
            "/etc/shadow",
            "C:\\Windows\\System32\\config\\SAM",
            "../../../../root/.ssh/id_rsa",
        ];

        for path in traversal_paths {
            // The engine should not attempt to read these paths
            // (though in this test we're only testing JSON parsing, not file reading)
            let result = engine.detect_from_terraform_plan(Path::new(path));

            // Should fail with file system error, not security issue
            assert!(result.is_err(), "Path traversal attempt should fail: {}", path);

            if let Err(e) = result {
                assert_eq!(e.category, ErrorCategory::FileSystemError,
                    "Path traversal should result in FileSystemError, got {:?} for path {}", e.category, path);
                assert!(!e.message.is_empty(), "Error should have message for path {}", path);
            }
        }
    }

    #[test]
    fn test_deterministic_error_responses() {
        let engine = DetectionEngine::new();

        let invalid_json = r#"{"planned_values": {"root_module": {"resources": [{"address": "test", "values": }]}}}"#;

        // Run the same invalid input multiple times
        let results: Vec<_> = (0..5)
            .map(|_| engine.detect_from_terraform_json(invalid_json))
            .collect();

        // All should fail with the same error
        for result in &results {
            assert!(result.is_err(), "Should consistently fail");
        }

        // Extract error messages
        let error_messages: Vec<_> = results.iter()
            .map(|r| r.as_ref().unwrap_err().message.clone())
            .collect();

        // All error messages should be identical
        let first_message = &error_messages[0];
        for (i, message) in error_messages.iter().enumerate() {
            assert_eq!(message, first_message,
                "Error message should be deterministic, got different message at iteration {}: {} vs {}", i, message, first_message);
        }
    }

    #[test]
    fn test_memory_behavior_large_inputs() {
        let engine = DetectionEngine::new();

        // Test with progressively larger inputs
        let sizes = vec![1000, 10000, 100000];

        for size in sizes {
            let large_json = format!(r#"
            {{
                "planned_values": {{
                    "root_module": {{
                        "resources": [
                            {{
                                "address": "test.large",
                                "values": {{
                                    "large_field": "{}"
                                }}
                            }}
                        ]
                    }}
                }}
            }}"#, "x".repeat(size));

            let result = engine.detect_from_terraform_json(&large_json);

            // Should not crash regardless of size
            match result {
                Ok(resources) => {
                    assert_eq!(resources.len(), 1, "Should process one resource for size {}", size);
                }
                Err(e) => {
                    // Memory or parsing errors are acceptable
                    assert!(matches!(e.category,
                        ErrorCategory::ParseError | ErrorCategory::InvalidInput | ErrorCategory::ValidationError),
                        "Error should be acceptable type for size {}, got {:?}", size, e.category);
                }
            }
        }
    }
}
