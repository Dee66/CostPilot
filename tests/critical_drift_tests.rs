// Integration tests for critical drift blocking

use costpilot::engines::autofix::drift_safety::{
    CriticalDriftConfig, CriticalDriftDetector, CriticalDriftReason,
    DriftChecksum, DriftedAttribute, DriftSeverity,
};

fn create_drift(
    resource_id: &str,
    resource_type: &str,
    attributes: Vec<(&str, &str, &str, DriftSeverity)>,
) -> DriftChecksum {
    DriftChecksum {
        resource_id: resource_id.to_string(),
        resource_type: resource_type.to_string(),
        current_checksum: "current".to_string(),
        expected_checksum: "expected".to_string(),
        drift_detected: !attributes.is_empty(),
        checked_at: "2025-12-07T10:00:00Z".to_string(),
        drifted_attributes: attributes
            .into_iter()
            .map(|(path, expected, actual, severity)| DriftedAttribute {
                path: path.to_string(),
                expected_value: expected.to_string(),
                actual_value: actual.to_string(),
                severity,
            })
            .collect(),
    }
}

#[test]
fn test_execution_blocked_by_security_group_drift() {
    let detector = CriticalDriftDetector::new();
    let drifts = vec![
        create_drift(
            "i-0123456789abcdef0",
            "aws_instance",
            vec![("vpc_security_group_ids", "[\"sg-12345\"]", "[\"sg-67890\"]", DriftSeverity::Critical)],
        ),
    ];

    let result = detector.check_critical_drift(drifts);

    assert!(result.should_block);
    assert_eq!(result.critical_drifts.len(), 1);
    assert_eq!(result.critical_drifts[0].reason, CriticalDriftReason::SecurityViolation);
    assert!(result.blocking_reason.is_some());
}

#[test]
fn test_execution_blocked_by_encryption_disabled() {
    let detector = CriticalDriftDetector::new();
    let drifts = vec![
        create_drift(
            "vol-0abcdef123456789",
            "aws_ebs_volume",
            vec![("encrypted", "true", "false", DriftSeverity::Critical)],
        ),
    ];

    let result = detector.check_critical_drift(drifts);

    assert!(result.should_block);
    assert_eq!(result.critical_drifts[0].reason, CriticalDriftReason::EncryptionDisabled);
}

#[test]
fn test_execution_blocked_by_iam_policy_change() {
    let detector = CriticalDriftDetector::new();
    let drifts = vec![
        create_drift(
            "role-admin",
            "aws_iam_role",
            vec![("assume_role_policy", "{\"Version\":\"2012-10-17\"}", "{\"Version\":\"2012-10-17\",\"Statement\":[]}", DriftSeverity::Critical)],
        ),
    ];

    let result = detector.check_critical_drift(drifts);

    assert!(result.should_block);
    assert_eq!(result.critical_drifts[0].reason, CriticalDriftReason::IamPolicyChanged);
}

#[test]
fn test_execution_blocked_by_public_access_change() {
    let detector = CriticalDriftDetector::new();
    let drifts = vec![
        create_drift(
            "bucket-sensitive-data",
            "aws_s3_bucket",
            vec![("public_access_block_configuration", "enabled", "disabled", DriftSeverity::Critical)],
        ),
    ];

    let result = detector.check_critical_drift(drifts);

    assert!(result.should_block);
    assert_eq!(result.critical_drifts[0].reason, CriticalDriftReason::NetworkSecurityChanged);
}

#[test]
fn test_execution_blocked_by_multiple_high_severity_drifts() {
    let detector = CriticalDriftDetector::new();
    let drifts = vec![
        create_drift(
            "i-0123456789abcdef0",
            "aws_instance",
            vec![
                ("instance_type", "t3.micro", "t3.2xlarge", DriftSeverity::High),
                ("subnet_id", "subnet-private", "subnet-public", DriftSeverity::High),
                ("key_name", "prod-key", "dev-key", DriftSeverity::High),
            ],
        ),
    ];

    let result = detector.check_critical_drift(drifts);

    assert!(result.should_block);
    assert!(result.critical_drifts.iter().any(|d| 
        d.reason == CriticalDriftReason::MultipleHighSeverityDrifts
    ));
}

#[test]
fn test_execution_blocked_by_protected_production_resource() {
    let detector = CriticalDriftDetector::new();
    let drifts = vec![
        create_drift(
            "prod-web-server-1",
            "aws_instance",
            vec![("instance_type", "t3.large", "t3.xlarge", DriftSeverity::High)],
        ),
    ];

    let result = detector.check_critical_drift(drifts);

    assert!(result.should_block);
    assert_eq!(result.critical_drifts[0].reason, CriticalDriftReason::ProtectedEnvironment);
}

