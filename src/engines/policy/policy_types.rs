use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Policy configuration loaded from .costpilot/policy.yml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyConfig {
    pub version: String,
    #[serde(default)]
    pub metadata: PolicyMetadata,
    #[serde(default)]
    pub budgets: BudgetPolicies,
    #[serde(default)]
    pub resources: ResourcePolicies,
    #[serde(default)]
    pub slos: Vec<SloPolicy>,
    #[serde(default)]
    pub enforcement: EnforcementConfig,
}

/// Policy metadata for versioning, approval tracking, and ownership
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PolicyMetadata {
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
    #[serde(default)]
    pub created_by: Option<String>,
    #[serde(default)]
    pub updated_by: Option<String>,
    #[serde(default)]
    pub approval_required: bool,
    #[serde(default)]
    pub approved_by: Option<String>,
    #[serde(default)]
    pub approved_at: Option<String>,
    #[serde(default)]
    pub approval_reference: Option<String>,
    #[serde(default)]
    pub owners: Vec<String>,
    #[serde(default)]
    pub reviewers: Vec<String>,
    #[serde(default)]
    pub tags: HashMap<String, String>,
    #[serde(default)]
    pub description: Option<String>,
}

/// Budget policies for global and per-module limits
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BudgetPolicies {
    #[serde(default)]
    pub global: Option<BudgetLimit>,
    #[serde(default)]
    pub modules: Vec<ModuleBudget>,
}

/// Budget limit with monthly cap and warning threshold
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BudgetLimit {
    pub monthly_limit: f64,
    #[serde(default = "default_warning_threshold")]
    pub warning_threshold: f64,
}

fn default_warning_threshold() -> f64 {
    0.8
}

/// Per-module budget
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModuleBudget {
    pub name: String,
    pub monthly_limit: f64,
}

/// Resource-specific policies
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct ResourcePolicies {
    #[serde(default)]
    pub nat_gateways: Option<NatGatewayPolicy>,
    #[serde(default)]
    pub ec2_instances: Option<Ec2Policy>,
    #[serde(default)]
    pub s3_buckets: Option<S3Policy>,
    #[serde(default)]
    pub lambda_functions: Option<LambdaPolicy>,
    #[serde(default)]
    pub dynamodb_tables: Option<DynamoDbPolicy>,
    #[serde(default)]
    pub s3_lifecycle_required: bool,
}

/// NAT Gateway policy
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NatGatewayPolicy {
    pub max_count: usize,
    #[serde(default)]
    pub require_justification: bool,
}

/// EC2 instance policy
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Ec2Policy {
    #[serde(default)]
    pub allowed_families: Vec<String>,
    #[serde(default)]
    pub max_size: Option<String>,
}

/// S3 bucket policy
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct S3Policy {
    #[serde(default)]
    pub require_lifecycle_rules: bool,
    #[serde(default)]
    pub require_encryption: bool,
}

/// Lambda function policy
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LambdaPolicy {
    #[serde(default)]
    pub require_concurrency_limit: bool,
    #[serde(default)]
    pub max_memory_mb: Option<u32>,
}

/// DynamoDB table policy
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DynamoDbPolicy {
    #[serde(default)]
    pub prefer_provisioned: bool,
}

/// SLO policy definition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SloPolicy {
    pub name: String,
    #[serde(rename = "type")]
    pub policy_type: String,
    #[serde(default)]
    pub value: Option<f64>,
    #[serde(default)]
    pub patterns: Vec<String>,
    pub severity: String,
}

/// Enforcement configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EnforcementConfig {
    #[serde(default = "default_enforcement_mode")]
    pub mode: String,
    #[serde(default)]
    pub fail_on_violation: bool,
}

fn default_enforcement_mode() -> String {
    "advisory".to_string()
}

impl Default for EnforcementConfig {
    fn default() -> Self {
        Self {
            mode: "advisory".to_string(),
            fail_on_violation: false,
        }
    }
}

