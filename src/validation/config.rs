// Config validator for costpilot.yaml
//
// Validates the main CostPilot configuration file against the expected schema.

use crate::validation::error::{ValidationError, ValidationResult, ValidationWarning};
use crate::validation::{ValidationReport, FileType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Main configuration schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostPilotConfig {
    #[serde(default)]
    pub version: Option<String>,
    
    #[serde(default)]
    pub default_region: Option<String>,
    
    #[serde(default)]
    pub scan: Option<ScanConfig>,
    
    #[serde(default)]
    pub policies: Option<PoliciesConfig>,
    
    #[serde(default)]
    pub output: Option<OutputConfig>,
    
    #[serde(default)]
    pub heuristics: Option<HeuristicsConfig>,
    
    #[serde(default)]
    pub slo: Option<SloConfig>,
    
    #[serde(default)]
    pub integrations: Option<IntegrationsConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanConfig {
    #[serde(default)]
    pub fail_on_critical: Option<bool>,
    
    #[serde(default)]
    pub show_autofix: Option<bool>,
    
    #[serde(default)]
    pub explain: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoliciesConfig {
    #[serde(default)]
    pub default: Option<String>,
    
    #[serde(default)]
    pub exemptions: Option<String>,
    
    #[serde(default)]
    pub directory: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    #[serde(default)]
    pub format: Option<String>,
    
    #[serde(default)]
    pub verbose: Option<bool>,
    
    #[serde(default)]
    pub color: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeuristicsConfig {
    #[serde(default)]
    pub auto_update: Option<bool>,
    
    #[serde(default)]
    pub cache_ttl: Option<String>,
    
    #[serde(default)]
    pub file: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SloConfig {
    #[serde(default)]
    pub config: Option<String>,
    
    #[serde(default)]
    pub snapshots_dir: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationsConfig {
    #[serde(default)]
    pub github: Option<GithubIntegration>,
    
    #[serde(default)]
    pub slack: Option<SlackIntegration>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GithubIntegration {
    pub enabled: bool,
    #[serde(default)]
    pub comment_on_pr: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlackIntegration {
    pub enabled: bool,
    #[serde(default)]
    pub webhook_url: Option<String>,
}

pub struct ConfigValidator;

impl ConfigValidator {
    /// Validate a configuration file
    pub fn validate_file(path: impl AsRef<Path>) -> ValidationResult<ValidationReport> {
        let path = path.as_ref();
        let mut report = ValidationReport::new(path, FileType::Config);

        // Read file
        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) => {
                report.add_error(
                    ValidationError::new(format!("Failed to read file: {}", e))
                        .with_error_code("E100")
                        .with_hint("Ensure the file exists and is readable")
                );
                return Ok(report);
            }
        };

        // Parse YAML
        let config: CostPilotConfig = match serde_yaml::from_str(&content) {
            Ok(c) => c,
            Err(e) => {
                report.add_error(ValidationError::from(e));
                return Ok(report);
            }
        };

        // Validate configuration
        Self::validate_config(&config, &mut report);

        Ok(report)
    }

    /// Validate configuration structure and values
    fn validate_config(config: &CostPilotConfig, report: &mut ValidationReport) {
        // Version check
        if let Some(version) = &config.version {
            if !Self::is_valid_semver(version) {
                report.add_error(
                    ValidationError::new(format!("Invalid version format: {}", version))
                        .with_field("version")
                        .with_error_code("E101")
                        .with_hint("Use semantic versioning (e.g., '1.0.0')")
                );
            }
        } else {
            report.add_warning(
                ValidationWarning::new("Version field is missing")
                    .with_field("version")
                    .with_warning_code("W100")
                    .with_suggestion("Add 'version: \"1.0.0\"' to track config version")
            );
        }

        // Default region check
        if let Some(region) = &config.default_region {
            if !Self::is_valid_aws_region(region) {
                report.add_warning(
                    ValidationWarning::new(format!("Unknown AWS region: {}", region))
                        .with_field("default_region")
                        .with_warning_code("W101")
                        .with_suggestion("Use standard AWS region codes (e.g., 'us-east-1', 'eu-west-1')")
                );
            }
        }

        // Output format validation
        if let Some(output) = &config.output {
            if let Some(format) = &output.format {
                if !matches!(format.as_str(), "json" | "text" | "markdown") {
                    report.add_error(
                        ValidationError::new(format!("Invalid output format: {}", format))
                            .with_field("output.format")
                            .with_error_code("E102")
                            .with_hint("Supported formats: 'json', 'text', 'markdown'")
                    );
                }
            }
        }

        // Heuristics cache TTL validation
        if let Some(heuristics) = &config.heuristics {
            if let Some(cache_ttl) = &heuristics.cache_ttl {
                if !Self::is_valid_duration(cache_ttl) {
                    report.add_error(
                        ValidationError::new(format!("Invalid cache TTL format: {}", cache_ttl))
                            .with_field("heuristics.cache_ttl")
                            .with_error_code("E103")
                            .with_hint("Use duration format: '24h', '30m', '1d'")
                    );
                }
            }
        }

        // Policy file paths validation
        if let Some(policies) = &config.policies {
            if let Some(default_policy) = &policies.default {
                if !default_policy.ends_with(".yaml") && !default_policy.ends_with(".yml") {
                    report.add_warning(
                        ValidationWarning::new("Default policy should be a YAML file")
                            .with_field("policies.default")
                            .with_warning_code("W102")
                            .with_suggestion("Use .yaml or .yml extension")
                    );
                }
            }
        }

        // Slack webhook validation
        if let Some(integrations) = &config.integrations {
            if let Some(slack) = &integrations.slack {
                if slack.enabled {
                    if let Some(webhook) = &slack.webhook_url {
                        if !webhook.starts_with("https://hooks.slack.com/") {
                            report.add_error(
                                ValidationError::new("Invalid Slack webhook URL")
                                    .with_field("integrations.slack.webhook_url")
                                    .with_error_code("E104")
                                    .with_hint("Webhook URL must start with 'https://hooks.slack.com/'")
                            );
                        }
                    } else {
                        report.add_error(
                            ValidationError::new("Slack integration enabled but webhook_url is missing")
                                .with_field("integrations.slack.webhook_url")
                                .with_error_code("E105")
                                .with_hint("Add 'webhook_url' field or disable Slack integration")
                        );
                    }
                }
            }
        }
    }

    fn is_valid_semver(version: &str) -> bool {
        let parts: Vec<&str> = version.split('.').collect();
        if parts.len() != 3 {
            return false;
        }
        parts.iter().all(|p| p.parse::<u32>().is_ok())
    }

    fn is_valid_aws_region(region: &str) -> bool {
        const VALID_REGIONS: &[&str] = &[
            "us-east-1", "us-east-2", "us-west-1", "us-west-2",
            "eu-west-1", "eu-west-2", "eu-west-3", "eu-central-1", "eu-north-1",
            "ap-south-1", "ap-northeast-1", "ap-northeast-2", "ap-northeast-3",
            "ap-southeast-1", "ap-southeast-2", "ap-east-1",
            "ca-central-1", "sa-east-1", "af-south-1", "me-south-1",
        ];
        VALID_REGIONS.contains(&region)
    }

    fn is_valid_duration(duration: &str) -> bool {
        let re = regex::Regex::new(r"^\d+[smhd]$").unwrap();
        re.is_match(duration)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_semver() {
        assert!(ConfigValidator::is_valid_semver("1.0.0"));
        assert!(ConfigValidator::is_valid_semver("0.1.0"));
        assert!(!ConfigValidator::is_valid_semver("1.0"));
        assert!(!ConfigValidator::is_valid_semver("v1.0.0"));
    }

    #[test]
    fn test_valid_aws_region() {
        assert!(ConfigValidator::is_valid_aws_region("us-east-1"));
        assert!(ConfigValidator::is_valid_aws_region("eu-west-1"));
        assert!(!ConfigValidator::is_valid_aws_region("invalid-region"));
    }

    #[test]
    fn test_valid_duration() {
        assert!(ConfigValidator::is_valid_duration("24h"));
        assert!(ConfigValidator::is_valid_duration("30m"));
        assert!(ConfigValidator::is_valid_duration("1d"));
        assert!(!ConfigValidator::is_valid_duration("24hours"));
        assert!(!ConfigValidator::is_valid_duration("invalid"));
    }

    #[test]
    fn test_parse_valid_config() {
        let yaml = r#"
version: "1.0.0"
default_region: us-east-1
scan:
  fail_on_critical: true
  show_autofix: true
output:
  format: json
  verbose: false
"#;
        let config: CostPilotConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.version, Some("1.0.0".to_string()));
        assert_eq!(config.default_region, Some("us-east-1".to_string()));
    }
}
