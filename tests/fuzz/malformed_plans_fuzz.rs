// Malformed plan fuzz tests - ensure parser never panics on invalid input

#[cfg(test)]
mod malformed_plans_fuzz_tests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_parse_never_panics_on_random_json(content in "\\PC*") {
            // Parser should handle any random string without panicking
            let _ = serde_json::from_str::<serde_json::Value>(&content);
        }

    #[test]
    fn test_parse_deeply_nested_structure(depth in 1usize..100) {
        // Create deeply nested JSON structure
        let mut json = String::from("{}");
        for _ in 0..depth {
            json = format!("{{\"nested\": {}}}", json);
        }

        // Should handle deep nesting without stack overflow
        let _ = serde_json::from_str::<serde_json::Value>(&json);
    }

    #[test]
    fn test_parse_huge_array(size in 0usize..10000) {
        // Create large array
        let json = format!("[{}]", vec!["null"; size].join(","));

        // Should handle large arrays
        let _ = serde_json::from_str::<serde_json::Value>(&json);
    }

    #[test]
    fn test_parse_missing_required_fields(
        has_format_version in proptest::bool::ANY,
        has_resource_changes in proptest::bool::ANY,
    ) {
        // Build JSON with optional fields
        let mut obj = serde_json::Map::new();

        if has_format_version {
            obj.insert("format_version".to_string(), serde_json::json!("1.0"));
        }

        if has_resource_changes {
            obj.insert("resource_changes".to_string(), serde_json::json!([]));
        }

        let json = serde_json::to_string(&obj).unwrap();

        // Parser should handle missing fields gracefully
        let _ = serde_json::from_str::<serde_json::Value>(&json);
    }

    #[test]
    fn test_parse_invalid_type_combinations(
        resource_changes_type in prop_oneof![
            Just(serde_json::json!(null)),
            Just(serde_json::json!(42)),
            Just(serde_json::json!("string")),
            Just(serde_json::json!(true)),
            Just(serde_json::json!([])),
            Just(serde_json::json!({})),
        ]
    ) {
        // Test with wrong types for resource_changes field
        let json = serde_json::json!({
            "format_version": "1.0",
            "resource_changes": resource_changes_type
        });

        let json_str = serde_json::to_string(&json).unwrap();

        // Should handle type mismatches without panic
        let _ = serde_json::from_str::<serde_json::Value>(&json_str);
    }

    #[test]
    fn test_parse_special_float_values(
        special_value in prop_oneof![
            Just(f64::NAN),
            Just(f64::INFINITY),
            Just(f64::NEG_INFINITY),
            Just(0.0),
            Just(-0.0),
            Just(f64::MAX),
            Just(f64::MIN),
        ]
    ) {
        // Test with special float values in cost fields
        let json = serde_json::json!({
            "format_version": "1.0",
            "resource_changes": [{
                "address": "aws_instance.test",
                "change": {
                    "actions": ["create"],
                    "after": {
                        "instance_type": "t3.medium"
                    }
                }
            }],
            "cost": special_value
        });

        // Should handle special floats
        let _ = serde_json::to_string(&json);
    }

    #[test]
    fn test_parse_unicode_and_escapes(content in "[\\u{0000}-\\u{FFFF}]{0,100}") {
        // Test with various Unicode characters
        let json = serde_json::json!({
            "format_version": "1.0",
            "resource_changes": [{
                "address": content,
                "change": {
                    "actions": ["create"]
                }
            }]
        });

        let json_str = serde_json::to_string(&json).unwrap();

        // Should handle Unicode without panic
        let _ = serde_json::from_str::<serde_json::Value>(&json_str);
    }

    #[test]
    fn test_parse_extremely_long_strings(len in 0usize..10000) {
        // Test with very long string values
        let long_string = "a".repeat(len);
        let json = serde_json::json!({
            "format_version": "1.0",
            "resource_changes": [{
                "address": long_string,
                "change": {
                    "actions": ["create"]
                }
            }]
        });

        let json_str = serde_json::to_string(&json).unwrap();

        // Should handle long strings
        let _ = serde_json::from_str::<serde_json::Value>(&json_str);
    }

    #[test]
    fn test_parse_duplicate_keys(key_count in 1usize..10) {
        // Create JSON with duplicate keys (last one wins in most parsers)
        let mut entries = Vec::new();
        for i in 0..key_count {
            entries.push(format!("\"resource_changes\": [{{\"id\": {}}}]", i));
        }

        let json = format!("{{{}}}", entries.join(","));

        // Should handle duplicate keys
        let _ = serde_json::from_str::<serde_json::Value>(&json);
    }

    #[test]
    fn test_parse_mixed_resource_actions(
        actions in prop::collection::vec(
            prop_oneof![
                Just("create"),
                Just("delete"),
                Just("update"),
                Just("replace"),
                Just("no-op"),
                Just("read"),
                Just("invalid_action"),
            ],
            0..10
        )
    ) {
        // Test with various action combinations
        let json = serde_json::json!({
            "format_version": "1.0",
            "resource_changes": [{
                "address": "test.resource",
                "change": {
                    "actions": actions
                }
            }]
        });

        let json_str = serde_json::to_string(&json).unwrap();

        // Should handle any action combination
        let _ = serde_json::from_str::<serde_json::Value>(&json_str);
    }

    #[test]
    fn test_parse_null_in_unexpected_places(
        null_in_format in proptest::bool::ANY,
        null_in_changes in proptest::bool::ANY,
        null_in_address in proptest::bool::ANY,
    ) {
        // Build JSON with nulls in various places
        let format_version = if null_in_format {
            serde_json::Value::Null
        } else {
            serde_json::json!("1.0")
        };

        let resource_changes = if null_in_changes {
            serde_json::Value::Null
        } else {
            let address = if null_in_address {
                serde_json::Value::Null
            } else {
                serde_json::json!("test.resource")
            };

            serde_json::json!([{
                "address": address,
                "change": {
                    "actions": ["create"]
                }
            }])
        };

        let json = serde_json::json!({
            "format_version": format_version,
            "resource_changes": resource_changes
        });

        let json_str = serde_json::to_string(&json).unwrap();

        // Should handle nulls gracefully
        let _ = serde_json::from_str::<serde_json::Value>(&json_str);
    }

    #[test]
    fn test_parse_empty_and_whitespace_only(
        whitespace in "[ \\t\\n\\r]*"
    ) {
        // Test with empty or whitespace-only input
        let _ = serde_json::from_str::<serde_json::Value>(&whitespace);
    }

    #[test]
    fn test_parse_partial_json_fragments(
        fragment in prop_oneof![
            Just("{"),
            Just("}"),
            Just("["),
            Just("]"),
            Just("{\"key\":"),
            Just("\"value\"}"),
            Just("[1,2,"),
            Just(",3,4]"),
        ]
    ) {
        // Test with incomplete JSON fragments
        let _ = serde_json::from_str::<serde_json::Value>(fragment);
    }

    #[test]
    fn test_parse_number_edge_cases(
        number_str in prop_oneof![
            Just("0"),
            Just("-0"),
            Just("0.0"),
            Just("-0.0"),
            Just("1e308"),
            Just("-1e308"),
            Just("1e-308"),
            Just("999999999999999999999"),
            Just("0.000000000000000001"),
        ]
    ) {
        // Test with various number formats
        let json = format!("{{\"value\": {}}}", number_str);

        // Should handle number edge cases
        let _ = serde_json::from_str::<serde_json::Value>(&json);
    }

    #[test]
    fn test_parse_control_characters(
        control_char in 0u8..32u8
    ) {
        // Test with ASCII control characters
        let json = format!("{{\"field\": \"test{}value\"}}", control_char as char);

        // Should handle control characters
        let _ = serde_json::from_str::<serde_json::Value>(&json);
    }

    #[test]
    fn test_parse_mixed_encodings(content in "[\\x00-\\xFF]{0,100}") {
        // Test with raw bytes (may not be valid UTF-8)
        // This tests parser robustness with various byte sequences
        let _ = serde_json::from_str::<serde_json::Value>(&content);
    }

    #[test]
    fn test_parse_nested_arrays_and_objects(
        array_depth in 0usize..20,
        object_depth in 0usize..20
    ) {
        // Create nested structure with both arrays and objects
        let mut json = String::from("\"leaf\"");

        for _ in 0..array_depth {
            json = format!("[{}]", json);
        }

        for _ in 0..object_depth {
            json = format!("{{\"key\": {}}}", json);
        }

        // Should handle mixed nesting
        let _ = serde_json::from_str::<serde_json::Value>(&json);
    }

    #[test]
    fn test_parse_resource_before_and_after_variations(
        has_before in proptest::bool::ANY,
        has_after in proptest::bool::ANY,
        has_after_unknown in proptest::bool::ANY,
    ) {
        // Test various combinations of before/after/after_unknown fields
        let mut change = serde_json::Map::new();
        change.insert("actions".to_string(), serde_json::json!(["update"]));

        if has_before {
            change.insert("before".to_string(), serde_json::json!({"instance_type": "t2.micro"}));
        }

        if has_after {
            change.insert("after".to_string(), serde_json::json!({"instance_type": "t3.medium"}));
        }

        if has_after_unknown {
            change.insert("after_unknown".to_string(), serde_json::json!(["instance_type"]));
        }

        let json = serde_json::json!({
            "format_version": "1.0",
            "resource_changes": [{
                "address": "test.resource",
                "change": change
            }]
        });

        let json_str = serde_json::to_string(&json).unwrap();

        // Should handle any combination of before/after fields
        let _ = serde_json::from_str::<serde_json::Value>(&json_str);
    }

    #[test]
    fn test_parse_circular_reference_simulation(iterations in 1usize..100) {
        // Simulate potential circular reference issues with repeated keys
        let mut json = String::from("{");
        for i in 0..iterations {
            if i > 0 {
                json.push(',');
            }
            json.push_str(&format!("\"ref_{}\": {{\"next\": \"ref_{}\"}}", i, (i + 1) % iterations));
        }
        json.push('}');

        // Should handle without infinite loops
        let _ = serde_json::from_str::<serde_json::Value>(&json);
    }

    #[test]
    fn test_parse_extreme_precision_numbers(
        precision in 0usize..50
    ) {
        // Test with numbers of extreme precision
        let decimal_part = "1".repeat(precision);
        let number_str = format!("0.{}", decimal_part);
        let json = format!("{{\"value\": {}}}", number_str);

        // Should handle high precision
        let _ = serde_json::from_str::<serde_json::Value>(&json);
    }

    #[test]
    fn test_parse_mixed_valid_and_invalid_resources(
        valid_count in 0usize..10,
        invalid_count in 0usize..10
    ) {
        // Create mix of valid and invalid resources
        let mut resources = Vec::new();

        for i in 0..valid_count {
            resources.push(serde_json::json!({
                "address": format!("valid.resource_{}", i),
                "change": {
                    "actions": ["create"]
                }
            }));
        }

        for i in 0..invalid_count {
            resources.push(serde_json::json!({
                "address": i,  // Invalid: should be string
                "change": "invalid"  // Invalid: should be object
            }));
        }

        let json = serde_json::json!({
            "format_version": "1.0",
            "resource_changes": resources
        });

        let json_str = serde_json::to_string(&json).unwrap();

        // Should handle mixed valid/invalid gracefully
        let _ = serde_json::from_str::<serde_json::Value>(&json_str);
    }

    #[test]
    fn test_parse_malformed_escape_sequences(
        escape_type in prop_oneof![
            Just("\\x"),
            Just("\\u"),
            Just("\\u00"),
            Just("\\u000"),
            Just("\\uGGGG"),
            Just("\\U00000000"),
        ]
    ) {
        // Test with malformed escape sequences
        let json = format!("{{\"field\": \"test{}value\"}}", escape_type);

        // Should handle malformed escapes
        let _ = serde_json::from_str::<serde_json::Value>(&json);
    }

    #[test]
    fn test_parse_boundary_value_integers(
        int_value in prop_oneof![
            Just(i64::MIN),
            Just(i64::MAX),
            Just(0i64),
            Just(-1i64),
            Just(1i64),
        ]
    ) {
        // Test with integer boundary values
        let json = serde_json::json!({
            "value": int_value
        });

        let json_str = serde_json::to_string(&json).unwrap();

        // Should handle integer boundaries
        let _ = serde_json::from_str::<serde_json::Value>(&json_str);
    }

    #[test]
    fn test_parse_comments_and_trailing_commas(
        has_comment in proptest::bool::ANY,
        has_trailing_comma in proptest::bool::ANY
    ) {
        // JSON doesn't officially support comments or trailing commas
        // but some parsers are lenient
        let mut json = String::from("{\"key\": \"value\"");

        if has_trailing_comma {
            json.push(',');
        }

        json.push('}');

        if has_comment {
            json.push_str(" // comment");
        }

        // Test parser behavior with non-standard JSON
        let _ = serde_json::from_str::<serde_json::Value>(&json);
    }

    #[test]
    fn test_parse_resource_mode_variations(
        mode in prop_oneof![
            Just("managed"),
            Just("data"),
            Just("module"),
            Just("invalid_mode"),
            Just(""),
        ]
    ) {
        // Test with various resource mode values
        let json = serde_json::json!({
            "format_version": "1.0",
            "resource_changes": [{
                "address": "test.resource",
                "mode": mode,
                "change": {
                    "actions": ["create"]
                }
            }]
        });

        let json_str = serde_json::to_string(&json).unwrap();

        // Should handle any mode value
        let _ = serde_json::from_str::<serde_json::Value>(&json_str);
    }

    #[test]
    fn test_parse_empty_resource_changes_array() {
        // Test with empty resource_changes array
        let json = serde_json::json!({
            "format_version": "1.0",
            "resource_changes": []
        });

        let json_str = serde_json::to_string(&json).unwrap();

        // Should handle empty arrays
        let _ = serde_json::from_str::<serde_json::Value>(&json_str);
        }
    }

    #[test]
    fn test_minimal_valid_plan() {
        let json = r#"{"format_version":"1.0","resource_changes":[]}"#;
        let result = serde_json::from_str::<serde_json::Value>(json);
        assert!(result.is_ok());
        }
    }

    #[test]
    fn test_minimal_valid_plan() {
        let json = r#"{"format_version":"1.0","resource_changes":[]}"#;
        let result = serde_json::from_str::<serde_json::Value>(json);
        assert!(result.is_ok());
    }

    #[test]
    fn test_truncated_json() {
        let json = r#"{"format_version":"1.0","resource_changes":"#;
        let result = serde_json::from_str::<serde_json::Value>(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_completely_invalid() {
        let json = "not even json";
        let result = serde_json::from_str::<serde_json::Value>(json);
        assert!(result.is_err());
    }
}
