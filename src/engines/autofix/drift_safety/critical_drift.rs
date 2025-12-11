// Critical drift blocking for execution safety

use serde::{Deserialize, Serialize};

use super::drift_checksum::{DriftChecksum, DriftSeverity};

/// Critical drift detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CriticalDriftCheck {
    /// Total resources checked
    pub total_resources: usize,

    /// Resources with critical drift
    pub critical_drifts: Vec<CriticalDrift>,

    /// Whether execution should be blocked
    pub should_block: bool,

    /// Blocking reason (if blocked)
    pub blocking_reason: Option<String>,

    /// Timestamp of check
    pub checked_at: String,
}

/// Details of a critical drift that blocks execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CriticalDrift {
    /// Resource identifier
    pub resource_id: String,

    /// Resource type
    pub resource_type: String,

    /// Drifted attribute causing critical issue
    pub attribute: String,

    /// Expected value
    pub expected_value: String,

    /// Actual value
    pub actual_value: String,

    /// Why this drift is critical
    pub reason: CriticalDriftReason,
}

/// Reason why drift is considered critical
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CriticalDriftReason {
    /// Security configuration changed
    SecurityViolation,

    /// Encryption disabled or weakened
    EncryptionDisabled,

    /// IAM policy or permissions changed
    IamPolicyChanged,

    /// Network security configuration changed
    NetworkSecurityChanged,

    /// Compliance-critical setting changed
    ComplianceViolation,

    /// Resource in protected production environment
    ProtectedEnvironment,

    /// Multiple high-severity drifts on same resource
    MultipleHighSeverityDrifts,
}

/// Configuration for critical drift blocking
#[derive(Debug, Clone)]
pub struct CriticalDriftConfig {
    /// Whether to enable critical drift blocking
    pub enabled: bool,

    /// Minimum severity to consider as blocking
    pub min_blocking_severity: DriftSeverity,

    /// Protected resource patterns (e.g., "prod-*", "module.security.*")
    pub protected_patterns: Vec<String>,

    /// Critical attributes that always block
    pub critical_attributes: Vec<String>,

    /// Maximum number of high-severity drifts before blocking
    pub max_high_severity_drifts: usize,
}

impl Default for CriticalDriftConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            min_blocking_severity: DriftSeverity::Critical,
            protected_patterns: vec![
                "prod-*".to_string(),
                "production-*".to_string(),
                "module.security.*".to_string(),
            ],
            critical_attributes: vec![
                "security_group".to_string(),
                "iam_policy".to_string(),
                "iam_role".to_string(),
                "encryption".to_string(),
                "encryption_enabled".to_string(),
                "kms_key_id".to_string(),
                "public".to_string(),
                "publicly_accessible".to_string(),
                "vpc_security_group_ids".to_string(),
            ],
            max_high_severity_drifts: 3,
        }
    }
}

/// Critical drift detector
pub struct CriticalDriftDetector {
    config: CriticalDriftConfig,
}

impl Default for CriticalDriftDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl CriticalDriftDetector {
    /// Create new critical drift detector with default config
    pub fn new() -> Self {
        Self {
            config: CriticalDriftConfig::default(),
        }
    }

    /// Create with custom config
    pub fn with_config(config: CriticalDriftConfig) -> Self {
        Self { config }
    }

