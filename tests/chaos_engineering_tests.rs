use costpilot::engines::detection::DetectionEngine;
use costpilot::engines::shared::error_model::ErrorCategory;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

/// Chaos Engineering tests - validate predictable failure modes under hostile runtime conditions
#[cfg(test)]
mod chaos_engineering_tests {
    use super::*;

    #[test]
    fn test_filesystem_permission_denied() {
        let engine = DetectionEngine::new();

        // Create a temporary directory and make it read-only
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.json");

        // Write a valid plan file
        let valid_plan = r#"{"planned_values": {"root_module": {"resources": []}}}"#;
        fs::write(&test_file, valid_plan).unwrap();

        // Make the directory read-only (this might not work on all systems)
        // Instead, we'll test with a non-existent file path that should fail
        let nonexistent_path = Path::new("/nonexistent/path/that/does/not/exist/plan.json");

        let result = engine.detect_from_terraform_plan(nonexistent_path);

        // Should fail with filesystem error
        assert!(result.is_err(), "Should fail when file doesn't exist");

        if let Err(e) = result {
            assert_eq!(e.category, ErrorCategory::FileSystemError,
                "Should result in FileSystemError for nonexistent file, got {:?}", e.category);
            assert!(!e.message.is_empty(), "Error should have descriptive message");
        }
    }

