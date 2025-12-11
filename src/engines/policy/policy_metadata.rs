use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Rich metadata for policy management and governance
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PolicyMetadata {
    /// Unique policy identifier
    pub id: String,

    /// Human-readable policy name
    pub name: String,

    /// Detailed description of policy purpose
    pub description: String,

    /// Policy category for organization
    pub category: PolicyCategory,

    /// Severity level for violations
    pub severity: Severity,

    /// Current policy status
    pub status: PolicyStatus,

    /// Version tracking
    pub version: String,

    /// Ownership and responsibility
    pub ownership: PolicyOwnership,

    /// Lifecycle management
    pub lifecycle: PolicyLifecycle,

    /// Tags for categorization and search
    #[serde(default)]
    pub tags: HashSet<String>,

    /// Links to documentation and related resources
    #[serde(default)]
    pub links: PolicyLinks,

    /// Execution and impact metrics
    #[serde(default)]
    pub metrics: PolicyMetrics,

    /// Additional custom metadata
    #[serde(default)]
    pub custom: HashMap<String, String>,
}

/// Policy category for organization
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum PolicyCategory {
    /// Cost budget and spending limits
    Budget,

    /// Resource count and type restrictions
    Resource,

    /// Security and compliance rules
    Security,

    /// Tagging and naming conventions
    Governance,

    /// Performance and optimization
    Performance,

    /// Service Level Objectives
    Slo,

    /// Environmental (dev/staging/prod) policies
    Environment,

    /// Custom category
    Custom(String),
}

/// Severity level for policy violations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// Informational only, no action required
    Info,

    /// Warning that should be reviewed
    Warning,

    /// Error that should be fixed
    Error,

    /// Critical issue requiring immediate attention
    Critical,
}

impl Severity {
    /// Check if this severity should block deployment
    pub fn is_blocking(&self) -> bool {
        matches!(self, Severity::Error | Severity::Critical)
    }

    /// Get numeric score for severity (higher = more severe)
    pub fn score(&self) -> u8 {
        match self {
            Severity::Info => 1,
            Severity::Warning => 2,
            Severity::Error => 3,
            Severity::Critical => 4,
        }
    }
}

/// Policy status in lifecycle
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum PolicyStatus {
    /// Policy is in draft and not enforced
    Draft,

    /// Policy is active and enforced
    Active,

    /// Policy is temporarily disabled
    Disabled,

    /// Policy is deprecated, will be removed
    Deprecated,

    /// Policy is archived (historical record)
    Archived,
}

impl PolicyStatus {
    /// Check if policy should be enforced
    pub fn is_enforced(&self) -> bool {
        matches!(self, PolicyStatus::Active)
    }

    /// Check if policy is active or in use
    pub fn is_active(&self) -> bool {
        matches!(self, PolicyStatus::Active | PolicyStatus::Disabled)
    }
}

/// Ownership and responsibility information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PolicyOwnership {
    /// Policy author (creator)
    pub author: String,

    /// Current policy owner (responsible party)
    pub owner: String,

    /// Team or department responsible
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team: Option<String>,

    /// Contact email for questions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contact: Option<String>,

    /// Reviewers who approved the policy
    #[serde(default)]
    pub reviewers: Vec<String>,
}

/// Policy lifecycle management
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PolicyLifecycle {
    /// When policy was created
    pub created_at: DateTime<Utc>,

    /// When policy was last modified
    pub updated_at: DateTime<Utc>,

    /// When policy becomes effective (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effective_from: Option<DateTime<Utc>>,

    /// When policy expires (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effective_until: Option<DateTime<Utc>>,

    /// Deprecation information if status is deprecated
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deprecation: Option<DeprecationInfo>,

    /// Revision history
    #[serde(default)]
    pub revisions: Vec<PolicyRevision>,
}

impl PolicyLifecycle {
    /// Check if policy is currently effective based on dates
    pub fn is_effective_now(&self) -> bool {
        let now = Utc::now();

        // Check effective_from
        if let Some(from) = self.effective_from {
            if now < from {
                return false;
            }
        }

        // Check effective_until
        if let Some(until) = self.effective_until {
            if now > until {
                return false;
            }
        }

        true
    }
}

