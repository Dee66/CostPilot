use super::policy_types::*;
use crate::errors::{CostPilotError, ErrorCategory};
use std::fs;
use std::path::Path;

/// Policy loader for reading and parsing policy files
pub struct PolicyLoader;

impl PolicyLoader {
    /// Load policy configuration from file
    pub fn load_from_file(path: &Path) -> Result<PolicyConfig, CostPilotError> {
        // Check if file exists
        if !path.exists() {
            return Err(CostPilotError::new(
                "POLICY_001",
                ErrorCategory::FileSystemError,
                format!("Policy file not found: {}", path.display()),
            )
            .with_hint("Run 'costpilot init' to generate a sample policy file".to_string()));
        }

        // Read file content
        let content = fs::read_to_string(path).map_err(|e| {
            CostPilotError::new(
                "POLICY_002",
                ErrorCategory::FileSystemError,
                format!("Failed to read policy file: {}", e),
            )
        })?;

        // Parse YAML
        Self::parse_yaml(&content)
    }

    /// Parse policy configuration from YAML string
    pub fn parse_yaml(yaml_content: &str) -> Result<PolicyConfig, CostPilotError> {
        serde_yaml::from_str(yaml_content).map_err(|e| {
            CostPilotError::new(
                "POLICY_003",
                ErrorCategory::ValidationError,
                format!("Failed to parse policy YAML: {}", e),
            )
            .with_hint("Check that the policy file is valid YAML and follows the expected schema".to_string())
        })
    }

    /// Validate policy configuration
    pub fn validate(config: &PolicyConfig) -> Result<(), CostPilotError> {
        // Validate version format
        if config.version.is_empty() {
            return Err(CostPilotError::new(
                "POLICY_004",
                ErrorCategory::ValidationError,
                "Policy version is required".to_string(),
            ));
        }

        // Validate budget limits are positive
        if let Some(global) = &config.budgets.global {
            if global.monthly_limit <= 0.0 {
                return Err(CostPilotError::new(
                    "POLICY_005",
                    ErrorCategory::ValidationError,
                    "Global monthly limit must be positive".to_string(),
                ));
            }
            if global.warning_threshold <= 0.0 || global.warning_threshold > 1.0 {
                return Err(CostPilotError::new(
                    "POLICY_006",
                    ErrorCategory::ValidationError,
                    "Warning threshold must be between 0 and 1".to_string(),
                ));
            }
        }

        // Validate module budgets
        for module in &config.budgets.modules {
            if module.monthly_limit <= 0.0 {
                return Err(CostPilotError::new(
                    "POLICY_007",
                    ErrorCategory::ValidationError,
                    format!("Module '{}' monthly limit must be positive", module.name),
                ));
            }
        }

        // Validate NAT gateway policy
        if let Some(nat) = &config.resources.nat_gateways {
            if nat.max_count == 0 {
                return Err(CostPilotError::new(
                    "POLICY_008",
                    ErrorCategory::ValidationError,
                    "NAT gateway max_count must be at least 1".to_string(),
                ));
            }
        }

        // Validate enforcement mode
        match config.enforcement.mode.as_str() {
            "advisory" | "blocking" => {}
            _ => {
                return Err(CostPilotError::new(
                    "POLICY_009",
                    ErrorCategory::ValidationError,
                    format!(
                        "Invalid enforcement mode '{}', must be 'advisory' or 'blocking'",
                        config.enforcement.mode
                    ),
                ));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic_policy() {
        let yaml = r#"
version: 1.0.0
budgets:
  global:
    monthly_limit: 1000
    warning_threshold: 0.8
enforcement:
  mode: advisory
"#;

        let config = PolicyLoader::parse_yaml(yaml).unwrap();
        assert_eq!(config.version, "1.0.0");
        assert!(config.budgets.global.is_some());
        assert_eq!(config.budgets.global.unwrap().monthly_limit, 1000.0);
    }

    #[test]
    fn test_validate_invalid_budget() {
        let yaml = r#"
version: 1.0.0
budgets:
  global:
    monthly_limit: -100
"#;

        let config = PolicyLoader::parse_yaml(yaml).unwrap();
        let result = PolicyLoader::validate(&config);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_invalid_enforcement_mode() {
        let yaml = r#"
version: 1.0.0
enforcement:
  mode: invalid_mode
"#;

        let config = PolicyLoader::parse_yaml(yaml).unwrap();
        let result = PolicyLoader::validate(&config);
        assert!(result.is_err());
    }
}
