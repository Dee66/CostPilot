use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Drift-safe autofix operation with rollback support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftSafeOperation {
    /// Unique operation ID
    pub id: String,

    /// Timestamp when operation was created
    pub created_at: String,

    /// Resource being fixed
    pub resource_id: String,

    /// Type of resource
    pub resource_type: String,

    /// Fix being applied
    pub fix_description: String,

    /// Original state before fix (for rollback)
    pub original_state: ResourceState,

    /// Expected state after fix
    pub target_state: ResourceState,

    /// Current operation status
    pub status: OperationStatus,

    /// Safety checks that must pass
    pub safety_checks: Vec<SafetyCheck>,

    /// Rollback plan
    pub rollback_plan: RollbackPlan,

    /// Execution log
    pub execution_log: Vec<LogEntry>,
}

/// Resource state snapshot for rollback
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceState {
    /// Resource configuration snapshot
    pub config: HashMap<String, serde_json::Value>,

    /// Cost estimate at this state
    pub estimated_cost: f64,

    /// Timestamp of this state
    pub timestamp: String,

    /// Configuration hash for verification
    pub config_hash: String,
}

/// Operation status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OperationStatus {
    /// Pending execution
    Pending,

    /// Safety checks in progress
    ValidatingSafety,

    /// Applying fix
    Applying,

    /// Successfully applied
    Applied,

    /// Failed during application
    Failed,

    /// Rolling back
    RollingBack,

    /// Successfully rolled back
    RolledBack,

    /// Rollback failed (manual intervention needed)
    RollbackFailed,
}

/// Safety check for drift-safe operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyCheck {
    /// Check name
    pub name: String,

    /// Check description
    pub description: String,

    /// Check type
    pub check_type: SafetyCheckType,

    /// Check status
    pub status: CheckStatus,

    /// Result message
    pub message: Option<String>,

    /// When check was performed
    pub checked_at: Option<String>,
}

/// Type of safety check
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SafetyCheckType {
    /// Verify no drift since state snapshot
    NoDrift,

    /// Verify resource still exists
    ResourceExists,

    /// Verify configuration hash matches
    ConfigHashMatch,

    /// Verify cost impact is acceptable
    CostImpactAcceptable,

    /// Verify no policy violations
    NoPolicyViolations,

    /// Verify no SLO violations
    NoSloViolations,

    /// Custom safety check
    Custom(String),
}

/// Check status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CheckStatus {
    /// Not yet checked
    Pending,

    /// Check passed
    Passed,

    /// Check failed
    Failed,

    /// Check skipped
    Skipped,
}

/// Rollback plan for operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackPlan {
    /// Steps to rollback to original state
    pub steps: Vec<RollbackStep>,

    /// Maximum time allowed for rollback
    pub timeout_seconds: u32,

    /// Whether rollback is automatic on failure
    pub auto_rollback: bool,

    /// Rollback status
    pub status: RollbackStatus,
}

/// Single rollback step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackStep {
    /// Step order (lower executes first)
    pub order: u32,

    /// Step description
    pub description: String,

    /// Configuration to restore
    pub restore_config: HashMap<String, serde_json::Value>,

    /// Verification after this step
    pub verification: Option<String>,
}

/// Rollback status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RollbackStatus {
    /// Not triggered
    NotTriggered,

    /// In progress
    InProgress,

    /// Successfully completed
    Completed,

    /// Failed (manual intervention needed)
    Failed,
}

/// Log entry for operation tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    /// Timestamp
    pub timestamp: String,

    /// Log level
    pub level: LogLevel,

    /// Log message
    pub message: String,

    /// Additional context
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<HashMap<String, String>>,
}

/// Log level
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum LogLevel {
    Debug,
    Info,
    Warning,
    Error,
    Critical,
}

/// Drift detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftDetection {
    /// Whether drift was detected
    pub has_drift: bool,

    /// Drifted attributes
    pub drifted_attributes: Vec<DriftedAttribute>,

    /// Drift severity
    pub severity: DriftSeverity,

    /// When drift was detected
    pub detected_at: String,
}

/// Drifted attribute
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftedAttribute {
    /// Attribute name
    pub name: String,

    /// Expected value
    pub expected: serde_json::Value,

    /// Actual value
    pub actual: serde_json::Value,

    /// Impact of drift
    pub impact: String,
}

/// Drift severity
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum DriftSeverity {
    /// Minor drift, safe to proceed
    Minor,

    /// Moderate drift, caution advised
    Moderate,

    /// Major drift, operation should be blocked
    Major,

    /// Critical drift, immediate attention needed
    Critical,
}

impl DriftSafeOperation {
    /// Create a new drift-safe operation
    pub fn new(
        resource_id: String,
        resource_type: String,
        fix_description: String,
        original_state: ResourceState,
        target_state: ResourceState,
    ) -> Self {
        let id = Self::generate_operation_id(&resource_id);

        Self {
            id,
            created_at: Utc::now().to_rfc3339(),
            resource_id,
            resource_type,
            fix_description,
            original_state,
            target_state,
            status: OperationStatus::Pending,
            safety_checks: Vec::new(),
            rollback_plan: RollbackPlan {
                steps: Vec::new(),
                timeout_seconds: 300,
                auto_rollback: true,
                status: RollbackStatus::NotTriggered,
            },
            execution_log: Vec::new(),
        }
    }