/// Deprecation information for policies being phased out
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DeprecationInfo {
    /// When policy was deprecated
    pub deprecated_at: DateTime<Utc>,

    /// Reason for deprecation
    pub reason: String,

    /// Replacement policy ID (if any)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replacement_policy_id: Option<String>,

    /// Migration guide or instructions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub migration_guide: Option<String>,
}

/// Policy revision history entry
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PolicyRevision {
    /// Version number
    pub version: String,

    /// When this revision was made
    pub timestamp: DateTime<Utc>,

    /// Who made the revision
    pub author: String,

    /// Description of changes
    pub changes: String,
}

/// Links to external resources
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct PolicyLinks {
    /// Link to policy documentation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub documentation: Option<String>,

    /// Link to runbook or remediation guide
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runbook: Option<String>,

    /// Link to ticket/issue tracker
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ticket: Option<String>,

    /// Additional related links
    #[serde(default)]
    pub related: Vec<String>,
}

/// Policy execution and impact metrics
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct PolicyMetrics {
    /// Total number of evaluations
    #[serde(default)]
    pub evaluation_count: u64,

    /// Number of violations detected
    #[serde(default)]
    pub violation_count: u64,

    /// Number of exemptions granted
    #[serde(default)]
    pub exemption_count: u64,

    /// Last evaluation timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_evaluated: Option<DateTime<Utc>>,

    /// Last violation timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_violation: Option<DateTime<Utc>>,

    /// Average violation rate (violations per evaluation)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub violation_rate: Option<f64>,
}

impl PolicyMetrics {
    /// Record an evaluation
    pub fn record_evaluation(&mut self, has_violation: bool) {
        self.evaluation_count += 1;
        if has_violation {
            self.violation_count += 1;
            self.last_violation = Some(Utc::now());
        }
        self.last_evaluated = Some(Utc::now());
        self.update_violation_rate();
    }

    /// Record an exemption
    pub fn record_exemption(&mut self) {
        self.exemption_count += 1;
    }

    /// Update violation rate calculation
    fn update_violation_rate(&mut self) {
        if self.evaluation_count > 0 {
            self.violation_rate = Some(self.violation_count as f64 / self.evaluation_count as f64);
        }
    }
}

/// Complete policy with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyWithMetadata<T> {
    /// Policy metadata
    pub metadata: PolicyMetadata,

    /// Policy specification/rules
    pub spec: T,
}

impl<T> PolicyWithMetadata<T> {
    /// Create a new policy with metadata
    pub fn new(metadata: PolicyMetadata, spec: T) -> Self {
        Self { metadata, spec }
    }

    /// Check if policy should be enforced
    pub fn should_enforce(&self) -> bool {
        self.metadata.status.is_enforced() && self.metadata.lifecycle.is_effective_now()
    }

    /// Check if violation should block deployment
    pub fn is_blocking(&self) -> bool {
        self.metadata.severity.is_blocking()
    }
}

impl PolicyMetadata {
    /// Create a new basic policy metadata
    pub fn new(
        id: String,
        name: String,
        description: String,
        category: PolicyCategory,
        severity: Severity,
        author: String,
        owner: String,
    ) -> Self {
        let now = Utc::now();

        Self {
            id,
            name,
            description,
            category,
            severity,
            status: PolicyStatus::Draft,
            version: "1.0.0".to_string(),
            ownership: PolicyOwnership {
                author,
                owner,
                team: None,
                contact: None,
                reviewers: Vec::new(),
            },
            lifecycle: PolicyLifecycle {
                created_at: now,
                updated_at: now,
                effective_from: None,
                effective_until: None,
                deprecation: None,
                revisions: Vec::new(),
            },
            tags: HashSet::new(),
            links: PolicyLinks::default(),
            metrics: PolicyMetrics::default(),
            custom: HashMap::new(),
        }
    }

    /// Activate the policy
    pub fn activate(&mut self) {
        self.status = PolicyStatus::Active;
        self.lifecycle.updated_at = Utc::now();
    }

    /// Disable the policy
    pub fn disable(&mut self) {
        self.status = PolicyStatus::Disabled;
        self.lifecycle.updated_at = Utc::now();
    }

    /// Deprecate the policy
    pub fn deprecate(&mut self, reason: String, replacement_policy_id: Option<String>) {
        self.status = PolicyStatus::Deprecated;
        self.lifecycle.deprecation = Some(DeprecationInfo {
            deprecated_at: Utc::now(),
            reason,
            replacement_policy_id,
            migration_guide: None,
        });
        self.lifecycle.updated_at = Utc::now();
    }

