use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Policy configuration loaded from .costpilot/policy.yml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyConfig {
    pub version: String,
    #[serde(default)]
    pub budgets: BudgetPolicies,
    #[serde(default)]
    pub resources: ResourcePolicies,
    #[serde(default)]
    pub slos: Vec<SloPolicy>,
    #[serde(default)]
    pub enforcement: EnforcementConfig,
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetLimit {
    pub monthly_limit: f64,
    #[serde(default = "default_warning_threshold")]
    pub warning_threshold: f64,
}

fn default_warning_threshold() -> f64 {
    0.8
}

/// Per-module budget
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleBudget {
    pub name: String,
    pub monthly_limit: f64,
}

/// Resource-specific policies
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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
}

/// NAT Gateway policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatGatewayPolicy {
    pub max_count: usize,
    #[serde(default)]
    pub require_justification: bool,
}

/// EC2 instance policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ec2Policy {
    #[serde(default)]
    pub allowed_families: Vec<String>,
    #[serde(default)]
    pub max_size: Option<String>,
}

/// S3 bucket policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S3Policy {
    #[serde(default)]
    pub require_lifecycle_rules: bool,
    #[serde(default)]
    pub require_encryption: bool,
}

/// Lambda function policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LambdaPolicy {
    #[serde(default)]
    pub require_concurrency_limit: bool,
    #[serde(default)]
    pub max_memory_mb: Option<u32>,
}

/// DynamoDB table policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamoDbPolicy {
    #[serde(default)]
    pub prefer_provisioned: bool,
}

/// SLO policy definition
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub passed: bool,
}

impl PolicyResult {
    pub fn new() -> Self {
        Self {
            violations: Vec::new(),
            warnings: Vec::new(),
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
}
