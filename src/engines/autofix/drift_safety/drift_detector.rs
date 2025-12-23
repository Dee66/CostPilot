// Drift detector implementation

use crate::engines::autofix::drift_safety::drift_checksum::{
    DriftChecksum, DriftDetector as ChecksumDriftDetector,
};
use crate::engines::shared::models::ResourceChange;

/// Infrastructure drift detector that combines checksum verification with cloud provider queries
pub struct DriftDetector {
    checksum_detector: ChecksumDriftDetector,
}

impl Default for DriftDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl DriftDetector {
    /// Create new drift detector
    pub fn new() -> Self {
        Self {
            checksum_detector: ChecksumDriftDetector::new(),
        }
    }

    /// Detect infrastructure drift by comparing IaC state with actual cloud state
    pub fn detect_infrastructure_drift(
        &self,
        change: &ResourceChange,
    ) -> Result<DriftChecksum, Box<dyn std::error::Error>> {
        // In a real implementation, this would query the cloud provider API
        // For now, we'll simulate drift detection based on the resource change

        // Convert configs to HashMap
        let expected_config = Self::config_to_hashmap(&change.old_config);
        let current_config = Self::config_to_hashmap(&change.new_config);

        // Use checksum detector to find drift
        let drift_result = self.checksum_detector.detect_drift(
            change.resource_id.clone(),
            change.resource_type.clone(),
            &expected_config,
            &current_config,
        );

        Ok(drift_result)
    }

    /// Convert Option<serde_json::Value> to HashMap
    fn config_to_hashmap(
        config: &Option<serde_json::Value>,
    ) -> std::collections::HashMap<String, serde_json::Value> {
        if let Some(serde_json::Value::Object(map)) = config {
            map.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
        } else {
            std::collections::HashMap::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engines::shared::models::ChangeAction;

    #[test]
    fn test_detect_infrastructure_drift() {
        let detector = DriftDetector::new();

        let change = ResourceChange::builder()
            .resource_id("aws_instance.test".to_string())
            .resource_type("aws_instance".to_string())
            .action(ChangeAction::Update)
            .old_config(serde_json::json!({"instance_type": "t3.medium"}))
            .new_config(serde_json::json!({"instance_type": "t3.xlarge"}))
            .build();

        let result = detector.detect_infrastructure_drift(&change);
        assert!(result.is_ok());

        let drift = result.unwrap();
        assert_eq!(drift.resource_id, "aws_instance.test");
        assert_eq!(drift.resource_type, "aws_instance");
        assert!(drift.drift_detected); // Should detect drift between different instance types
    }
}