#[test]
fn test_execution_passes_with_no_drift() {
    let detector = CriticalDriftDetector::new();
    let drifts = vec![
        create_drift("i-0123456789abcdef0", "aws_instance", vec![]),
        create_drift("vol-0abcdef123456789", "aws_ebs_volume", vec![]),
    ];

    let result = detector.check_critical_drift(drifts);

    assert!(!result.should_block);
    assert!(result.critical_drifts.is_empty());
    assert!(result.blocking_reason.is_none());
}

#[test]
fn test_execution_passes_with_low_severity_drift() {
    let detector = CriticalDriftDetector::new();
    let drifts = vec![
        create_drift(
            "i-0123456789abcdef0",
            "aws_instance",
            vec![("tags", "{\"Name\":\"server\"}", "{\"Name\":\"web-server\"}", DriftSeverity::Low)],
        ),
    ];

    let result = detector.check_critical_drift(drifts);

    assert!(!result.should_block);
    assert!(result.critical_drifts.is_empty());
}

#[test]
fn test_execution_passes_with_medium_severity_drift() {
    let detector = CriticalDriftDetector::new();
    let drifts = vec![
        create_drift(
            "i-0123456789abcdef0",
            "aws_instance",
            vec![("monitoring", "false", "true", DriftSeverity::Medium)],
        ),
    ];

    let result = detector.check_critical_drift(drifts);

    assert!(!result.should_block);
    assert!(result.critical_drifts.is_empty());
}

#[test]
fn test_multiple_critical_drifts_all_reported() {
    let detector = CriticalDriftDetector::new();
    let drifts = vec![
        create_drift(
            "i-111",
            "aws_instance",
            vec![("security_group", "sg-1", "sg-2", DriftSeverity::Critical)],
        ),
        create_drift(
            "vol-222",
            "aws_ebs_volume",
            vec![("encrypted", "true", "false", DriftSeverity::Critical)],
        ),
        create_drift(
            "role-333",
            "aws_iam_role",
            vec![("iam_policy", "policy-1", "policy-2", DriftSeverity::Critical)],
        ),
    ];

    let result = detector.check_critical_drift(drifts);

    assert!(result.should_block);
    assert_eq!(result.critical_drifts.len(), 3);
}

#[test]
fn test_custom_config_with_different_threshold() {
    let config = CriticalDriftConfig {
        enabled: true,
        min_blocking_severity: DriftSeverity::High,
        ..Default::default()
    };
    let detector = CriticalDriftDetector::with_config(config);
    
    let drifts = vec![
        create_drift(
            "i-0123456789abcdef0",
            "aws_instance",
            vec![("instance_type", "t3.micro", "t3.large", DriftSeverity::High)],
        ),
    ];

    let result = detector.check_critical_drift(drifts);

    assert!(result.should_block);
}

#[test]
fn test_disabled_config_never_blocks() {
    let config = CriticalDriftConfig {
        enabled: false,
        ..Default::default()
    };
    let detector = CriticalDriftDetector::with_config(config);
    
    let drifts = vec![
        create_drift(
            "i-0123456789abcdef0",
            "aws_instance",
            vec![("security_group", "sg-1", "sg-2", DriftSeverity::Critical)],
        ),
    ];

    let result = detector.check_critical_drift(drifts);

    assert!(!result.should_block);
}

#[test]
fn test_custom_protected_patterns() {
    let config = CriticalDriftConfig {
        enabled: true,
        protected_patterns: vec!["critical-*".to_string(), "module.core.*".to_string()],
        min_blocking_severity: DriftSeverity::Critical,
        ..Default::default()
    };
    let detector = CriticalDriftDetector::with_config(config);
    
    let drifts = vec![
        create_drift(
            "critical-database",
            "aws_rds_instance",
            vec![("instance_class", "db.t3.micro", "db.t3.large", DriftSeverity::High)],
        ),
    ];

    let result = detector.check_critical_drift(drifts);

    assert!(result.should_block);
    assert_eq!(result.critical_drifts[0].reason, CriticalDriftReason::ProtectedEnvironment);
}

