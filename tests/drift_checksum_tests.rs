// Tests for drift detection checksum functionality

use costpilot::engines::autofix::drift_safety::{
    DriftDetectionConfig, DriftDetector, DriftSeverity,
};
use serde_json::json;
use std::collections::HashMap;

fn create_ec2_config() -> HashMap<String, serde_json::Value> {
    let mut config = HashMap::new();
    config.insert("instance_type".to_string(), json!("t3.micro"));
    config.insert("ami".to_string(), json!("ami-0abcdef1234567890"));
    config.insert("vpc_security_group_ids".to_string(), json!(["sg-12345"]));
    config.insert("subnet_id".to_string(), json!("subnet-abc123"));
    config.insert(
        "tags".to_string(),
        json!({
            "Name": "web-server",
            "Environment": "production"
        }),
    );
    config
}

#[test]
fn test_checksum_is_sha256_format() {
    let detector = DriftDetector::new();
    let config = create_ec2_config();

    let checksum = detector.calculate_checksum(&config);

    // SHA256 produces 64-character hexadecimal string
    assert_eq!(checksum.len(), 64);
    assert!(checksum.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn test_identical_configs_produce_same_checksum() {
    let detector = DriftDetector::new();
    let config1 = create_ec2_config();
    let config2 = create_ec2_config();

    let checksum1 = detector.calculate_checksum(&config1);
    let checksum2 = detector.calculate_checksum(&config2);

    assert_eq!(checksum1, checksum2);
}

#[test]
fn test_different_instance_types_different_checksums() {
    let detector = DriftDetector::new();
    let config1 = create_ec2_config();
    let mut config2 = create_ec2_config();

    config2.insert("instance_type".to_string(), json!("t3.small"));

    let checksum1 = detector.calculate_checksum(&config1);
    let checksum2 = detector.calculate_checksum(&config2);

    assert_ne!(checksum1, checksum2);
}

#[test]
fn test_ignored_attributes_dont_affect_checksum() {
    let detector = DriftDetector::new();
    let mut config1 = create_ec2_config();
    let mut config2 = create_ec2_config();

    // Add ignored attributes (id, arn, timestamps)
    config1.insert("id".to_string(), json!("i-123456789abcdef01"));
    config1.insert(
        "arn".to_string(),
        json!("arn:aws:ec2:us-east-1:123456789012:instance/i-123456789abcdef01"),
    );
    config1.insert("created_at".to_string(), json!("2025-12-01T10:00:00Z"));

    config2.insert("id".to_string(), json!("i-fedcba9876543210"));
    config2.insert(
        "arn".to_string(),
        json!("arn:aws:ec2:us-east-1:123456789012:instance/i-fedcba9876543210"),
    );
    config2.insert("created_at".to_string(), json!("2025-12-07T15:30:00Z"));

    let checksum1 = detector.calculate_checksum(&config1);
    let checksum2 = detector.calculate_checksum(&config2);

    // Checksums should match despite different ignored attributes
    assert_eq!(checksum1, checksum2);
}

#[test]
fn test_no_drift_detected_when_configs_match() {
    let detector = DriftDetector::new();
    let expected = create_ec2_config();
    let current = create_ec2_config();

    let result = detector.detect_drift(
        "i-123456".to_string(),
        "aws_instance".to_string(),
        &expected,
        &current,
    );

    assert!(!result.drift_detected);
    assert_eq!(result.current_checksum, result.expected_checksum);
    assert!(result.drifted_attributes.is_empty());
}

#[test]
fn test_drift_detected_when_instance_type_changes() {
    let detector = DriftDetector::new();
    let expected = create_ec2_config();
    let mut current = create_ec2_config();
    current.insert("instance_type".to_string(), json!("t3.large"));

    let result = detector.detect_drift(
        "i-123456".to_string(),
        "aws_instance".to_string(),
        &expected,
        &current,
    );

    assert!(result.drift_detected);
    assert_ne!(result.current_checksum, result.expected_checksum);
    assert_eq!(result.drifted_attributes.len(), 1);
    assert_eq!(result.drifted_attributes[0].path, "instance_type");
}

#[test]
fn test_drift_detected_when_tags_modified() {
    let detector = DriftDetector::new();
    let expected = create_ec2_config();
    let mut current = create_ec2_config();
    current.insert(
        "tags".to_string(),
        json!({
            "Name": "web-server",
            "Environment": "staging" // Changed from production
        }),
    );

    let result = detector.detect_drift(
        "i-123456".to_string(),
        "aws_instance".to_string(),
        &expected,
        &current,
    );

    assert!(result.drift_detected);
    assert_eq!(result.drifted_attributes.len(), 1);
    assert_eq!(result.drifted_attributes[0].path, "tags");
}

#[test]
fn test_drift_detected_when_security_group_changes() {
    let detector = DriftDetector::new();
    let expected = create_ec2_config();
    let mut current = create_ec2_config();
    current.insert("vpc_security_group_ids".to_string(), json!(["sg-67890"]));

    let result = detector.detect_drift(
        "i-123456".to_string(),
        "aws_instance".to_string(),
        &expected,
        &current,
    );

    assert!(result.drift_detected);
    assert_eq!(result.drifted_attributes.len(), 1);
    assert_eq!(result.drifted_attributes[0].path, "vpc_security_group_ids");
}

#[test]
fn test_critical_severity_for_security_attributes() {
    let detector = DriftDetector::new();
    let mut expected = HashMap::new();
    expected.insert("security_group".to_string(), json!("sg-12345"));

    let mut current = HashMap::new();
    current.insert("security_group".to_string(), json!("sg-67890"));

    let result = detector.detect_drift(
        "test".to_string(),
        "aws_instance".to_string(),
        &expected,
        &current,
    );

    assert!(result.drift_detected);
    assert_eq!(result.drifted_attributes.len(), 1);
    assert_eq!(
        result.drifted_attributes[0].severity,
        DriftSeverity::Critical
    );
}

#[test]
fn test_high_severity_for_encryption_attributes() {
    let detector = DriftDetector::new();
    let mut expected = HashMap::new();
    expected.insert("encryption_enabled".to_string(), json!(true));

    let mut current = HashMap::new();
    current.insert("encryption_enabled".to_string(), json!(false));

    let result = detector.detect_drift(
        "test".to_string(),
        "aws_ebs_volume".to_string(),
        &expected,
        &current,
    );

    assert!(result.drift_detected);
    assert_eq!(result.drifted_attributes.len(), 1);
    assert_eq!(result.drifted_attributes[0].severity, DriftSeverity::High);
}

#[test]
fn test_medium_severity_for_tag_changes() {
    let detector = DriftDetector::new();
    let mut expected = HashMap::new();
    expected.insert("tags".to_string(), json!({"env": "prod"}));

    let mut current = HashMap::new();
    current.insert("tags".to_string(), json!({"env": "dev"}));

    let result = detector.detect_drift(
        "test".to_string(),
        "aws_instance".to_string(),
        &expected,
        &current,
    );

    assert!(result.drift_detected);
    assert_eq!(result.drifted_attributes.len(), 1);
    assert_eq!(result.drifted_attributes[0].severity, DriftSeverity::Medium);
}

#[test]
fn test_missing_attribute_detected_as_high_severity() {
    let detector = DriftDetector::new();
    let mut expected = HashMap::new();
    expected.insert("required_attribute".to_string(), json!("value"));

    let current = HashMap::new(); // Missing the attribute

    let result = detector.detect_drift(
        "test".to_string(),
        "aws_instance".to_string(),
        &expected,
        &current,
    );

    assert!(result.drift_detected);
    assert_eq!(result.drifted_attributes.len(), 1);
    assert_eq!(result.drifted_attributes[0].severity, DriftSeverity::High);
    assert_eq!(result.drifted_attributes[0].actual_value, "null");
}

#[test]
fn test_unexpected_attribute_detected_as_medium_severity() {
    let detector = DriftDetector::new();
    let expected = HashMap::new();

    let mut current = HashMap::new();
    current.insert("unexpected_attribute".to_string(), json!("value"));

    let result = detector.detect_drift(
        "test".to_string(),
        "aws_instance".to_string(),
        &expected,
        &current,
    );

    assert!(result.drift_detected);
    assert_eq!(result.drifted_attributes.len(), 1);
    assert_eq!(result.drifted_attributes[0].severity, DriftSeverity::Medium);
    assert_eq!(result.drifted_attributes[0].expected_value, "null");
}

#[test]
fn test_verify_checksum_with_correct_value() {
    let detector = DriftDetector::new();
    let config = create_ec2_config();

    let checksum = detector.calculate_checksum(&config);
    assert!(detector.verify_checksum(&config, &checksum));
}

#[test]
fn test_verify_checksum_with_incorrect_value() {
    let detector = DriftDetector::new();
    let config = create_ec2_config();

    let wrong_checksum = "0000000000000000000000000000000000000000000000000000000000000000";
    assert!(!detector.verify_checksum(&config, wrong_checksum));
}

#[test]
fn test_custom_ignored_attributes() {
    let config = DriftDetectionConfig {
        ignored_attributes: vec!["internal_id".to_string(), "last_sync_time".to_string()],
        include_metadata: false,
        min_severity: DriftSeverity::Low,
    };
    let detector = DriftDetector::with_config(config);

    let mut config1 = HashMap::new();
    config1.insert("value".to_string(), json!("test"));
    config1.insert("internal_id".to_string(), json!("abc123"));
    config1.insert("last_sync_time".to_string(), json!("2025-12-07T10:00:00Z"));

    let mut config2 = HashMap::new();
    config2.insert("value".to_string(), json!("test"));
    config2.insert("internal_id".to_string(), json!("xyz789"));
    config2.insert("last_sync_time".to_string(), json!("2025-12-07T15:30:00Z"));

    let checksum1 = detector.calculate_checksum(&config1);
    let checksum2 = detector.calculate_checksum(&config2);

    assert_eq!(checksum1, checksum2);
}

#[test]
fn test_min_severity_filter_excludes_low_severity() {
    let config = DriftDetectionConfig {
        ignored_attributes: vec![],
        include_metadata: false,
        min_severity: DriftSeverity::High,
    };
    let detector = DriftDetector::with_config(config);

    let mut expected = HashMap::new();
    expected.insert("tags".to_string(), json!({"env": "prod"}));

    let mut current = HashMap::new();
    current.insert("tags".to_string(), json!({"env": "dev"}));

    let drifted = detector.find_drifted_attributes(&expected, &current);

    // Tags have Medium severity, should be filtered out
    assert!(drifted.is_empty());
}

#[test]
fn test_multiple_drifted_attributes() {
    let detector = DriftDetector::new();
    let mut expected = HashMap::new();
    expected.insert("instance_type".to_string(), json!("t3.micro"));
    expected.insert("ami".to_string(), json!("ami-12345"));
    expected.insert("tags".to_string(), json!({"env": "prod"}));

    let mut current = HashMap::new();
    current.insert("instance_type".to_string(), json!("t3.large"));
    current.insert("ami".to_string(), json!("ami-67890"));
    current.insert("tags".to_string(), json!({"env": "dev"}));

    let result = detector.detect_drift(
        "i-123456".to_string(),
        "aws_instance".to_string(),
        &expected,
        &current,
    );

    assert!(result.drift_detected);
    assert_eq!(result.drifted_attributes.len(), 3);
}

#[test]
fn test_drift_result_includes_timestamp() {
    let detector = DriftDetector::new();
    let expected = create_ec2_config();
    let mut current = create_ec2_config();
    current.insert("instance_type".to_string(), json!("t3.large"));

    let result = detector.detect_drift(
        "i-123456".to_string(),
        "aws_instance".to_string(),
        &expected,
        &current,
    );

    assert!(!result.checked_at.is_empty());
    assert!(result.checked_at.contains("T")); // ISO 8601 format
    assert!(result.checked_at.contains("Z") || result.checked_at.contains("+"));
    // Timezone
}

#[test]
fn test_resource_metadata_included_in_result() {
    let detector = DriftDetector::new();
    let expected = create_ec2_config();
    let current = create_ec2_config();

    let result = detector.detect_drift(
        "i-123456789abcdef01".to_string(),
        "aws_instance".to_string(),
        &expected,
        &current,
    );

    assert_eq!(result.resource_id, "i-123456789abcdef01");
    // TODO: resource_type no longer stored in Detection, validate via resource_id prefix
}