/// Policy violation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyViolation {
    pub policy_name: String,
    pub severity: String,
    pub resource_id: String,
    pub message: String,
    pub actual_value: String,
    pub expected_value: String,
}

/// Policy evaluation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyResult {
    pub violations: Vec<PolicyViolation>,
    pub warnings: Vec<String>,
    pub applied_exemptions: Vec<String>,
    pub passed: bool,
}

impl PolicyConfig {
    /// Create a new policy configuration with default metadata
    pub fn new() -> Self {
        Self {
            version: "1.0.0".to_string(),
            metadata: PolicyMetadata {
                created_at: Some(chrono::Utc::now().to_rfc3339()),
                created_by: None,
                approval_required: false,
                owners: Vec::new(),
                reviewers: Vec::new(),
                tags: HashMap::new(),
                ..Default::default()
            },
            budgets: Default::default(),
            resources: Default::default(),
            slos: Vec::new(),
            enforcement: Default::default(),
        }
    }

    /// Increment version when policy content changes
    pub fn increment_version(&mut self, user: Option<String>) {
        let current_version = semver::Version::parse(&self.version).unwrap_or_else(|_| {
            // If version is invalid, start from 1.0.0
            semver::Version::new(1, 0, 0)
        });
        let new_version = semver::Version::new(current_version.major, current_version.minor, current_version.patch + 1);

        self.version = new_version.to_string();
        self.metadata.updated_at = Some(chrono::Utc::now().to_rfc3339());
        self.metadata.updated_by = user;
    }

    /// Mark policy as approved
    pub fn approve(&mut self, approver: String, reference: Option<String>) {
        self.metadata.approved_by = Some(approver);
        self.metadata.approved_at = Some(chrono::Utc::now().to_rfc3339());
        self.metadata.approval_reference = reference;
    }

    /// Check if policy requires approval
    pub fn requires_approval(&self) -> bool {
        self.metadata.approval_required
    }

    /// Check if policy is approved
    pub fn is_approved(&self) -> bool {
        self.metadata.approved_by.is_some() && self.metadata.approved_at.is_some()
    }

    /// Add an owner to the policy
    pub fn add_owner(&mut self, owner: String) {
        if !self.metadata.owners.contains(&owner) {
            self.metadata.owners.push(owner);
        }
    }

    /// Add a reviewer to the policy
    pub fn add_reviewer(&mut self, reviewer: String) {
        if !self.metadata.reviewers.contains(&reviewer) {
            self.metadata.reviewers.push(reviewer);
        }
    }

    /// Set policy description
    pub fn set_description(&mut self, description: String) {
        self.metadata.description = Some(description);
    }

    /// Initialize metadata when loading from file (for backward compatibility)
    pub fn initialize_metadata(&mut self, user: Option<String>) {
        if self.metadata.created_at.is_none() {
            self.metadata.created_at = Some(chrono::Utc::now().to_rfc3339());
            self.metadata.created_by = user.clone();
        }
        if self.metadata.updated_at.is_none() {
            self.metadata.updated_at = Some(chrono::Utc::now().to_rfc3339());
            self.metadata.updated_by = user;
        }
    }
}


impl Default for PolicyConfig {
    fn default() -> Self {
        Self::new()
    }
}
impl PolicyResult {
    pub fn new() -> Self {
        Self {
            violations: Vec::new(),
            warnings: Vec::new(),
            applied_exemptions: Vec::new(),
            passed: true,
        }
    }

    pub fn add_violation(&mut self, violation: PolicyViolation) {
        self.passed = false;
        self.violations.push(violation);
    }

    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }

    pub fn add_applied_exemption(&mut self, exemption_id: String) {
        self.applied_exemptions.push(exemption_id);
    }

}
impl Default for PolicyResult {
    fn default() -> Self {
        Self::new()
    }
}
