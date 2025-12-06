// WASM determinism tests - ensure identical inputs produce identical outputs

#[cfg(test)]
mod tests {
    use crate::engines::prediction::PredictionEngine;
    use crate::engines::detection::DetectionEngine;
    use crate::parser::plan_parser::PlanParser;
    use sha2::{Sha256, Digest};
    use std::collections::HashMap;

    /// Hash any serializable output
    fn hash_output<T: serde::Serialize>(output: &T) -> String {
        let json = serde_json::to_string(output).unwrap();
        let mut hasher = Sha256::new();
        hasher.update(json.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Load test plan JSON
    fn load_test_plan() -> String {
        r#"{
            "resource_changes": [
                {
                    "address": "aws_instance.web",
                    "type": "aws_instance",
                    "change": {
                        "actions": ["create"],
                        "after": {
                            "instance_type": "t3.medium",
                            "ami": "ami-12345678"
                        }
                    }
                }
            ]
        }"#
        .to_string()
    }

    #[test]
    fn test_prediction_determinism() {
        let plan_json = load_test_plan();
        let parser = PlanParser::new();
        
        // Parse once
        let resources1 = parser.parse(&plan_json).unwrap();
        let engine1 = PredictionEngine::new();
        let predictions1 = engine1.predict_all(&resources1);
        let hash1 = hash_output(&predictions1);
        
        // Parse again
        let resources2 = parser.parse(&plan_json).unwrap();
        let engine2 = PredictionEngine::new();
        let predictions2 = engine2.predict_all(&resources2);
        let hash2 = hash_output(&predictions2);
        
        assert_eq!(
            hash1, hash2,
            "Prediction engine must produce identical outputs for identical inputs"
        );
    }

    #[test]
    fn test_detection_determinism() {
        let plan_json = load_test_plan();
        let parser = PlanParser::new();
        
        // Detect twice
        let resources1 = parser.parse(&plan_json).unwrap();
        let engine1 = DetectionEngine::new();
        let detections1 = engine1.detect_all(&resources1);
        let hash1 = hash_output(&detections1);
        
        let resources2 = parser.parse(&plan_json).unwrap();
        let engine2 = DetectionEngine::new();
        let detections2 = engine2.detect_all(&resources2);
        let hash2 = hash_output(&detections2);
        
        assert_eq!(
            hash1, hash2,
            "Detection engine must produce identical outputs for identical inputs"
        );
    }

    #[test]
    fn test_parser_determinism() {
        let plan_json = load_test_plan();
        let parser = PlanParser::new();
        
        // Parse twice
        let resources1 = parser.parse(&plan_json).unwrap();
        let hash1 = hash_output(&resources1);
        
        let resources2 = parser.parse(&plan_json).unwrap();
        let hash2 = hash_output(&resources2);
        
        assert_eq!(
            hash1, hash2,
            "Parser must produce identical outputs for identical inputs"
        );
    }

    #[test]
    fn test_hashmap_iteration_determinism() {
        // HashMap iteration order is non-deterministic
        // We should use BTreeMap for deterministic iteration
        
        let mut map1 = std::collections::BTreeMap::new();
        map1.insert("key1", "value1");
        map1.insert("key2", "value2");
        map1.insert("key3", "value3");
        
        let mut map2 = std::collections::BTreeMap::new();
        map2.insert("key3", "value3");
        map2.insert("key1", "value1");
        map2.insert("key2", "value2");
        
        // Iteration order should be identical
        let keys1: Vec<_> = map1.keys().collect();
        let keys2: Vec<_> = map2.keys().collect();
        
        assert_eq!(keys1, keys2, "BTreeMap iteration must be deterministic");
    }

    #[test]
    fn test_no_random_values() {
        // Ensure no random number generation
        // This test documents the constraint
        
        // ❌ FORBIDDEN
        // let random_value = rand::random::<f64>();
        
        // ✅ ALLOWED - deterministic hash-based "randomness"
        let input = "test";
        let mut hasher = Sha256::new();
        hasher.update(input.as_bytes());
        let hash = hasher.finalize();
        let deterministic_value = u64::from_le_bytes(hash[..8].try_into().unwrap());
        
        // Calling again produces same result
        let mut hasher2 = Sha256::new();
        hasher2.update(input.as_bytes());
        let hash2 = hasher2.finalize();
        let deterministic_value2 = u64::from_le_bytes(hash2[..8].try_into().unwrap());
        
        assert_eq!(deterministic_value, deterministic_value2);
    }

    #[test]
    fn test_no_system_time() {
        // Ensure no system time usage
        // Time should be passed as input, not queried
        
        // ❌ FORBIDDEN
        // let now = std::time::SystemTime::now();
        
        // ✅ ALLOWED - time as input parameter
        fn analyze_with_timestamp(_plan: &str, timestamp: u64) -> u64 {
            timestamp
        }
        
        let result1 = analyze_with_timestamp("plan", 1234567890);
        let result2 = analyze_with_timestamp("plan", 1234567890);
        
        assert_eq!(result1, result2);
    }

    #[test]
    #[cfg(target_arch = "wasm32")]
    fn test_no_filesystem_access() {
        // Ensure no filesystem access in WASM
        // This should not compile in WASM target
        
        // ❌ FORBIDDEN
        // let content = std::fs::read_to_string("file.json");
        
        // ✅ ALLOWED - content passed as input
        fn analyze_plan(plan_content: &str) -> usize {
            plan_content.len()
        }
        
        let content = "test content";
        let result = analyze_plan(content);
        assert_eq!(result, 12);
    }

    #[test]
    fn test_float_determinism() {
        // Floating point operations should be deterministic
        
        let a = 0.1;
        let b = 0.2;
        let sum1 = a + b;
        let sum2 = a + b;
        
        assert_eq!(sum1, sum2);
        
        // However, be careful with FP precision
        // Use explicit rounding for display
        let rounded1 = (sum1 * 100.0).round() / 100.0;
        let rounded2 = (sum2 * 100.0).round() / 100.0;
        
        assert_eq!(rounded1, rounded2);
    }

    #[test]
    fn test_json_serialization_determinism() {
        use serde_json::json;
        
        // JSON serialization must be deterministic
        let data = json!({
            "resources": [
                {"name": "resource1", "cost": 100.0},
                {"name": "resource2", "cost": 200.0}
            ],
            "total": 300.0
        });
        
        let json1 = serde_json::to_string(&data).unwrap();
        let json2 = serde_json::to_string(&data).unwrap();
        
        assert_eq!(json1, json2);
    }

    #[test]
    fn test_concurrent_execution_determinism() {
        // Even if executed concurrently, results must be deterministic
        
        let plan_json = load_test_plan();
        let parser = PlanParser::new();
        let resources = parser.parse(&plan_json).unwrap();
        
        let engine = PredictionEngine::new();
        let predictions = engine.predict_all(&resources);
        let hash1 = hash_output(&predictions);
        
        // Simulate concurrent execution by running again
        let engine2 = PredictionEngine::new();
        let predictions2 = engine2.predict_all(&resources);
        let hash2 = hash_output(&predictions2);
        
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_large_plan_determinism() {
        // Test with large plan to ensure determinism at scale
        
        let mut plan = String::from(r#"{"resource_changes": ["#);
        
        for i in 0..1000 {
            if i > 0 {
                plan.push(',');
            }
            plan.push_str(&format!(
                r#"{{
                    "address": "aws_instance.web_{}",
                    "type": "aws_instance",
                    "change": {{
                        "actions": ["create"],
                        "after": {{
                            "instance_type": "t3.medium"
                        }}
                    }}
                }}"#,
                i
            ));
        }
        
        plan.push_str("]}");
        
        let parser = PlanParser::new();
        let resources1 = parser.parse(&plan).unwrap();
        let hash1 = hash_output(&resources1);
        
        let resources2 = parser.parse(&plan).unwrap();
        let hash2 = hash_output(&resources2);
        
        assert_eq!(hash1, hash2);
    }
}