    /// Generate unique operation ID
    fn generate_operation_id(resource_id: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let now = Utc::now();
        let mut hasher = DefaultHasher::new();
        resource_id.hash(&mut hasher);
        now.timestamp_millis().hash(&mut hasher);
        let hash = hasher.finish();

        format!("op_{:016x}", hash)
    }

    /// Add safety check
    pub fn add_safety_check(&mut self, check: SafetyCheck) {
        self.safety_checks.push(check);
    }

    /// Add log entry
    pub fn log(&mut self, level: LogLevel, message: String) {
        self.execution_log.push(LogEntry {
            timestamp: Utc::now().to_rfc3339(),
            level,
            message,
            context: None,
        });
    }

    /// Check if all safety checks passed
    pub fn all_safety_checks_passed(&self) -> bool {
        !self.safety_checks.is_empty()
            && self
                .safety_checks
                .iter()
                .all(|c| c.status == CheckStatus::Passed)
    }

    /// Check if operation can proceed
    pub fn can_proceed(&self) -> bool {
        self.status == OperationStatus::Pending && self.all_safety_checks_passed()
    }

    /// Mark operation as failed and trigger rollback
    pub fn mark_failed(&mut self, reason: String) {
        self.status = OperationStatus::Failed;
        self.log(LogLevel::Error, format!("Operation failed: {}", reason));

        if self.rollback_plan.auto_rollback {
            self.trigger_rollback();
        }
    }

    /// Trigger rollback
    pub fn trigger_rollback(&mut self) {
        self.status = OperationStatus::RollingBack;
        self.rollback_plan.status = RollbackStatus::InProgress;
        self.log(LogLevel::Warning, "Rollback triggered".to_string());
    }

    /// Mark rollback as complete
    pub fn mark_rollback_complete(&mut self) {
        self.status = OperationStatus::RolledBack;
        self.rollback_plan.status = RollbackStatus::Completed;
        self.log(
            LogLevel::Info,
            "Rollback completed successfully".to_string(),
        );
    }

    /// Get operation summary
    pub fn summary(&self) -> String {
        format!(
            "Operation {}: {} on {} - Status: {:?}",
            self.id, self.fix_description, self.resource_id, self.status
        )
    }
}

impl ResourceState {
    /// Create a new resource state
    pub fn new(config: HashMap<String, serde_json::Value>, estimated_cost: f64) -> Self {
        let config_hash = Self::compute_hash(&config);

        Self {
            config,
            estimated_cost,
            timestamp: Utc::now().to_rfc3339(),
            config_hash,
        }
    }

    /// Compute configuration hash
    fn compute_hash(config: &HashMap<String, serde_json::Value>) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        let mut keys: Vec<_> = config.keys().collect();
        keys.sort();

        for key in keys {
            key.hash(&mut hasher);
            if let Ok(json_str) = serde_json::to_string(&config[key]) {
                json_str.hash(&mut hasher);
            }
        }

        format!("{:016x}", hasher.finish())
    }

    /// Verify hash matches current config
    pub fn verify_hash(&self) -> bool {
        Self::compute_hash(&self.config) == self.config_hash
    }
}

impl SafetyCheck {
    /// Create a new safety check
    pub fn new(name: String, description: String, check_type: SafetyCheckType) -> Self {
        Self {
            name,
            description,
            check_type,
            status: CheckStatus::Pending,
            message: None,
            checked_at: None,
        }
    }

    /// Mark check as passed
    pub fn mark_passed(&mut self, message: String) {
        self.status = CheckStatus::Passed;
        self.message = Some(message);
        self.checked_at = Some(Utc::now().to_rfc3339());
    }

    /// Mark check as failed
    pub fn mark_failed(&mut self, message: String) {
        self.status = CheckStatus::Failed;
        self.message = Some(message);
        self.checked_at = Some(Utc::now().to_rfc3339());
    }
}

impl DriftDetection {
    /// Create a new drift detection result
    pub fn no_drift() -> Self {
        Self {
            has_drift: false,
            drifted_attributes: Vec::new(),
            severity: DriftSeverity::Minor,
            detected_at: Utc::now().to_rfc3339(),
        }
    }

    /// Create drift detection with findings
    pub fn with_drift(drifted_attributes: Vec<DriftedAttribute>) -> Self {
        let severity = if drifted_attributes.len() > 10 {
            DriftSeverity::Critical
        } else if drifted_attributes.len() > 5 {
            DriftSeverity::Major
        } else if drifted_attributes.len() > 2 {
            DriftSeverity::Moderate
        } else {
            DriftSeverity::Minor
        };

        Self {
            has_drift: true,
            drifted_attributes,
            severity,
            detected_at: Utc::now().to_rfc3339(),
        }
    }

