// Drift detection with SHA256 checksum verification

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

/// Drift detection result with checksum verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftChecksum {
    /// Resource identifier
    pub resource_id: String,

    /// Resource type (e.g., "aws_instance", "aws_s3_bucket")
    pub resource_type: String,

    /// SHA256 checksum of current state
    pub current_checksum: String,

    /// SHA256 checksum of expected state (from Terraform/IaC)
    pub expected_checksum: String,

    /// Whether drift was detected (checksums don't match)
    pub drift_detected: bool,

    /// Timestamp when check was performed
    pub checked_at: String,

    /// Drifted attributes (if any)
    pub drifted_attributes: Vec<DriftedAttribute>,
}

/// Attribute that has drifted from expected state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftedAttribute {
    /// Attribute path (e.g., "tags.Environment", "instance_type")
    pub path: String,

    /// Expected value from IaC
    pub expected_value: String,

    /// Current actual value
    pub actual_value: String,

    /// Severity of this drift
    pub severity: DriftSeverity,
}

/// Severity level for drift detection
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum DriftSeverity {
    /// Low severity (cosmetic changes, non-critical tags)
    Low,

    /// Medium severity (configuration changes that don't affect security/cost)
    Medium,

    /// High severity (security, cost, or compliance impact)
    High,

    /// Critical severity (severe security or compliance violation)
    Critical,
}

/// Configuration for drift detection
#[derive(Debug, Clone)]
pub struct DriftDetectionConfig {
    /// Attributes to ignore in checksum calculation (e.g., timestamps, UIDs)
    pub ignored_attributes: Vec<String>,

    /// Whether to include metadata in checksum
    pub include_metadata: bool,

    /// Minimum drift severity to report
    pub min_severity: DriftSeverity,
}

impl Default for DriftDetectionConfig {
    fn default() -> Self {
        Self {
            ignored_attributes: vec![
                "id".to_string(),
                "arn".to_string(),
                "created_at".to_string(),
                "updated_at".to_string(),
                "last_modified".to_string(),
            ],
            include_metadata: false,
            min_severity: DriftSeverity::Low,
        }
    }
}

/// Drift detector using SHA256 checksums
pub struct DriftDetector {
    config: DriftDetectionConfig,
}

impl Default for DriftDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl DriftDetector {
    /// Create new drift detector with default config
    pub fn new() -> Self {
        Self {
            config: DriftDetectionConfig::default(),
        }
    }

    /// Create drift detector with custom config
    pub fn with_config(config: DriftDetectionConfig) -> Self {
        Self { config }
    }

    /// Calculate SHA256 checksum of resource configuration
    pub fn calculate_checksum(&self, config: &HashMap<String, serde_json::Value>) -> String {
        let filtered = self.filter_config(config);
        // Canonicalize JSON by parsing through serde to normalize key order
        let json = serde_json::to_value(&filtered).unwrap();
        let canonical = serde_json::to_string(&json).unwrap_or_default();

        let mut hasher = Sha256::new();
        hasher.update(canonical.as_bytes());
        let result = hasher.finalize();

        format!("{:x}", result)
    }

    /// Filter out ignored attributes from configuration
    fn filter_config(
        &self,
        config: &HashMap<String, serde_json::Value>,
    ) -> HashMap<String, serde_json::Value> {
        config
            .iter()
            .filter(|(key, _)| !self.config.ignored_attributes.contains(key))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }

    /// Detect drift between current and expected state
    pub fn detect_drift(
        &self,
        resource_id: String,
        resource_type: String,
        expected_config: &HashMap<String, serde_json::Value>,
        current_config: &HashMap<String, serde_json::Value>,
    ) -> DriftChecksum {
        let expected_checksum = self.calculate_checksum(expected_config);
        let current_checksum = self.calculate_checksum(current_config);
        let drift_detected = expected_checksum != current_checksum;

        let drifted_attributes = if drift_detected {
            self.find_drifted_attributes(expected_config, current_config)
        } else {
            Vec::new()
        };

        DriftChecksum {
            resource_id,
            resource_type,
            current_checksum,
            expected_checksum,
            drift_detected,
            checked_at: chrono::Utc::now().to_rfc3339(),
            drifted_attributes,
        }
    }