    /// Check if critical drift should block execution
    pub fn check_critical_drift(&self, drift_results: Vec<DriftChecksum>) -> CriticalDriftCheck {
        if !self.config.enabled {
            return CriticalDriftCheck {
                total_resources: drift_results.len(),
                critical_drifts: Vec::new(),
                should_block: false,
                blocking_reason: None,
                checked_at: chrono::Utc::now().to_rfc3339(),
            };
        }

        let mut critical_drifts = Vec::new();

        for drift in drift_results.iter().filter(|d| d.drift_detected) {
            // Check for critical severity drifts
            for attr in &drift.drifted_attributes {
                if attr.severity >= self.config.min_blocking_severity {
                    let reason = self.determine_critical_reason(&drift.resource_id, &attr.path);
                    critical_drifts.push(CriticalDrift {
                        resource_id: drift.resource_id.clone(),
                        resource_type: drift.resource_type.clone(),
                        attribute: attr.path.clone(),
                        expected_value: attr.expected_value.clone(),
                        actual_value: attr.actual_value.clone(),
                        reason,
                    });
                }
            }

            // Check for multiple high-severity drifts
            let high_severity_count = drift
                .drifted_attributes
                .iter()
                .filter(|a| a.severity >= DriftSeverity::High)
                .count();

            if high_severity_count >= self.config.max_high_severity_drifts {
                critical_drifts.push(CriticalDrift {
                    resource_id: drift.resource_id.clone(),
                    resource_type: drift.resource_type.clone(),
                    attribute: "multiple".to_string(),
                    expected_value: format!("<{} attributes>", high_severity_count),
                    actual_value: "drifted".to_string(),
                    reason: CriticalDriftReason::MultipleHighSeverityDrifts,
                });
            }

            // Check protected resource patterns
            if self.is_protected_resource(&drift.resource_id) {
                for attr in drift
                    .drifted_attributes
                    .iter()
                    .filter(|a| a.severity >= DriftSeverity::High)
                {
                    critical_drifts.push(CriticalDrift {
                        resource_id: drift.resource_id.clone(),
                        resource_type: drift.resource_type.clone(),
                        attribute: attr.path.clone(),
                        expected_value: attr.expected_value.clone(),
                        actual_value: attr.actual_value.clone(),
                        reason: CriticalDriftReason::ProtectedEnvironment,
                    });
                }
            }
        }

        let should_block = !critical_drifts.is_empty();
        let blocking_reason = if should_block {
            Some(format!(
                "{} critical drift(s) detected that block execution",
                critical_drifts.len()
            ))
        } else {
            None
        };

        CriticalDriftCheck {
            total_resources: drift_results.len(),
            critical_drifts,
            should_block,
            blocking_reason,
            checked_at: chrono::Utc::now().to_rfc3339(),
        }
    }

    /// Determine why drift is critical
    fn determine_critical_reason(
        &self,
        _resource_id: &str,
        attribute: &str,
    ) -> CriticalDriftReason {
        let attr_lower = attribute.to_lowercase();

        if attr_lower.contains("security") || attr_lower.contains("security_group") {
            CriticalDriftReason::SecurityViolation
        } else if attr_lower.contains("encryption") || attr_lower.contains("kms") {
            CriticalDriftReason::EncryptionDisabled
        } else if attr_lower.contains("iam")
            || attr_lower.contains("policy")
            || attr_lower.contains("role")
        {
            CriticalDriftReason::IamPolicyChanged
        } else if attr_lower.contains("vpc")
            || attr_lower.contains("subnet")
            || attr_lower.contains("public")
        {
            CriticalDriftReason::NetworkSecurityChanged
        } else {
            CriticalDriftReason::ComplianceViolation
        }
    }

    /// Check if resource matches protected patterns
    fn is_protected_resource(&self, resource_id: &str) -> bool {
        self.config.protected_patterns.iter().any(|pattern| {
            if pattern.ends_with('*') {
                let prefix = pattern.trim_end_matches('*');
                resource_id.starts_with(prefix)
            } else {
                resource_id == pattern
            }
        })
    }