    /// Add a revision to the history
    pub fn add_revision(&mut self, author: String, changes: String) {
        let revision = PolicyRevision {
            version: self.version.clone(),
            timestamp: Utc::now(),
            author,
            changes,
        };
        self.lifecycle.revisions.push(revision);
        self.lifecycle.updated_at = Utc::now();
    }

    /// Add a tag
    pub fn add_tag(&mut self, tag: String) {
        self.tags.insert(tag);
    }

    /// Add multiple tags
    pub fn add_tags(&mut self, tags: Vec<String>) {
        self.tags.extend(tags);
    }

    /// Check if policy has a specific tag
    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.contains(tag)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_severity_ordering() {
        assert!(Severity::Critical > Severity::Error);
        assert!(Severity::Error > Severity::Warning);
        assert!(Severity::Warning > Severity::Info);
    }

    #[test]
    fn test_severity_blocking() {
        assert!(!Severity::Info.is_blocking());
        assert!(!Severity::Warning.is_blocking());
        assert!(Severity::Error.is_blocking());
        assert!(Severity::Critical.is_blocking());
    }

    #[test]
    fn test_severity_score() {
        assert_eq!(Severity::Info.score(), 1);
        assert_eq!(Severity::Warning.score(), 2);
        assert_eq!(Severity::Error.score(), 3);
        assert_eq!(Severity::Critical.score(), 4);
    }

    #[test]
    fn test_policy_status_enforcement() {
        assert!(!PolicyStatus::Draft.is_enforced());
        assert!(PolicyStatus::Active.is_enforced());
        assert!(!PolicyStatus::Disabled.is_enforced());
        assert!(!PolicyStatus::Deprecated.is_enforced());
        assert!(!PolicyStatus::Archived.is_enforced());
    }

    #[test]
    fn test_policy_status_active() {
        assert!(!PolicyStatus::Draft.is_active());
        assert!(PolicyStatus::Active.is_active());
        assert!(PolicyStatus::Disabled.is_active());
        assert!(!PolicyStatus::Deprecated.is_active());
        assert!(!PolicyStatus::Archived.is_active());
    }

    #[test]
    fn test_lifecycle_is_effective_now() {
        let past = Utc::now() - chrono::Duration::hours(1);
        let future = Utc::now() + chrono::Duration::hours(1);

        // No date restrictions
        let lifecycle = PolicyLifecycle {
            created_at: past,
            updated_at: past,
            effective_from: None,
            effective_until: None,
            deprecation: None,
            revisions: Vec::new(),
        };
        assert!(lifecycle.is_effective_now());

        // Effective from past
        let lifecycle = PolicyLifecycle {
            created_at: past,
            updated_at: past,
            effective_from: Some(past),
            effective_until: None,
            deprecation: None,
            revisions: Vec::new(),
        };
        assert!(lifecycle.is_effective_now());

        // Effective from future (not yet effective)
        let lifecycle = PolicyLifecycle {
            created_at: past,
            updated_at: past,
            effective_from: Some(future),
            effective_until: None,
            deprecation: None,
            revisions: Vec::new(),
        };
        assert!(!lifecycle.is_effective_now());

        // Effective until future
        let lifecycle = PolicyLifecycle {
            created_at: past,
            updated_at: past,
            effective_from: None,
            effective_until: Some(future),
            deprecation: None,
            revisions: Vec::new(),
        };
        assert!(lifecycle.is_effective_now());

        // Effective until past (expired)
        let lifecycle = PolicyLifecycle {
            created_at: past,
            updated_at: past,
            effective_from: None,
            effective_until: Some(past),
            deprecation: None,
            revisions: Vec::new(),
        };
        assert!(!lifecycle.is_effective_now());
    }

    #[test]
    fn test_policy_metadata_new() {
        let metadata = PolicyMetadata::new(
            "test-policy".to_string(),
            "Test Policy".to_string(),
            "A test policy".to_string(),
            PolicyCategory::Budget,
            Severity::Warning,
            "alice".to_string(),
            "bob".to_string(),
        );

        assert_eq!(metadata.id, "test-policy");
        assert_eq!(metadata.status, PolicyStatus::Draft);
        assert_eq!(metadata.version, "1.0.0");
        assert_eq!(metadata.ownership.author, "alice");
        assert_eq!(metadata.ownership.owner, "bob");
    }

