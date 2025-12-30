// WASM memory stress tests

#[cfg(test)]
mod tests {
    use costpilot::engines::prediction::PredictionEngine;
    use costpilot::engines::detection::terraform::{convert_to_resource_changes, parse_terraform_plan};
    use costpilot::wasm::{validate_input_size, MemoryTracker, SandboxLimits, ValidationResult};

    /// Generate a large Terraform plan for stress testing
    fn generate_large_plan(num_resources: usize) -> String {
        let mut plan = String::from(r#"{"resource_changes": ["#);

        for i in 0..num_resources {
            if i > 0 {
                plan.push(',');
            }
            plan.push_str(&format!(
                r#"{{
                    "address": "aws_instance.web_{}",
                    "type": "aws_instance",
                    "name": "web_{}",
                    "change": {{
                        "actions": ["create"],
                        "after": {{
                            "instance_type": "t3.medium",
                            "ami": "ami-{:08x}",
                            "tags": {{
                                "Name": "web-server-{}",
                                "Environment": "production",
                                "Team": "platform",
                                "CostCenter": "engineering"
                            }}
                        }}
                    }}
                }}"#,
                i, i, i, i
            ));
        }

        plan.push_str("]}");
        plan
    }

    #[test]
    fn test_memory_limit_validation() {
        let limits = SandboxLimits::default();

        // Small input should pass
        let small_data = vec![0u8; 1024 * 1024]; // 1 MB
        let result = validate_input_size(&small_data, &limits);
        assert!(matches!(result, ValidationResult::Ok));

        // Large input should fail
        let large_data = vec![0u8; 25 * 1024 * 1024]; // 25 MB
        let result = validate_input_size(&large_data, &limits);
        assert!(matches!(
            result,
            ValidationResult::ExceedsFileSize { .. }
        ));
    }

    #[test]
    fn test_small_plan_memory() {
        let _tracker = MemoryTracker::new();

        let plan = generate_large_plan(10); // 10 resources

        let plan_obj = parse_terraform_plan(&plan).expect("Should parse small plan");
        let resources = convert_to_resource_changes(&plan_obj).unwrap();
        assert_eq!(resources.len(), 10);

        // Small plans should complete quickly
        let mut engine = PredictionEngine::new().unwrap();
        let predictions = engine.predict(&resources).unwrap();
        assert_eq!(predictions.len(), 10);
    }

    #[test]
    fn test_medium_plan_memory() {
        let _tracker = MemoryTracker::new();

        let plan = generate_large_plan(1000); // 1k resources

        let plan_obj = parse_terraform_plan(&plan).expect("Should parse medium plan");
        let resources = convert_to_resource_changes(&plan_obj).unwrap();
        assert_eq!(resources.len(), 1000);

        let mut engine = PredictionEngine::new().unwrap();
        let predictions = engine.predict(&resources).unwrap();
        assert_eq!(predictions.len(), 1000);
    }

    #[test]
    fn test_large_plan_memory() {
        let mut tracker = MemoryTracker::new();

        let plan = generate_large_plan(10000); // 10k resources
        tracker.update();

        let plan_obj = parse_terraform_plan(&plan).expect("Should parse large plan");
        let resources = convert_to_resource_changes(&plan_obj).unwrap();
        assert_eq!(resources.len(), 10000);

        tracker.update();

        let mut engine = PredictionEngine::new().unwrap();
        let predictions = engine.predict(&resources).unwrap();
        assert_eq!(predictions.len(), 10000);

        tracker.update();

        // Check peak memory (informational)
        println!("Peak memory usage: {:.2} MB", tracker.peak_usage_mb());

        // Should complete within WASM memory limit (256 MB)
        // This is validated by the WASM runtime itself
    }

    #[test]
    fn test_memory_pressure() {
        // Simulate memory pressure by allocating and releasing

        for iteration in 1..=10 {
            let plan = generate_large_plan(1000);
            let plan_obj = parse_terraform_plan(&plan).unwrap();
            let resources = convert_to_resource_changes(&plan_obj).unwrap();

            let mut engine = PredictionEngine::new().unwrap();
            let predictions = engine.predict(&resources).unwrap();

            assert_eq!(predictions.len(), 1000);

            // Resources and predictions should be dropped here
            drop(resources);
            drop(predictions);

            if iteration % 2 == 0 {
                println!("Completed iteration {}/10", iteration);
            }
        }

        // Should complete without OOM
    }

    #[test]
    fn test_nested_json_depth() {
        use costpilot::wasm::validate_json_depth;

        let limits = SandboxLimits::default();

        // Shallow nesting should pass
        let shallow = r#"{"a": {"b": {"c": "value"}}}"#;
        let result = validate_json_depth(shallow, &limits);
        assert!(matches!(result, ValidationResult::Ok));

        // Deep nesting (within limit) should pass
        let mut deep = String::from("{");
        for i in 0..30 {
            if i > 0 {
                deep.push_str(r#", "#);
            }
            deep.push_str(&format!(r#""level{}": {{"#, i));
        }
        deep.push_str(r#""value": "deepest""#);
        for _ in 0..30 {
            deep.push('}');
        }

        let result = validate_json_depth(&deep, &limits);
        assert!(matches!(result, ValidationResult::Ok));
    }

    #[test]
    fn test_repeated_parsing() {
        // Test memory stability over repeated operations

        let plan = generate_large_plan(100);

        for _ in 0..100 {
            let plan_obj = parse_terraform_plan(&plan).unwrap();
            let resources = convert_to_resource_changes(&plan_obj).unwrap();
            assert_eq!(resources.len(), 100);
        }

        // Should complete without memory leaks
    }

    #[test]
    fn test_concurrent_memory_usage() {
        // Simulate concurrent processing

        let plans: Vec<String> = (0..10).map(|i| generate_large_plan(100 * i + 10)).collect();

        let mut results = Vec::new();
        for plan in &plans {
            let plan_obj = parse_terraform_plan(plan).unwrap();
            let resources = convert_to_resource_changes(&plan_obj).unwrap();
            let mut engine = PredictionEngine::new().unwrap();
            let predictions = engine.predict(&resources).unwrap();
            results.push(predictions.len());
        }

        // Verify all completed
        assert_eq!(results.len(), 10);

        // Memory should be released after processing
    }

    #[test]
    fn test_allocation_patterns() {
        // Test different allocation patterns

        // Pattern 1: Many small allocations
        let small_plans: Vec<String> = (0..1000).map(|_i| generate_large_plan(1)).collect();
        assert_eq!(small_plans.len(), 1000);

        // Pattern 2: Few large allocations
        let large_plans: Vec<String> = (0..10).map(|_i| generate_large_plan(100)).collect();
        assert_eq!(large_plans.len(), 10);

        // Both patterns should complete
    }

    #[test]
    fn test_string_allocation_stress() {
        // Test string allocation under pressure

        let mut strings = Vec::new();
        for i in 0..10000 {
            strings.push(format!("resource_name_{}_with_long_suffix", i));
        }

        assert_eq!(strings.len(), 10000);

        // Verify first and last
        assert!(strings[0].starts_with("resource_name_0"));
        assert!(strings[9999].starts_with("resource_name_9999"));
    }

    #[test]
    fn test_hashmap_allocation_stress() {
        use std::collections::HashMap;

        // Test HashMap allocation
        let mut map = HashMap::new();

        for i in 0..10000 {
            map.insert(format!("key_{}", i), format!("value_{}", i));
        }

        assert_eq!(map.len(), 10000);
        assert_eq!(map.get("key_0"), Some(&"value_0".to_string()));
        assert_eq!(map.get("key_9999"), Some(&"value_9999".to_string()));
    }

    #[test]
    fn test_vector_growth() {
        // Test vector growth patterns

        let mut vec = Vec::new();

        // Grow gradually
        for i in 0..10000 {
            vec.push(i);
        }

        assert_eq!(vec.len(), 10000);
        assert_eq!(vec[0], 0);
        assert_eq!(vec[9999], 9999);

        // Capacity should be reasonable
        let capacity = vec.capacity();
        assert!(capacity >= 10000);
        assert!(capacity < 20000); // Not excessively over-allocated
    }

    #[test]
    fn test_memory_cleanup() {
        // Test that memory is properly released

        {
            let plan = generate_large_plan(5000);
            let plan_obj = parse_terraform_plan(&plan).unwrap();
            let resources = convert_to_resource_changes(&plan_obj).unwrap();
            assert_eq!(resources.len(), 5000);

            // plan, plan_obj, and resources dropped here
        }

        // Memory should be released
        // Subsequent allocations should succeed

        let new_plan = generate_large_plan(5000);
        let plan_obj = parse_terraform_plan(&new_plan).unwrap();
        let resources = convert_to_resource_changes(&plan_obj).unwrap();
        assert_eq!(resources.len(), 5000);
    }
}