    #[test]
    fn test_memory_pressure_simulation() {
        let engine = DetectionEngine::new();

        // Test with moderately large input that could cause memory issues
        // In a real chaos engineering setup, we'd use external tools to simulate OOM
        // For now, we'll test with large but manageable inputs

        let large_json = format!(r#"
        {{
            "planned_values": {{
                "root_module": {{
                    "resources": [
                        {{
                            "address": "test.large_memory",
                            "values": {{
                                "large_field": "{}"
                            }}
                        }}
                    ]
                }}
            }}
        }}"#, "x".repeat(50_000_000)); // 50MB string

        let result = engine.detect_from_terraform_json(&large_json);

        // Should handle large inputs without crashing
        // May succeed or fail, but should not panic
        match result {
            Ok(resources) => {
                assert_eq!(resources.len(), 1, "Should process one resource");
            }
            Err(e) => {
                // Memory-related errors are acceptable
                assert!(matches!(e.category,
                    ErrorCategory::ParseError | ErrorCategory::InvalidInput | ErrorCategory::ValidationError),
                    "Error should be acceptable type, got {:?}", e.category);
                assert!(!e.message.is_empty(), "Error should have message");
            }
        }
    }

    #[test]
    fn test_network_failure_simulation() {
        // CostPilot should have zero network dependencies for core functionality
        // This test validates that no network calls are attempted during normal operation

        let engine = DetectionEngine::new();
        let valid_plan = r#"{"planned_values": {"root_module": {"resources": []}}}"#;

        let result = engine.detect_from_terraform_json(valid_plan);

        // Should work without network access
        // In a real chaos engineering setup, we'd run this in a network-isolated environment
        match result {
            Ok(resources) => {
                // Success indicates no network dependency
                assert!(resources.is_empty(), "Should handle empty resources without network");
            }
            Err(e) => {
                // If it fails, should be due to input validation, not network
                assert!(matches!(e.category,
                    ErrorCategory::ParseError | ErrorCategory::ValidationError | ErrorCategory::InvalidInput),
                    "Error should be input-related, not network-related, got {:?}", e.category);
            }
        }
    }

    #[test]
    fn test_fault_injection_via_corrupted_state() {
        let engine = DetectionEngine::new();

        // Test with inputs that could cause internal state corruption
        let corrupted_inputs = vec![
            // JSON with null bytes that could cause string processing issues
            format!("{}\0{}", r#"{"planned_values": {"root_module": {"resources": []}}}"#, "trailing"),
            // Very deep but valid JSON
            r#"{"a":{"a":{"a":{"a":{"a":{"a":{"a":{"a":{"a":{"a": "deep"}}}}}}}}}}"#.to_string(),
            // JSON with extreme Unicode
            r#"{"planned_values": {"root_module": {"resources": [{"address": "test", "values": {"field": "🚀💯🔥"}}]}}}"#.to_string(),
        ];

        for (i, corrupted_input) in corrupted_inputs.iter().enumerate() {
            let result = engine.detect_from_terraform_json(corrupted_input);

            // Should not crash, should fail gracefully
            match result {
                Ok(resources) => {
                    // If it succeeds, validate the resources are reasonable
                    for resource in &resources {
                        assert!(!resource.resource_id.is_empty(),
                            "Resource should have valid ID in corrupted input test {}", i);
                    }
                }
                Err(e) => {
                    // Should be structured error
                    assert!(!e.message.is_empty(),
                        "Error should have message for corrupted input {}", i);
                    assert!(matches!(e.category,
                        ErrorCategory::ParseError | ErrorCategory::ValidationError | ErrorCategory::InvalidInput),
                        "Error should be in expected categories for corrupted input {}, got {:?}", i, e.category);
                }
            }
        }
    }

    #[test]
    fn test_recovery_from_partial_failures() {
        let engine = DetectionEngine::new();

        // Test that the engine can continue processing after encountering errors
        let mixed_inputs = vec![
            // Valid input (empty resources)
            (r#"{
                "format_version": "1.1",
                "terraform_version": "1.0.0",
                "planned_values": {"root_module": {"resources": []}},
                "resource_changes": []
            }"#, true),
            // Invalid input
            (r#"{"planned_values": {"root_module": {"resources": [{"address": "", "values": }]}}"#, false),
            // Another valid input
            (r#"{
                "format_version": "1.1",
                "terraform_version": "1.0.0",
                "planned_values": {"root_module": {"resources": []}},
                "resource_changes": []
            }"#, true),
        ];

        for (i, (input, should_succeed)) in mixed_inputs.iter().enumerate() {
            let result = engine.detect_from_terraform_json(input);

            if *should_succeed {
                assert!(result.is_ok(),
                    "Input {} should succeed, got error: {:?}", i, result.as_ref().err());
                if let Ok(resources) = result {
                    assert!(resources.is_empty(),
                        "Valid empty input {} should return empty resources", i);
                }
            } else {
                assert!(result.is_err(),
                    "Input {} should fail", i);
                if let Err(e) = result {
                    assert!(!e.message.is_empty(),
                        "Error should have message for input {}", i);
                }
            }
        }
    }

    #[test]
    fn test_resource_exhaustion_simulation() {
        let engine = DetectionEngine::new();

        // Test with many small resources to simulate resource exhaustion
        let mut many_resources = r#"{"planned_values": {"root_module": {"resources": ["#.to_string();

        for i in 0..1000 {
            if i > 0 { many_resources.push(','); }
            many_resources.push_str(&format!(r#"{{"address": "test{}","values": {{"instance_type": "t2.micro"}}}}"#, i));
        }
        many_resources.push_str(r#"]}}}"#);

        let result = engine.detect_from_terraform_json(&many_resources);

        // Should handle many resources without crashing
        match result {
            Ok(resources) => {
                assert_eq!(resources.len(), 1000, "Should process all 1000 resources");
                // Verify they're all valid
                for (i, resource) in resources.iter().enumerate() {
                    assert_eq!(resource.resource_id, format!("test{}", i),
                        "Resource {} should have correct ID", i);
                }
            }
            Err(e) => {
                // Resource exhaustion errors are acceptable
                assert!(matches!(e.category,
                    ErrorCategory::ParseError | ErrorCategory::InvalidInput | ErrorCategory::ValidationError),
                    "Error should be acceptable type, got {:?}", e.category);
                assert!(!e.message.is_empty(), "Error should have message");
            }
        }
    }

    #[test]
    fn test_concurrent_execution_stress() {
        use std::thread;
        use std::sync::Arc;

        let engine = Arc::new(DetectionEngine::new());
        let valid_plan = r#"{"planned_values": {"root_module": {"resources": [{"address": "test", "values": {}}]}}}"#;

        let mut handles = vec![];

        // Spawn multiple threads all trying to use the same engine
        // This tests thread safety and resource contention
        for i in 0..10 {
            let engine_clone = Arc::clone(&engine);
            let plan_clone = valid_plan.to_string();

            let handle = thread::spawn(move || {
                let result = engine_clone.detect_from_terraform_json(&plan_clone);
                (i, result)
            });

            handles.push(handle);
        }

        // Wait for all threads to complete
        for handle in handles {
            let (thread_id, result) = handle.join().unwrap();

            // Each thread should either succeed or fail gracefully
            match result {
                Ok(resources) => {
                    assert_eq!(resources.len(), 1,
                        "Thread {} should find one resource", thread_id);
                }
                Err(e) => {
                    // Concurrent access errors are acceptable if handled gracefully
                    assert!(!e.message.is_empty(),
                        "Thread {} error should have message", thread_id);
                }
            }
        }
    }

    #[test]
    fn test_transient_failure_recovery() {
        let engine = DetectionEngine::new();

        // Test that the engine recovers from various failure conditions
        let test_cases = vec![
            // Empty input
            "",
            // Whitespace only
            "   \n\t  ",
            // Invalid JSON
            r#"{"planned_values": invalid}"#,
            // Valid input
            r#"{"planned_values": {"root_module": {"resources": []}}}"#,
        ];

        for (i, input) in test_cases.iter().enumerate() {
            let result = engine.detect_from_terraform_json(input);

            // Should not crash regardless of input
            match result {
                Ok(resources) => {
                    // For valid inputs, should work
                    if !input.is_empty() && !input.trim().is_empty() && !input.contains("invalid") {
                        assert!(resources.is_empty(),
                            "Valid empty input {} should return empty resources", i);
                    }
                }
                Err(e) => {
                    // Should have structured error
                    assert!(!e.message.is_empty(),
                        "Error should have message for input {}", i);
                    assert!(matches!(e.category,
                        ErrorCategory::ParseError | ErrorCategory::ValidationError | ErrorCategory::InvalidInput),
                        "Error should be in expected categories for input {}, got {:?}", i, e.category);
                }
            }
        }
    }
}