    #[test]
    fn test_policy_metadata_activate() {
        let mut metadata = PolicyMetadata::new(
            "test".to_string(),
            "Test".to_string(),
            "Test".to_string(),
            PolicyCategory::Budget,
            Severity::Error,
            "alice".to_string(),
            "alice".to_string(),
        );

        assert_eq!(metadata.status, PolicyStatus::Draft);
        metadata.activate();
        assert_eq!(metadata.status, PolicyStatus::Active);
    }

    #[test]
    fn test_policy_metadata_deprecate() {
        let mut metadata = PolicyMetadata::new(
            "old-policy".to_string(),
            "Old Policy".to_string(),
            "Deprecated".to_string(),
            PolicyCategory::Budget,
            Severity::Warning,
            "alice".to_string(),
            "alice".to_string(),
        );

        metadata.deprecate(
            "Replaced by new-policy".to_string(),
            Some("new-policy".to_string()),
        );

        assert_eq!(metadata.status, PolicyStatus::Deprecated);
        assert!(metadata.lifecycle.deprecation.is_some());

        let deprecation = metadata.lifecycle.deprecation.unwrap();
        assert_eq!(deprecation.reason, "Replaced by new-policy");
        assert_eq!(
            deprecation.replacement_policy_id,
            Some("new-policy".to_string())
        );
    }

    #[test]
    fn test_policy_metadata_tags() {
        let mut metadata = PolicyMetadata::new(
            "test".to_string(),
            "Test".to_string(),
            "Test".to_string(),
            PolicyCategory::Budget,
            Severity::Info,
            "alice".to_string(),
            "alice".to_string(),
        );

        metadata.add_tag("production".to_string());
        metadata.add_tag("cost-optimization".to_string());

        assert!(metadata.has_tag("production"));
        assert!(metadata.has_tag("cost-optimization"));
        assert!(!metadata.has_tag("development"));
    }

    #[test]
    fn test_policy_metrics_record() {
        let mut metrics = PolicyMetrics::default();

        // Record evaluations
        metrics.record_evaluation(false);
        assert_eq!(metrics.evaluation_count, 1);
        assert_eq!(metrics.violation_count, 0);
        assert!(metrics.last_evaluated.is_some());
        assert!(metrics.last_violation.is_none());

        metrics.record_evaluation(true);
        assert_eq!(metrics.evaluation_count, 2);
        assert_eq!(metrics.violation_count, 1);
        assert!(metrics.last_violation.is_some());

        // Check violation rate
        assert_eq!(metrics.violation_rate, Some(0.5));

        // Record exemption
        metrics.record_exemption();
        assert_eq!(metrics.exemption_count, 1);
    }

    #[test]
    fn test_policy_with_metadata_should_enforce() {
        let mut metadata = PolicyMetadata::new(
            "test".to_string(),
            "Test".to_string(),
            "Test".to_string(),
            PolicyCategory::Budget,
            Severity::Error,
            "alice".to_string(),
            "alice".to_string(),
        );

        let policy = PolicyWithMetadata::new(metadata.clone(), ());

        // Draft status - should not enforce
        assert!(!policy.should_enforce());

        // Active status - should enforce
        metadata.activate();
        let policy = PolicyWithMetadata::new(metadata.clone(), ());
        assert!(policy.should_enforce());

        // Disabled status - should not enforce
        metadata.disable();
        let policy = PolicyWithMetadata::new(metadata, ());
        assert!(!policy.should_enforce());
    }

    #[test]
    fn test_policy_with_metadata_blocking() {
        let metadata = PolicyMetadata::new(
            "test".to_string(),
            "Test".to_string(),
            "Test".to_string(),
            PolicyCategory::Budget,
            Severity::Warning,
            "alice".to_string(),
            "alice".to_string(),
        );

        let policy = PolicyWithMetadata::new(metadata, ());
        assert!(!policy.is_blocking());

        let metadata = PolicyMetadata::new(
            "test".to_string(),
            "Test".to_string(),
            "Test".to_string(),
            PolicyCategory::Budget,
            Severity::Critical,
            "alice".to_string(),
            "alice".to_string(),
        );

        let policy = PolicyWithMetadata::new(metadata, ());
        assert!(policy.is_blocking());
    }
}
