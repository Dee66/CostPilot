use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Product specification configuration loaded from product.yml
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProductSpec {
    pub metadata: Metadata,
    pub platform: Platform,
    pub x_capabilities: XCapabilities,
    pub zero_cost_policy: ZeroCostPolicy,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Metadata {
    pub product_id: String,
    pub product_name: String,
    pub spec_version: String,
    pub spec_state: String,
    pub version: String,
    pub version_strategy: String,
    pub owner: String,
    pub created_at: String,
    pub launch_tier: String,
    pub source_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Platform {
    pub runtime: String,
    pub orchestrator: String,
    pub deployable_unit: String,
    pub cost_mode: String,
    pub core_components: HashMap<String, Component>,
    pub architecture_invariants: Vec<String>,
    pub capabilities: PlatformCapabilities,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Component {
    pub language: Option<String>,
    pub framework: Option<String>,
    pub responsibilities: Vec<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_yaml::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PlatformCapabilities {
    pub scanning: bool,
    pub patching: bool,
    pub ai_generation: bool,
    pub metrics_output: bool,
    pub wasm_safe: bool,
    pub deterministic: bool,
    pub offline_mode: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct XCapabilities {
    pub detect: DetectCapability,
    pub predict: PredictCapability,
    pub explain: ExplainCapability,
    pub autofix: AutofixCapability,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DetectCapability {
    pub description: String,
    pub categories: Vec<String>,
    pub detection_rules: Vec<String>,
    pub outputs: Vec<String>,
    pub regression_classifier: RegressionClassifier,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RegressionClassifier {
    pub types: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PredictCapability {
    pub description: String,
    pub heuristics_source: String,
    pub cold_start_inference: ColdStartInference,
    pub prediction_intervals: PredictionIntervals,
    pub outputs: Vec<String>,
    pub invariants: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ColdStartInference {
    pub enabled: bool,
    pub defaults: ColdStartDefaults,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ColdStartDefaults {
    pub dynamodb_rcu: u32,
    pub dynamodb_wcu: u32,
    pub lambda_requests: u64,
    pub s3_monthly_gb: u32,
    pub ec2_utilization_pct: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PredictionIntervals {
    pub enabled: bool,
    pub range_factor: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExplainCapability {
    pub description: String,
    pub modes: HashMap<String, String>,
    pub outputs: Vec<String>,
    pub invariants: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AutofixCapability {
    pub description: String,
    pub modes: HashMap<String, AutofixMode>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AutofixMode {
    pub description: String,
    pub availability: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ZeroCostPolicy {
    pub allowed_modes: Vec<String>,
    pub default_mode: String,
    pub invariants: Vec<String>,
    pub forbidden_actions: Vec<String>,
    pub simulation_rules: SimulationRules,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SimulationRules {
    pub terraform: TerraformSimulation,
    pub cloud_sdks: CloudSdksSimulation,
    pub network: NetworkSimulation,
    pub deployment: DeploymentSimulation,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TerraformSimulation {
    pub simulate_plan: bool,
    pub allow_apply: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CloudSdksSimulation {
    pub simulate: bool,
    pub forbid_real_calls: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NetworkSimulation {
    pub outbound_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DeploymentSimulation {
    pub dry_run_only: bool,
}

/// Load product specification from the default path
pub fn load_product_spec() -> Result<ProductSpec, ConfigError> {
    let path = Path::new("products/costpilot/product.yml");
    load_product_spec_from_path(path)
}

/// Load product specification from a custom path
pub fn load_product_spec_from_path<P: AsRef<Path>>(path: P) -> Result<ProductSpec, ConfigError> {
    let content = fs::read_to_string(path).map_err(ConfigError::Io)?;
    let spec: ProductSpec = serde_yaml::from_str(&content).map_err(ConfigError::Yaml)?;
    Ok(spec)
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("YAML parsing error: {0}")]
    Yaml(#[from] serde_yaml::Error),
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;

    #[test]
    fn test_spec_loader_roundtrip() {
        // Load the actual spec
        let spec = load_product_spec().expect("Failed to load product spec");

        // Verify key fields are loaded correctly
        assert_eq!(spec.metadata.product_id, "costpilot");
        assert_eq!(spec.metadata.product_name, "CostPilot");
        assert_eq!(spec.metadata.version, "1.0.2");
        assert_eq!(spec.platform.runtime, "wasm_sandbox");
        assert_eq!(spec.zero_cost_policy.default_mode, "safe");
        assert!(spec.x_capabilities.predict.cold_start_inference.enabled);
    }

    #[test]
    fn test_zero_cost_policy_loaded() {
        let spec = load_product_spec().expect("Failed to load product spec");

        // Verify zero cost policy fields
        assert!(spec.zero_cost_policy.allowed_modes.contains(&"safe".to_string()));
        assert!(spec.zero_cost_policy.allowed_modes.contains(&"simulate".to_string()));
        assert_eq!(spec.zero_cost_policy.default_mode, "safe");

        // Verify invariants
        assert!(spec.zero_cost_policy.invariants.len() > 0);
        assert!(spec.zero_cost_policy.invariants.iter().any(|inv| inv.contains("No IAM permissions")));

        // Verify forbidden actions
        assert!(spec.zero_cost_policy.forbidden_actions.contains(&"terraform_apply".to_string()));
        assert!(spec.zero_cost_policy.forbidden_actions.contains(&"cloud_resource_create".to_string()));

        // Verify simulation rules
        assert!(!spec.zero_cost_policy.simulation_rules.terraform.allow_apply);
        assert!(spec.zero_cost_policy.simulation_rules.terraform.simulate_plan);
        assert!(!spec.zero_cost_policy.simulation_rules.network.outbound_enabled);
    }

    #[test]
    fn test_invalid_yaml_returns_error() {
        let temp_file = NamedTempFile::new().unwrap();
        fs::write(&temp_file, "invalid: yaml: content: [unclosed").unwrap();

        let result = load_product_spec_from_path(temp_file.path());
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ConfigError::Yaml(_)));
    }

    #[test]
    fn test_missing_file_returns_error() {
        let result = load_product_spec_from_path("nonexistent.yml");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ConfigError::Io(_)));
    }
}
