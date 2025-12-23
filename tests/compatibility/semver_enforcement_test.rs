// Semver enforcement tests for API stability

use costpilot::engines::prediction::heuristics_loader::HeuristicsLoader;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_heuristics_version_compatibility() {
    // Test that compatible versions are accepted
    let compatible_versions = vec!["1.0.0", "1.0.1", "1.1.0", "1.2.5"];

    for version in compatible_versions {
        let result = create_test_heuristics_with_version(version);
        assert!(result.is_ok(), "Version {} should be compatible", version);
    }
}

#[test]
fn test_heuristics_version_too_old() {
    // Test that old versions are rejected
    let old_versions = vec!["0.9.0", "0.5.2"];

    for version in old_versions {
        let result = create_test_heuristics_with_version(version);
        assert!(
            result.is_err(),
            "Version {} should be rejected (too old)",
            version
        );
        if let Err(e) = result {
            assert!(
                e.to_string().contains("too old"),
                "Error should mention version is too old: {}",
                e
            );
        }
    }
}

#[test]
fn test_heuristics_version_too_new() {
    // Test that incompatible major versions are rejected
    let new_versions = vec!["2.0.0", "3.0.0", "10.5.2"];

    for version in new_versions {
        let result = create_test_heuristics_with_version(version);
        assert!(
            result.is_err(),
            "Version {} should be rejected (incompatible major)",
            version
        );
        if let Err(e) = result {
            assert!(
                e.to_string().contains("not compatible"),
                "Error should mention incompatibility: {}",
                e
            );
        }
    }
}

#[test]
fn test_invalid_version_format() {
    // Test that invalid version formats are rejected
    let invalid_versions = vec!["1", "invalid", "1.0.0.0.beta", "not-a-version"];

    for version in invalid_versions {
        let result = create_test_heuristics_with_version(version);
        // Some may fail at parse, others at validation
        assert!(
            result.is_err(),
            "Version '{}' should be rejected (invalid format)",
            version
        );
    }
}

#[test]
fn test_api_contract_exists() {
    // Test that API contract golden file exists
    let contract_path = PathBuf::from("tests/golden/api_contract.json");
    assert!(
        contract_path.exists(),
        "API contract golden file must exist"
    );

    // Test that it's valid JSON
    let content = fs::read_to_string(&contract_path).expect("Failed to read API contract");
    let parsed: serde_json::Value =
        serde_json::from_str(&content).expect("API contract must be valid JSON");

    // Verify required fields
    assert!(
        parsed.get("version").is_some(),
        "API contract must have version"
    );
    assert!(
        parsed.get("public_structs").is_some(),
        "API contract must document public structs"
    );
    assert!(
        parsed.get("public_enums").is_some(),
        "API contract must document public enums"
    );
    assert!(
        parsed.get("cli_commands").is_some(),
        "API contract must document CLI commands"
    );
    assert!(
        parsed.get("backwards_compatibility_policy").is_some(),
        "API contract must have compatibility policy"
    );
}

#[test]
fn test_error_code_stability() {
    // Test that error codes follow stable pattern
    let error_categories = vec![
        ("validation", "E100-E199"),
        ("policy", "E200-E299"),
        ("baselines", "E300-E399"),
        ("slo", "E400-E499"),
    ];

    // Read API contract
    let contract_path = PathBuf::from("tests/golden/api_contract.json");
    let content = fs::read_to_string(&contract_path).expect("Failed to read API contract");
    let contract: serde_json::Value =
        serde_json::from_str(&content).expect("API contract must be valid JSON");

    let error_codes = contract
        .get("error_codes")
        .expect("API contract must have error_codes");

    for (category, range) in error_categories {
        let code_range = error_codes
            .get(category)
            .and_then(|v| v.as_str())
            .expect(&format!("Error code range for {} must exist", category));
        assert_eq!(
            code_range, range,
            "Error code range for {} must be stable",
            category
        );
    }
}

#[test]
fn test_performance_budgets_documented() {
    // Test that performance budgets are documented in API contract
    let contract_path = PathBuf::from("tests/golden/api_contract.json");
    let content = fs::read_to_string(&contract_path).expect("Failed to read API contract");
    let contract: serde_json::Value =
        serde_json::from_str(&content).expect("API contract must be valid JSON");

    let budgets = contract
        .get("performance_budgets")
        .expect("API contract must document performance budgets");

    // Verify all required budgets are documented
    let required_budgets = vec![
        "prediction_latency_ms",
        "mapping_latency_ms",
        "autofix_latency_ms",
        "total_scan_latency_ms",
        "slo_eval_latency_ms",
        "wasm_memory_mb",
    ];

    for budget in required_budgets {
        assert!(
            budgets.get(budget).is_some(),
            "Performance budget '{}' must be documented",
            budget
        );
    }
}

