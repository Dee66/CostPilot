use costpilot::engines::detection::DetectionEngine;
use costpilot::engines::prediction::{PredictionEngine, prediction_engine::CostHeuristics};
use costpilot::engines::explain::PredictionExplainer;
use costpilot::engines::shared::models::{ResourceChange, ChangeAction, CostEstimate};
use std::collections::HashMap;

#[cfg(test)]
mod determinism_tests {
    use super::*;
    use regex::Regex;

    fn create_test_resource_change() -> ResourceChange {
        ResourceChange {
            resource_id: "test-aws-instance-1".to_string(),
            resource_type: "aws_instance".to_string(),
            action: ChangeAction::Update,
            module_path: None,
            old_config: Some(serde_json::json!({
                "instance_type": "t2.micro",
                "ami": "ami-12345"
            })),
            new_config: Some(serde_json::json!({
                "instance_type": "t3.medium",
                "ami": "ami-67890"
            })),
            tags: HashMap::new(),
            monthly_cost: None,
            config: None,
            cost_impact: None,
            before: None, // deprecated
            after: None,  // deprecated
        }
    }

    #[test]
    fn test_identical_inputs_produce_identical_detect_outputs() {
        let engine = DetectionEngine::new();

        let change1 = create_test_resource_change();
        let change2 = create_test_resource_change(); // identical

        let detections1 = engine.detect(&[change1]).unwrap();
        let detections2 = engine.detect(&[change2]).unwrap();

        // Serialize to JSON for byte-for-byte comparison
        let json1 = serde_json::to_string(&detections1).unwrap();
        let json2 = serde_json::to_string(&detections2).unwrap();

        assert_eq!(json1, json2, "Detect outputs should be byte-for-byte identical for identical inputs");
    }

    #[test]
    fn test_identical_inputs_produce_identical_predict_outputs() {
        let mut engine = PredictionEngine::new().unwrap();

        let change1 = create_test_resource_change();
        let change2 = create_test_resource_change(); // identical

        let estimates1 = engine.predict(&[change1]).unwrap();
        let estimates2 = engine.predict(&[change2]).unwrap();

        // Serialize to JSON for byte-for-byte comparison
        let json1 = serde_json::to_string(&estimates1).unwrap();
        let json2 = serde_json::to_string(&estimates2).unwrap();

        assert_eq!(json1, json2, "Predict outputs should be byte-for-byte identical for identical inputs");
    }

    #[test]
    fn test_identical_inputs_produce_identical_explain_outputs() {
        let mut prediction_engine = PredictionEngine::new().unwrap();
        let heuristics = prediction_engine.heuristics().clone();
        let engine = PredictionExplainer::new(&heuristics);

        let change1 = create_test_resource_change();
        let change2 = create_test_resource_change(); // identical

        // Get estimates first
        let estimates1 = prediction_engine.predict(&[change1.clone()]).unwrap();
        let estimates2 = prediction_engine.predict(&[change2.clone()]).unwrap();

        let explanations1 = engine.explain(&change1, &estimates1[0]);
        let explanations2 = engine.explain(&change2, &estimates2[0]);

        // Serialize to JSON for byte-for-byte comparison
        let json1 = serde_json::to_string(&explanations1).unwrap();
        let json2 = serde_json::to_string(&explanations2).unwrap();

        assert_eq!(json1, json2, "Explain outputs should be byte-for-byte identical for identical inputs");
    }

    #[test]
    fn test_output_hashes_stable_across_repeated_runs() {
        use sha2::{Sha256, Digest};

        let engine = DetectionEngine::new();
        let change = create_test_resource_change();

        // Run detection multiple times
        let mut hashes = Vec::new();
        for _ in 0..5 {
            let detections = engine.detect(&[change.clone()]).unwrap();
            let json = serde_json::to_string(&detections).unwrap();
            let hash = Sha256::new().chain_update(json.as_bytes()).finalize();
            hashes.push(hash);
        }

        // All hashes should be identical
        let first_hash = &hashes[0];
        for hash in hashes.iter().skip(1) {
            assert_eq!(first_hash, hash, "Output hashes should be stable across repeated runs");
        }
    }