#[test]
fn test_summary_format_includes_all_details() {
    let detector = CriticalDriftDetector::new();
    let drifts = vec![
        create_drift(
            "i-0123456789abcdef0",
            "aws_instance",
            vec![("security_group", "sg-12345", "sg-67890", DriftSeverity::Critical)],
        ),
    ];

    let result = detector.check_critical_drift(drifts);
    let summary = detector.format_summary(&result);

    assert!(summary.contains("Critical Drift Check:"));
    assert!(summary.contains("Resources checked: 1"));
    assert!(summary.contains("Critical drifts: 1"));
    assert!(summary.contains("BLOCKED"));
    assert!(summary.contains("i-0123456789abcdef0"));
    assert!(summary.contains("aws_instance"));
    assert!(summary.contains("security_group"));
    assert!(summary.contains("Expected: sg-12345"));
    assert!(summary.contains("Actual:   sg-67890"));
}

#[test]
fn test_summary_format_for_passing_check() {
    let detector = CriticalDriftDetector::new();
    let drifts = vec![
        create_drift("i-0123456789abcdef0", "aws_instance", vec![]),
    ];

    let result = detector.check_critical_drift(drifts);
    let summary = detector.format_summary(&result);

    assert!(summary.contains("PASSED"));
    assert!(!summary.contains("Critical Drifts:"));
}

#[test]
fn test_blocking_reason_message() {
    let detector = CriticalDriftDetector::new();
    let drifts = vec![
        create_drift(
            "i-111",
            "aws_instance",
            vec![("security_group", "sg-1", "sg-2", DriftSeverity::Critical)],
        ),
        create_drift(
            "vol-222",
            "aws_ebs_volume",
            vec![("encrypted", "true", "false", DriftSeverity::Critical)],
        ),
    ];

    let result = detector.check_critical_drift(drifts);

    assert!(result.blocking_reason.is_some());
    let reason = result.blocking_reason.unwrap();
    assert!(reason.contains("2"));
    assert!(reason.contains("critical drift"));
}

#[test]
fn test_network_security_changes_detected() {
    let detector = CriticalDriftDetector::new();
    let drifts = vec![
        create_drift(
            "i-0123456789abcdef0",
            "aws_instance",
            vec![("vpc_id", "vpc-private", "vpc-public", DriftSeverity::Critical)],
        ),
    ];

    let result = detector.check_critical_drift(drifts);

    assert!(result.should_block);
    assert_eq!(result.critical_drifts[0].reason, CriticalDriftReason::NetworkSecurityChanged);
}

#[test]
fn test_kms_key_change_detected() {
    let detector = CriticalDriftDetector::new();
    let drifts = vec![
        create_drift(
            "vol-0abcdef123456789",
            "aws_ebs_volume",
            vec![("kms_key_id", "arn:aws:kms:us-east-1:111111111111:key/key1", "arn:aws:kms:us-east-1:111111111111:key/key2", DriftSeverity::Critical)],
        ),
    ];

    let result = detector.check_critical_drift(drifts);

    assert!(result.should_block);
    assert_eq!(result.critical_drifts[0].reason, CriticalDriftReason::EncryptionDisabled);
}

#[test]
fn test_mixed_severity_only_blocks_on_critical() {
    let detector = CriticalDriftDetector::new();
    let drifts = vec![
        create_drift(
            "i-0123456789abcdef0",
            "aws_instance",
            vec![
                ("tags", "{}", "{\"Name\":\"test\"}", DriftSeverity::Low),
                ("monitoring", "false", "true", DriftSeverity::Medium),
                ("security_group", "sg-1", "sg-2", DriftSeverity::Critical),
            ],
        ),
    ];

    let result = detector.check_critical_drift(drifts);

    assert!(result.should_block);
    assert_eq!(result.critical_drifts.len(), 1); // Only the critical one
    assert_eq!(result.critical_drifts[0].attribute, "security_group");
}

#[test]
fn test_exactly_max_high_severity_threshold() {
    let detector = CriticalDriftDetector::new();
    // Default max_high_severity_drifts is 3
    let drifts = vec![
        create_drift(
            "i-0123456789abcdef0",
            "aws_instance",
            vec![
                ("attr1", "val1", "val2", DriftSeverity::High),
                ("attr2", "val1", "val2", DriftSeverity::High),
                ("attr3", "val1", "val2", DriftSeverity::High),
            ],
        ),
    ];

    let result = detector.check_critical_drift(drifts);

    assert!(result.should_block);
}

#[test]
fn test_below_high_severity_threshold_does_not_block() {
    let detector = CriticalDriftDetector::new();
    let drifts = vec![
        create_drift(
            "i-0123456789abcdef0",
            "aws_instance",
            vec![
                ("attr1", "val1", "val2", DriftSeverity::High),
                ("attr2", "val1", "val2", DriftSeverity::High),
            ],
        ),
    ];

    let result = detector.check_critical_drift(drifts);

    assert!(!result.should_block); // Only 2 high-severity, threshold is 3
}