    /// Generate human-readable summary
    pub fn format_summary(&self, check: &CriticalDriftCheck) -> String {
        let mut output = String::new();

        output.push_str("Critical Drift Check:\n");
        output.push_str(&format!("  Resources checked: {}\n", check.total_resources));
        output.push_str(&format!(
            "  Critical drifts: {}\n",
            check.critical_drifts.len()
        ));
        output.push_str(&format!(
            "  Status: {}\n",
            if check.should_block {
                "🚫 BLOCKED"
            } else {
                "✓ PASSED"
            }
        ));

        if let Some(reason) = &check.blocking_reason {
            output.push_str(&format!("\nBlocking Reason: {}\n", reason));
        }

        if !check.critical_drifts.is_empty() {
            output.push_str("\nCritical Drifts:\n");
            for drift in &check.critical_drifts {
                output.push_str(&format!(
                    "  - {} [{}]: {} (reason: {:?})\n",
                    drift.resource_id, drift.resource_type, drift.attribute, drift.reason
                ));
                output.push_str(&format!("    Expected: {}\n", drift.expected_value));
                output.push_str(&format!("    Actual:   {}\n", drift.actual_value));
            }
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engines::autofix::drift_safety::DriftedAttribute;

    fn create_test_drift(
        resource_id: &str,
        resource_type: &str,
        attributes: Vec<DriftedAttribute>,
    ) -> DriftChecksum {
        DriftChecksum {
            resource_id: resource_id.to_string(),
            resource_type: resource_type.to_string(),
            current_checksum: "current".to_string(),
            expected_checksum: "expected".to_string(),
            drift_detected: !attributes.is_empty(),
            checked_at: chrono::Utc::now().to_rfc3339(),
            drifted_attributes: attributes,
        }
    }

    #[test]
    fn test_no_drift_does_not_block() {
        let detector = CriticalDriftDetector::new();
        let drift_results = vec![create_test_drift("i-123", "aws_instance", vec![])];

        let result = detector.check_critical_drift(drift_results);

        assert!(!result.should_block);
        assert!(result.critical_drifts.is_empty());
        assert!(result.blocking_reason.is_none());
    }

    #[test]
    fn test_critical_security_drift_blocks() {
        let detector = CriticalDriftDetector::new();
        let drift_results = vec![create_test_drift(
            "i-123",
            "aws_instance",
            vec![DriftedAttribute {
                path: "security_group".to_string(),
                expected_value: "sg-12345".to_string(),
                actual_value: "sg-67890".to_string(),
                severity: DriftSeverity::Critical,
            }],
        )];

        let result = detector.check_critical_drift(drift_results);

        assert!(result.should_block);
        assert_eq!(result.critical_drifts.len(), 1);
        assert_eq!(
            result.critical_drifts[0].reason,
            CriticalDriftReason::SecurityViolation
        );
    }

    #[test]
    fn test_encryption_drift_blocks() {
        let detector = CriticalDriftDetector::new();
        let drift_results = vec![create_test_drift(
            "vol-123",
            "aws_ebs_volume",
            vec![DriftedAttribute {
                path: "encryption_enabled".to_string(),
                expected_value: "true".to_string(),
                actual_value: "false".to_string(),
                severity: DriftSeverity::Critical,
            }],
        )];

        let result = detector.check_critical_drift(drift_results);

        assert!(result.should_block);
        assert_eq!(
            result.critical_drifts[0].reason,
            CriticalDriftReason::EncryptionDisabled
        );
    }

    #[test]
    fn test_iam_policy_drift_blocks() {
        let detector = CriticalDriftDetector::new();
        let drift_results = vec![create_test_drift(
            "role-123",
            "aws_iam_role",
            vec![DriftedAttribute {
                path: "iam_policy".to_string(),
                expected_value: "policy-v1".to_string(),
                actual_value: "policy-v2".to_string(),
                severity: DriftSeverity::Critical,
            }],
        )];

        let result = detector.check_critical_drift(drift_results);

        assert!(result.should_block);
        assert_eq!(
            result.critical_drifts[0].reason,
            CriticalDriftReason::IamPolicyChanged
        );
    }

    #[test]
    fn test_multiple_high_severity_drifts_blocks() {
        let detector = CriticalDriftDetector::new();
        let drift_results = vec![create_test_drift(
            "i-123",
            "aws_instance",
            vec![
                DriftedAttribute {
                    path: "instance_type".to_string(),
                    expected_value: "t3.micro".to_string(),
                    actual_value: "t3.large".to_string(),
                    severity: DriftSeverity::High,
                },
                DriftedAttribute {
                    path: "subnet_id".to_string(),
                    expected_value: "subnet-1".to_string(),
                    actual_value: "subnet-2".to_string(),
                    severity: DriftSeverity::High,
                },
                DriftedAttribute {
                    path: "vpc_id".to_string(),
                    expected_value: "vpc-1".to_string(),
                    actual_value: "vpc-2".to_string(),
                    severity: DriftSeverity::High,
                },
            ],
        )];

        let result = detector.check_critical_drift(drift_results);

        assert!(result.should_block);
        assert!(result
            .critical_drifts
            .iter()
            .any(|d| d.reason == CriticalDriftReason::MultipleHighSeverityDrifts));
    }

    #[test]
    fn test_protected_resource_with_high_drift_blocks() {
        let detector = CriticalDriftDetector::new();
        let drift_results = vec![create_test_drift(
            "prod-web-server",
            "aws_instance",
            vec![DriftedAttribute {
                path: "instance_type".to_string(),
                expected_value: "t3.micro".to_string(),
                actual_value: "t3.large".to_string(),
                severity: DriftSeverity::High,
            }],
        )];

        let result = detector.check_critical_drift(drift_results);

        assert!(result.should_block);
        assert!(result
            .critical_drifts
            .iter()
            .any(|d| d.reason == CriticalDriftReason::ProtectedEnvironment));
    }

    #[test]
    fn test_medium_severity_does_not_block() {
        let detector = CriticalDriftDetector::new();
        let drift_results = vec![create_test_drift(
            "i-123",
            "aws_instance",
            vec![DriftedAttribute {
                path: "tags".to_string(),
                expected_value: "prod".to_string(),
                actual_value: "dev".to_string(),
                severity: DriftSeverity::Medium,
            }],
        )];

        let result = detector.check_critical_drift(drift_results);

        assert!(!result.should_block);
        assert!(result.critical_drifts.is_empty());
    }

    #[test]
    fn test_disabled_config_does_not_block() {
        let config = CriticalDriftConfig {
            enabled: false,
            ..Default::default()
        };
        let detector = CriticalDriftDetector::with_config(config);

        let drift_results = vec![create_test_drift(
            "i-123",
            "aws_instance",
            vec![DriftedAttribute {
                path: "security_group".to_string(),
                expected_value: "sg-1".to_string(),
                actual_value: "sg-2".to_string(),
                severity: DriftSeverity::Critical,
            }],
        )];

        let result = detector.check_critical_drift(drift_results);

        assert!(!result.should_block);
    }

    #[test]
    fn test_is_protected_resource() {
        let detector = CriticalDriftDetector::new();

        assert!(detector.is_protected_resource("prod-web-server"));
        assert!(detector.is_protected_resource("production-database"));
        assert!(detector.is_protected_resource("module.security.firewall"));
        assert!(!detector.is_protected_resource("dev-instance"));
        assert!(!detector.is_protected_resource("test-resource"));
    }

    #[test]
    fn test_format_summary_with_critical_drifts() {
        let detector = CriticalDriftDetector::new();
        let drift_results = vec![create_test_drift(
            "i-123",
            "aws_instance",
            vec![DriftedAttribute {
                path: "security_group".to_string(),
                expected_value: "sg-12345".to_string(),
                actual_value: "sg-67890".to_string(),
                severity: DriftSeverity::Critical,
            }],
        )];

        let result = detector.check_critical_drift(drift_results);
        let summary = detector.format_summary(&result);

        assert!(summary.contains("BLOCKED"));
        assert!(summary.contains("Critical Drifts:"));
        assert!(summary.contains("i-123"));
        assert!(summary.contains("security_group"));
    }

    #[test]
    fn test_format_summary_no_drifts() {
        let detector = CriticalDriftDetector::new();
        let drift_results = vec![create_test_drift("i-123", "aws_instance", vec![])];

        let result = detector.check_critical_drift(drift_results);
        let summary = detector.format_summary(&result);

        assert!(summary.contains("PASSED"));
        assert!(!summary.contains("Critical Drifts:"));
    }
}