    /// Find specific attributes that have drifted
    pub fn find_drifted_attributes(
        &self,
        expected: &HashMap<String, serde_json::Value>,
        current: &HashMap<String, serde_json::Value>,
    ) -> Vec<DriftedAttribute> {
        let mut drifted = Vec::new();

        // Check all expected keys
        for (key, expected_value) in expected {
            if self.config.ignored_attributes.contains(key) {
                continue;
            }

            match current.get(key) {
                Some(current_value) if current_value != expected_value => {
                    let severity = self.determine_severity(key, expected_value, current_value);
                    if severity >= self.config.min_severity {
                        drifted.push(DriftedAttribute {
                            path: key.clone(),
                            expected_value: expected_value.to_string(),
                            actual_value: current_value.to_string(),
                            severity,
                        });
                    }
                }
                None => {
                    // Attribute missing in current state
                    drifted.push(DriftedAttribute {
                        path: key.clone(),
                        expected_value: expected_value.to_string(),
                        actual_value: "null".to_string(),
                        severity: DriftSeverity::High,
                    });
                }
                _ => {
                    // Values match, no drift
                }
            }
        }

        // Check for unexpected keys in current state
        for key in current.keys() {
            if !expected.contains_key(key) && !self.config.ignored_attributes.contains(key) {
                drifted.push(DriftedAttribute {
                    path: key.clone(),
                    expected_value: "null".to_string(),
                    actual_value: current.get(key).unwrap().to_string(),
                    severity: DriftSeverity::Medium,
                });
            }
        }

        drifted
    }

    /// Determine severity of drift based on attribute
    fn determine_severity(
        &self,
        key: &str,
        _expected: &serde_json::Value,
        _current: &serde_json::Value,
    ) -> DriftSeverity {
        // Critical attributes
        if key.contains("security") || key.contains("iam") || key.contains("policy") {
            return DriftSeverity::Critical;
        }

        // High severity attributes
        if key.contains("encryption")
            || key.contains("public")
            || key.contains("vpc")
            || key.contains("subnet")
            || key.contains("instance_type")
        {
            return DriftSeverity::High;
        }

        // Medium severity attributes
        if key.contains("tags") || key.contains("name") || key.contains("description") {
            return DriftSeverity::Medium;
        }

        // Default to medium
        DriftSeverity::Medium
    }