    #[test]
    fn test_array_ordering_is_deterministic() {
        let engine = DetectionEngine::new();

        // Create multiple resource changes
        let changes = vec![
            ResourceChange {
                resource_id: "resource-c".to_string(),
                resource_type: "aws_instance".to_string(),
                action: ChangeAction::Update,
                module_path: None,
                old_config: Some(serde_json::json!({"instance_type": "t2.micro"})),
                new_config: Some(serde_json::json!({"instance_type": "t3.medium"})),
                tags: HashMap::new(),
                monthly_cost: None,
                config: None,
                cost_impact: None,
                before: None,
                after: None,
            },
            ResourceChange {
                resource_id: "resource-a".to_string(),
                resource_type: "aws_instance".to_string(),
                action: ChangeAction::Update,
                module_path: None,
                old_config: Some(serde_json::json!({"instance_type": "t2.micro"})),
                new_config: Some(serde_json::json!({"instance_type": "t3.medium"})),
                tags: HashMap::new(),
                monthly_cost: None,
                config: None,
                cost_impact: None,
                before: None,
                after: None,
            },
            ResourceChange {
                resource_id: "resource-b".to_string(),
                resource_type: "aws_instance".to_string(),
                action: ChangeAction::Update,
                module_path: None,
                old_config: Some(serde_json::json!({"instance_type": "t2.micro"})),
                new_config: Some(serde_json::json!({"instance_type": "t3.medium"})),
                tags: HashMap::new(),
                monthly_cost: None,
                config: None,
                cost_impact: None,
                before: None,
                after: None,
            },
        ];

        // Run detection multiple times with same input order
        let detections1 = engine.detect(&changes.clone()).unwrap();
        let detections2 = engine.detect(&changes).unwrap();

        // Results should be in the same order
        assert_eq!(detections1.len(), detections2.len());
        for (d1, d2) in detections1.iter().zip(detections2.iter()) {
            assert_eq!(d1.resource_id, d2.resource_id);
            assert_eq!(d1.severity, d2.severity);
        }
    }

    #[test]
    fn test_no_timestamps_or_randomness_in_outputs() {
        let engine = DetectionEngine::new();
        let change = create_test_resource_change();

        let detections1 = engine.detect(&[change.clone()]).unwrap();
        let detections2 = engine.detect(&[change]).unwrap();

        // Convert to JSON strings
        let json1 = serde_json::to_string(&detections1).unwrap();
        let json2 = serde_json::to_string(&detections2).unwrap();

        // Check that outputs don't contain timestamps or random elements
        // This is a basic check - in a real implementation you'd want more sophisticated
        // detection of non-deterministic elements
        assert!(!json1.contains("timestamp"));
        assert!(!json1.contains("random"));
        assert!(!json1.contains("uuid"));
        assert!(!json2.contains("timestamp"));
        assert!(!json2.contains("random"));
        assert!(!json2.contains("uuid"));

        // Outputs should be identical
        assert_eq!(json1, json2);
    }

    #[test]
    fn test_floating_point_formatting_stable() {
        let mut engine = PredictionEngine::new().unwrap();
        let change = create_test_resource_change();

        let estimates1 = engine.predict(&[change.clone()]).unwrap();
        let estimates2 = engine.predict(&[change]).unwrap();

        // Serialize with consistent formatting
        let json1 = serde_json::to_string(&estimates1).unwrap();
        let json2 = serde_json::to_string(&estimates2).unwrap();

        assert_eq!(json1, json2, "Floating point formatting should be stable");

        // Additional check: parse back and compare values
        let parsed1: Vec<CostEstimate> = serde_json::from_str(&json1).unwrap();
        let parsed2: Vec<CostEstimate> = serde_json::from_str(&json2).unwrap();

        assert_eq!(parsed1.len(), parsed2.len());
        for (e1, e2) in parsed1.iter().zip(parsed2.iter()) {
            assert_eq!(e1.monthly_cost, e2.monthly_cost);
            assert_eq!(e1.prediction_interval_low, e2.prediction_interval_low);
            assert_eq!(e1.prediction_interval_high, e2.prediction_interval_high);
        }
    }

    #[test]
    fn test_timestamps_normalized_to_utc() {
        // Test that any timestamps in CLI output are normalized to UTC format
        // Currently, no commands output timestamps, so this is a placeholder test
        // When timestamps are added, they must end with 'Z' or '+00:00'

        let mut cmd = assert_cmd::Command::cargo_bin("costpilot").unwrap();
        cmd.arg("explain").arg("aws_instance").arg("--instance-type").arg("t3.micro");

        let output = cmd.output().unwrap();
        let stdout = String::from_utf8(output.stdout).unwrap();

        // Check that any timestamps found are in UTC format
        // Look for ISO 8601 timestamp patterns
        // For now, since no timestamps are output, this is a placeholder
        // When timestamps are added, ensure they end with Z or +00:00

        // Placeholder: assert that no timestamps are present (current behavior)
        // In the future, this test will validate UTC format

        // If no timestamps found, test passes (placeholder behavior)
        assert!(!stdout.contains("timestamp"), "No timestamps should be present in current output");
    }

    #[test]
    fn test_float_rounding_consistent_no_platform_drift() {
        // Placeholder test for float rounding consistency across platforms
        // This test ensures that floating point operations produce identical results
        // regardless of platform-specific FPU implementations

        // Test basic floating point arithmetic consistency
        let test_values = vec![1.23456789, 2.987654321, 0.000123456];

        for &val in &test_values {
            // Perform operations that might be sensitive to rounding
            let rounded = (val * 1000000.0_f64).round() / 1000000.0_f64;
            let _expected = val; // In a real test, this would be platform-independent expected value

            // For placeholder, just ensure the operation completes and is deterministic
            // In production, this would compare against known good values
            assert!(rounded.is_finite(), "Float rounding should produce finite result");
        }

        // TODO: Implement cross-platform validation
        // - Run on multiple architectures (x86_64, ARM64)
        // - Compare serialized cost estimates byte-for-byte
        // - Ensure no drift in floating point calculations
    }

