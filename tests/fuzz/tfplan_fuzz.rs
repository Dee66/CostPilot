// Terraform plan fuzz tests

#[cfg(test)]
mod tfplan_fuzz_tests {
    use proptest::prelude::*;
    use costpilot::validation::terraform_parser::TerraformParser;
    use serde_json::json;

    fn arb_terraform_resource() -> impl Strategy<Value = serde_json::Value> {
        (
            any::<String>(),
            any::<String>(),
            any::<String>(),
            prop::option::of(any::<String>()),
        ).prop_map(|(address, resource_type, name, instance_type)| {
            json!({
                "address": address,
                "type": resource_type,
                "name": name,
                "change": {
                    "actions": ["create"],
                    "after": {
                        "instance_type": instance_type.unwrap_or_else(|| "t3.medium".to_string())
                    }
                }
            })
        })
    }

    fn arb_terraform_plan() -> impl Strategy<Value = serde_json::Value> {
        prop::collection::vec(arb_terraform_resource(), 0..100).prop_map(|resources| {
            json!({
                "format_version": "1.0",
                "terraform_version": "1.5.0",
                "resource_changes": resources
            })
        })
    }

    proptest! {
        #[test]
        fn fuzz_parse_never_panics(
            plan in arb_terraform_plan()
        ) {
            let parser = TerraformParser::new();
            let _ = parser.parse(&plan.to_string());
        }

        #[test]
        fn fuzz_parse_deterministic(
            plan in arb_terraform_plan()
        ) {
            let parser = TerraformParser::new();
            let plan_str = plan.to_string();

            let result1 = parser.parse(&plan_str);
            let result2 = parser.parse(&plan_str);

            match (result1, result2) {
                (Ok(r1), Ok(r2)) => {
                    prop_assert_eq!(r1.len(), r2.len());
                }
                (Err(_), Err(_)) => {},
                _ => prop_assert!(false, "Inconsistent results"),
            }
        }

        #[test]
        fn fuzz_parse_malformed_json(
            text in "\\PC{0,1000}"
        ) {
            let parser = TerraformParser::new();
            let _ = parser.parse(&text);
        }

        #[test]
        fn fuzz_parse_deeply_nested_json(
            depth in 0..100usize
        ) {
            let mut nested = json!("value");
            for _ in 0..depth {
                nested = json!({ "nested": nested });
            }

            let plan = json!({
                "format_version": "1.0",
                "resource_changes": [nested]
            });

            let parser = TerraformParser::new();
            let _ = parser.parse(&plan.to_string());
        }

        #[test]
        fn fuzz_parse_empty_plan(
            _x in 0..1u8
        ) {
            let plan = json!({
                "format_version": "1.0",
                "resource_changes": []
            });

            let parser = TerraformParser::new();
            let result = parser.parse(&plan.to_string());
            prop_assert!(result.is_ok());
        }

        #[test]
        fn fuzz_parse_missing_fields(
            has_version in any::<bool>(),
            has_resources in any::<bool>()
        ) {
            let mut plan = serde_json::Map::new();

            if has_version {
                plan.insert("format_version".to_string(), json!("1.0"));
            }

            if has_resources {
                plan.insert("resource_changes".to_string(), json!([]));
            }

            let parser = TerraformParser::new();
            let _ = parser.parse(&json!(plan).to_string());
        }

        #[test]
        fn fuzz_parse_unicode_content(
            address in "\\PC{1,100}",
            resource_type in "\\PC{1,50}"
        ) {
            let plan = json!({
                "format_version": "1.0",
                "resource_changes": [{
                    "address": address,
                    "type": resource_type,
                    "change": {
                        "actions": ["create"]
                    }
                }]
            });

            let parser = TerraformParser::new();
            let _ = parser.parse(&plan.to_string());
        }

        #[test]
        fn fuzz_parse_extreme_array_sizes(
            size in 0..1000usize
        ) {
            let resources: Vec<_> = (0..size).map(|i| {
                json!({
                    "address": format!("resource_{}", i),
                    "type": "aws_instance",
                    "change": { "actions": ["create"] }
                })
            }).collect();

            let plan = json!({
                "format_version": "1.0",
                "resource_changes": resources
            });

            let parser = TerraformParser::new();
            let _ = parser.parse(&plan.to_string());
        }

        #[test]
        fn fuzz_parse_mixed_types(
            value in prop::oneof![
                Just(json!(null)),
                Just(json!(true)),
                Just(json!(42)),
                Just(json!("string")),
                Just(json!([])),
                Just(json!({})),
            ]
        ) {
            let plan = json!({
                "format_version": "1.0",
                "resource_changes": [value]
            });

            let parser = TerraformParser::new();
            let _ = parser.parse(&plan.to_string());
        }
    }
}