    /// Check if drift is blocking
    pub fn is_blocking(&self) -> bool {
        self.has_drift && self.severity >= DriftSeverity::Major
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_operation_creation() {
        let original = ResourceState::new(HashMap::new(), 100.0);
        let target = ResourceState::new(HashMap::new(), 50.0);

        let op = DriftSafeOperation::new(
            "aws_instance.web".to_string(),
            "aws_instance".to_string(),
            "Downsize to t3.medium".to_string(),
            original,
            target,
        );

        assert_eq!(op.status, OperationStatus::Pending);
        assert!(op.id.starts_with("op_"));
    }

    #[test]
    fn test_safety_checks() {
        let mut op = DriftSafeOperation::new(
            "test".to_string(),
            "test".to_string(),
            "test".to_string(),
            ResourceState::new(HashMap::new(), 100.0),
            ResourceState::new(HashMap::new(), 50.0),
        );

        let mut check = SafetyCheck::new(
            "no_drift".to_string(),
            "Verify no drift".to_string(),
            SafetyCheckType::NoDrift,
        );
        check.mark_passed("No drift detected".to_string());

        op.add_safety_check(check);
        assert!(op.all_safety_checks_passed());
    }

    #[test]
    fn test_failed_safety_check() {
        let mut op = DriftSafeOperation::new(
            "test".to_string(),
            "test".to_string(),
            "test".to_string(),
            ResourceState::new(HashMap::new(), 100.0),
            ResourceState::new(HashMap::new(), 50.0),
        );

        let mut check = SafetyCheck::new(
            "no_drift".to_string(),
            "Verify no drift".to_string(),
            SafetyCheckType::NoDrift,
        );
        check.mark_failed("Drift detected".to_string());

        op.add_safety_check(check);
        assert!(!op.all_safety_checks_passed());
        assert!(!op.can_proceed());
    }

    #[test]
    fn test_config_hash() {
        let mut config = HashMap::new();
        config.insert("instance_type".to_string(), serde_json::json!("t3.large"));

        let state = ResourceState::new(config.clone(), 100.0);
        assert!(state.verify_hash());

        // Modify config
        config.insert("ami".to_string(), serde_json::json!("ami-12345"));
        let new_hash = ResourceState::compute_hash(&config);
        assert_ne!(new_hash, state.config_hash);
    }

    #[test]
    fn test_drift_detection() {
        let drift = DriftDetection::no_drift();
        assert!(!drift.has_drift);
        assert!(!drift.is_blocking());

        let drifted = vec![DriftedAttribute {
            name: "instance_type".to_string(),
            expected: serde_json::json!("t3.medium"),
            actual: serde_json::json!("t3.large"),
            impact: "Cost increase".to_string(),
        }];

        let drift_detected = DriftDetection::with_drift(drifted);
        assert!(drift_detected.has_drift);
        assert_eq!(drift_detected.severity, DriftSeverity::Minor);
    }

    #[test]
    fn test_drift_severity() {
        // Minor drift
        let minor = DriftDetection::with_drift(vec![DriftedAttribute {
            name: "tag".to_string(),
            expected: serde_json::json!("v1"),
            actual: serde_json::json!("v2"),
            impact: "Metadata change".to_string(),
        }]);
        assert_eq!(minor.severity, DriftSeverity::Minor);
        assert!(!minor.is_blocking());

        // Major drift (6+ attributes)
        let major_drift: Vec<DriftedAttribute> = (0..6)
            .map(|i| DriftedAttribute {
                name: format!("attr_{}", i),
                expected: serde_json::json!(i),
                actual: serde_json::json!(i + 1),
                impact: "Change".to_string(),
            })
            .collect();

        let major = DriftDetection::with_drift(major_drift);
        assert_eq!(major.severity, DriftSeverity::Major);
        assert!(major.is_blocking());
    }

    #[test]
    fn test_rollback_trigger() {
        let mut op = DriftSafeOperation::new(
            "test".to_string(),
            "test".to_string(),
            "test".to_string(),
            ResourceState::new(HashMap::new(), 100.0),
            ResourceState::new(HashMap::new(), 50.0),
        );

        op.mark_failed("Test failure".to_string());
        assert_eq!(op.status, OperationStatus::RollingBack);
        assert_eq!(op.rollback_plan.status, RollbackStatus::InProgress);
    }

    #[test]
    fn test_logging() {
        let mut op = DriftSafeOperation::new(
            "test".to_string(),
            "test".to_string(),
            "test".to_string(),
            ResourceState::new(HashMap::new(), 100.0),
            ResourceState::new(HashMap::new(), 50.0),
        );

        op.log(LogLevel::Info, "Starting operation".to_string());
        op.log(LogLevel::Warning, "Safety check warning".to_string());
        op.log(LogLevel::Error, "Operation failed".to_string());

        assert_eq!(op.execution_log.len(), 3);
        assert_eq!(op.execution_log[0].level, LogLevel::Info);
        assert_eq!(op.execution_log[2].level, LogLevel::Error);
    }
}