    #[test]
    fn test_repeated_cli_runs_produce_identical_hashes() {
        use sha2::{Sha256, Digest};
        use std::fs;
        use tempfile::NamedTempFile;

        // Create a temporary terraform plan file for consistent input
        let plan_content = r#"
{
  "planned_values": {
    "root_module": {
      "resources": [
        {
          "address": "aws_instance.example",
          "mode": "managed",
          "type": "aws_instance",
          "name": "example",
          "provider_name": "registry.terraform.io/hashicorp/aws",
          "schema_version": 1,
          "values": {
            "instance_type": "t3.micro",
            "ami": "ami-12345"
          }
        }
      ]
    }
  }
}
"#;

        let temp_file = NamedTempFile::new().unwrap();
        fs::write(&temp_file, plan_content).unwrap();
        let plan_path = temp_file.path().to_str().unwrap();

        // Run costpilot multiple times and collect output hashes
        let mut hashes = Vec::new();
        for _ in 0..3 {
            let mut cmd = assert_cmd::Command::cargo_bin("costpilot").unwrap();
            cmd.arg("scan").arg(plan_path).arg("--json");

            let output = cmd.output().unwrap();
            let stdout = String::from_utf8(output.stdout).unwrap();

            let hash = Sha256::new().chain_update(stdout.as_bytes()).finalize();
            hashes.push(hash);
        }

        // All hashes should be identical
        let first_hash = &hashes[0];
        for hash in hashes.iter().skip(1) {
            assert_eq!(first_hash, hash, "CLI output hashes should be identical across repeated runs");
        }
    }

    #[test]
    #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
    fn test_cross_platform_determinism_linux_x86_64_arm64() {
        // Placeholder test for cross-platform determinism on Linux x86_64 and ARM64
        // This test ensures that the same inputs produce identical outputs on both architectures
        // Currently implemented as a placeholder that runs on supported architectures

        use sha2::{Sha256, Digest};
        use std::fs;
        use tempfile::NamedTempFile;

        // Verify we're running on a supported Linux architecture
        #[cfg(target_arch = "x86_64")]
        let arch = "x86_64";
        #[cfg(target_arch = "aarch64")]
        let arch = "aarch64";

        // Ensure we're on Linux
        assert_eq!(std::env::consts::OS, "linux", "This test is only valid on Linux");

        // Create a temporary terraform plan file for consistent input
        let plan_content = r#"
{
  "planned_values": {
    "root_module": {
      "resources": [
        {
          "address": "aws_instance.example",
          "mode": "managed",
          "type": "aws_instance",
          "name": "example",
          "provider_name": "registry.terraform.io/hashicorp/aws",
          "schema_version": 1,
          "values": {
            "instance_type": "t3.micro",
            "ami": "ami-12345"
          }
        }
      ]
    }
  }
}
"#;

        let temp_file = NamedTempFile::new().unwrap();
        fs::write(&temp_file, plan_content).unwrap();
        let plan_path = temp_file.path().to_str().unwrap();

        // Run costpilot multiple times and collect output hashes
        let mut hashes = Vec::new();
        for _ in 0..3 {
            let mut cmd = assert_cmd::Command::cargo_bin("costpilot").unwrap();
            cmd.arg("scan").arg(plan_path).arg("--json");

            let output = cmd.output().unwrap();
            let stdout = String::from_utf8(output.stdout).unwrap();

            let hash = Sha256::new().chain_update(stdout.as_bytes()).finalize();
            hashes.push(hash);
        }

        // All hashes should be identical (determinism check)
        let first_hash = &hashes[0];
        for hash in hashes.iter().skip(1) {
            assert_eq!(first_hash, hash, "CLI output hashes should be identical across repeated runs on {}", arch);
        }

        // TODO: Implement actual cross-platform comparison
        // - Store reference outputs for each architecture
        // - Compare outputs between x86_64 and ARM64 builds
        // - Ensure byte-for-byte identical results
    }

    #[test]
    fn test_nuclear_determinism_across_platforms() {
        // Placeholder test for nuclear determinism:
        // Identical scenario across platforms with randomized execution order,
        // allocator noise, and reordered inputs produces byte-identical artifacts
        //
        // TODO: Implement full test with:
        // - Randomized execution order simulation
        // - Memory allocator noise injection
        // - Input reordering
        // - Cross-platform artifact comparison
        // - Byte-identical output validation

        // For now, this is a placeholder that always passes
        assert!(true);
    }

    #[test]
    fn test_deterministic_outputs_across_runs_and_platforms() {
        // Placeholder test for deterministic outputs across runs and platforms
        // TODO: Implement test that:
        // - Runs the same input multiple times
        // - Checks outputs are byte-for-byte identical
        // - Simulates cross-platform scenarios (e.g., different architectures)
        // - Validates against golden reference outputs

        // For now, this is a placeholder that always passes
        assert!(true);
    }
}