#[test]
fn test_determinism_guarantees_documented() {
    // Test that determinism guarantees are documented
    let contract_path = PathBuf::from("tests/golden/api_contract.json");
    let content = fs::read_to_string(&contract_path).expect("Failed to read API contract");
    let contract: serde_json::Value =
        serde_json::from_str(&content).expect("API contract must be valid JSON");

    let guarantees = contract
        .get("determinism_guarantees")
        .and_then(|v| v.as_array())
        .expect("API contract must document determinism guarantees");

    assert!(
        !guarantees.is_empty(),
        "Must have at least one determinism guarantee"
    );

    // Check for key guarantees
    let guarantee_text = guarantees
        .iter()
        .map(|v| v.as_str().unwrap_or(""))
        .collect::<Vec<_>>()
        .join(" ");

    assert!(
        guarantee_text.contains("Identical inputs"),
        "Must guarantee identical inputs produce identical outputs"
    );
    assert!(
        guarantee_text.contains("zero-IAM") || guarantee_text.contains("No network"),
        "Must guarantee zero-IAM/no network calls"
    );
}

// Helper function to create test heuristics with specific version
fn create_test_heuristics_with_version(version: &str) -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let heuristics_path = temp_dir.path().join("cost_heuristics.json");

    // Create minimal valid heuristics JSON
    let heuristics_json = format!(
        r#"{{
        "version": "{}",
        "last_updated": "2025-12-07",
        "description": "Test heuristics",
        "default_region": "us-east-1",
        "compute": {{
            "ec2": {{
                "t3.micro": {{ "hourly": 0.0104, "monthly": 7.6 }}
            }},
            "lambda": {{
                "price_per_gb_second": 0.0000166667,
                "price_per_request": 0.0000002,
                "free_tier_requests": 1000000,
                "free_tier_compute_gb_seconds": 400000,
                "default_memory_mb": 128,
                "default_timeout_seconds": 3,
                "default_duration_ms": 1000
            }}
        }},
        "storage": {{
            "ebs": {{
                "gp3": {{ "per_gb": 0.08 }}
            }},
            "s3": {{
                "standard": {{ "per_gb": 0.023, "first_50tb_per_gb": 0.023 }},
                "glacier": {{ "per_gb": 0.004, "first_50tb_per_gb": null }},
                "requests": {{ "put_copy_post_list_per_1000": 0.005, "get_select_per_1000": 0.0004 }}
            }}
        }},
        "database": {{
            "rds": {{
                "mysql": {{
                    "db.t3.micro": {{ "hourly": 0.017, "monthly": 12.41 }}
                }},
                "postgres": {{
                    "db.t3.micro": {{ "hourly": 0.018, "monthly": 13.14 }}
                }},
                "storage_gp2_per_gb": 0.115,
                "storage_gp3_per_gb": 0.125,
                "backup_per_gb": 0.095
            }},
            "dynamodb": {{
                "on_demand": {{
                    "write_request_unit": 1.25,
                    "read_request_unit": 0.25,
                    "storage_per_gb": 0.25
                }},
                "provisioned": {{
                    "write_capacity_unit_hourly": 0.00065,
                    "read_capacity_unit_hourly": 0.00013,
                    "storage_per_gb": 0.25
                }},
                "storage_per_gb": 0.25
            }}
        }},
        "networking": {{
            "nat_gateway": {{
                "hourly": 0.045,
                "monthly": 32.85,
                "data_processing_per_gb": 0.045
            }},
            "load_balancer": {{
                "alb": {{
                    "hourly": 0.0225,
                    "monthly": 16.43,
                    "lcu_hourly": 0.008
                }}
            }}
        }},
        "cold_start_defaults": {{
            "dynamodb_unknown_rcu": 5,
            "dynamodb_unknown_wcu": 5,
            "lambda_default_invocations": 1000000,
            "nat_gateway_default_gb": 100,
            "s3_default_gb": 100,
            "ec2_default_utilization": 0.7
        }},
        "prediction_intervals": {{
            "range_factor": 0.3
        }}
    }}"#,
        version
    );

    fs::write(&heuristics_path, heuristics_json)?;

    // Try to load it
    let loader = HeuristicsLoader::new();
    loader.load_from_file(&heuristics_path)?;

    Ok(())
}