    /// Verify checksum matches expected value
    pub fn verify_checksum(
        &self,
        config: &HashMap<String, serde_json::Value>,
        expected_checksum: &str,
    ) -> bool {
        let actual_checksum = self.calculate_checksum(config);
        actual_checksum == expected_checksum
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn create_test_config() -> HashMap<String, serde_json::Value> {
        let mut config = HashMap::new();
        config.insert("instance_type".to_string(), json!("t3.micro"));
        config.insert("ami".to_string(), json!("ami-12345"));
        config.insert("tags".to_string(), json!({"Environment": "prod"}));
        config
    }

    #[test]
    fn test_calculate_checksum_deterministic() {
        let detector = DriftDetector::new();
        let config = create_test_config();

        let checksum1 = detector.calculate_checksum(&config);
        let checksum2 = detector.calculate_checksum(&config);

        assert_eq!(checksum1, checksum2);
        assert_eq!(checksum1.len(), 64); // SHA256 produces 64-char hex string
    }

    #[test]
    fn test_different_configs_different_checksums() {
        let detector = DriftDetector::new();
        let config1 = create_test_config();

        let mut config2 = create_test_config();
        config2.insert("instance_type".to_string(), json!("t3.small"));

        let checksum1 = detector.calculate_checksum(&config1);
        let checksum2 = detector.calculate_checksum(&config2);

        assert_ne!(checksum1, checksum2);
    }

    #[test]
    fn test_ignored_attributes_not_in_checksum() {
        let detector = DriftDetector::new();

        let mut config1 = create_test_config();
        config1.insert("id".to_string(), json!("i-123456"));

        let mut config2 = create_test_config();
        config2.insert("id".to_string(), json!("i-789012"));

        // IDs are different but should be ignored
        let checksum1 = detector.calculate_checksum(&config1);
        let checksum2 = detector.calculate_checksum(&config2);

        assert_eq!(checksum1, checksum2);
    }

    #[test]
    fn test_detect_drift_no_drift() {
        let detector = DriftDetector::new();
        let config = create_test_config();

        let result = detector.detect_drift(
            "i-123456".to_string(),
            "aws_instance".to_string(),
            &config,
            &config,
        );

        assert!(!result.drift_detected);
        assert_eq!(result.current_checksum, result.expected_checksum);
        assert!(result.drifted_attributes.is_empty());
    }

    #[test]
    fn test_detect_drift_with_drift() {
        let detector = DriftDetector::new();
        let expected = create_test_config();

        let mut current = create_test_config();
        current.insert("instance_type".to_string(), json!("t3.large"));

        let result = detector.detect_drift(
            "i-123456".to_string(),
            "aws_instance".to_string(),
            &expected,
            &current,
        );

        assert!(result.drift_detected);
        assert_ne!(result.current_checksum, result.expected_checksum);
        assert!(!result.drifted_attributes.is_empty());
    }

    #[test]
    fn test_find_drifted_attributes() {
        let detector = DriftDetector::new();
        let mut expected = HashMap::new();
        expected.insert("instance_type".to_string(), json!("t3.micro"));
        expected.insert("ami".to_string(), json!("ami-12345"));

        let mut current = HashMap::new();
        current.insert("instance_type".to_string(), json!("t3.large"));
        current.insert("ami".to_string(), json!("ami-12345"));

        let drifted = detector.find_drifted_attributes(&expected, &current);

        assert_eq!(drifted.len(), 1);
        assert_eq!(drifted[0].path, "instance_type");
        assert!(drifted[0].expected_value.contains("t3.micro"));
        assert!(drifted[0].actual_value.contains("t3.large"));
    }

    #[test]
    fn test_severity_determination() {
        let detector = DriftDetector::new();

        // Critical
        let severity =
            detector.determine_severity("security_group", &json!("sg-1"), &json!("sg-2"));
        assert_eq!(severity, DriftSeverity::Critical);

        // High
        let severity =
            detector.determine_severity("encryption_enabled", &json!(true), &json!(false));
        assert_eq!(severity, DriftSeverity::High);

        // Medium
        let severity = detector.determine_severity("tags", &json!({}), &json!({"key": "value"}));
        assert_eq!(severity, DriftSeverity::Medium);
    }

    #[test]
    fn test_verify_checksum() {
        let detector = DriftDetector::new();
        let config = create_test_config();

        let checksum = detector.calculate_checksum(&config);
        assert!(detector.verify_checksum(&config, &checksum));

        let wrong_checksum = "0000000000000000000000000000000000000000000000000000000000000000";
        assert!(!detector.verify_checksum(&config, wrong_checksum));
    }

    #[test]
    fn test_missing_attribute_is_high_severity() {
        let detector = DriftDetector::new();
        let mut expected = HashMap::new();
        expected.insert("critical_setting".to_string(), json!("value"));

        let current = HashMap::new(); // Missing the critical setting

        let drifted = detector.find_drifted_attributes(&expected, &current);

        assert_eq!(drifted.len(), 1);
        assert_eq!(drifted[0].severity, DriftSeverity::High);
        assert_eq!(drifted[0].actual_value, "null");
    }

    #[test]
    fn test_custom_ignored_attributes() {
        let config = DriftDetectionConfig {
            ignored_attributes: vec!["custom_id".to_string(), "timestamp".to_string()],
            include_metadata: false,
            min_severity: DriftSeverity::Low,
        };
        let detector = DriftDetector::with_config(config);

        let mut config1 = HashMap::new();
        config1.insert("value".to_string(), json!("test"));
        config1.insert("custom_id".to_string(), json!("abc"));

        let mut config2 = HashMap::new();
        config2.insert("value".to_string(), json!("test"));
        config2.insert("custom_id".to_string(), json!("xyz"));

        let checksum1 = detector.calculate_checksum(&config1);
        let checksum2 = detector.calculate_checksum(&config2);

        assert_eq!(checksum1, checksum2);
    }

    #[test]
    fn test_min_severity_filter() {
        let config = DriftDetectionConfig {
            ignored_attributes: vec![],
            include_metadata: false,
            min_severity: DriftSeverity::High,
        };
        let detector = DriftDetector::with_config(config);

        let mut expected = HashMap::new();
        expected.insert("tags".to_string(), json!({"env": "prod"})); // Medium severity

        let mut current = HashMap::new();
        current.insert("tags".to_string(), json!({"env": "dev"}));

        let drifted = detector.find_drifted_attributes(&expected, &current);

        // Should be filtered out due to min_severity
        assert!(drifted.is_empty());
    }
}
