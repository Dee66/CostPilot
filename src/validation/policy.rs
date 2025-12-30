// Policy validator
//
// Validates policy YAML files against the expected schema.

use crate::engines::policy::parser::PolicyRule;
use crate::engines::policy::PolicyExemption;
use crate::validation::error::{ValidationError, ValidationResult, ValidationWarning};
use crate::validation::{FileType, ValidationReport};
// use crate::engines::policy::Exemption; // TODO: Define Exemption type
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Policy configuration with rules and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Policy {
    // Use a flexible value for metadata so validation accepts a wide range
    // of metadata shapes used in different test fixtures and sample files.
    #[serde(default)]
    pub metadata: Option<serde_yaml::Value>,
    #[serde(default)]
    pub rules: Vec<PolicyRule>,
    #[serde(default)]
    pub exemptions: Vec<PolicyExemption>,
}

pub struct PolicyValidator;

impl PolicyValidator {
    /// Validate a policy file
    pub fn validate_file(path: impl AsRef<Path>) -> ValidationResult<ValidationReport> {
        let path = path.as_ref();
        let mut report = ValidationReport::new(path, FileType::Policy);

        // Read file
        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) => {
                report.add_error(
                    ValidationError::new(format!("Failed to read file: {}", e))
                        .with_error_code("E200")
                        .with_hint("Ensure the file exists and is readable"),
                );
                return Ok(report);
            }
        };

        // Parse YAML
        let policy: Policy = match serde_yaml::from_str(&content) {
            Ok(p) => p,
            Err(e) => {
                report.add_error(ValidationError::from(e));
                return Ok(report);
            }
        };

        // Validate policy
        Self::validate_policy(&policy, &mut report);

        Ok(report)
    }

    fn validate_policy(policy: &Policy, report: &mut ValidationReport) {
        // Validate metadata presence
        if policy.metadata.is_none() {
            report.add_warning(
                ValidationWarning::new("Policy metadata is missing")
                    .with_field("metadata")
                    .with_warning_code("W200")
                    .with_suggestion(
                        "Add metadata for better tracking (id, name, description, etc.)",
                    ),
            );
        }

        // Validate rules exist. Accept policies that declare budgets but no rules
        if policy.rules.is_empty() {
            let mut has_budgets = false;
            if let Some(serde_yaml::Value::Mapping(map)) = &policy.metadata {
                let key = serde_yaml::Value::String("budgets".to_string());
                if map.get(&key).is_some() {
                    has_budgets = true;
                }
            }

            if !has_budgets {
                report.add_error(
                    ValidationError::new("Policy has no rules defined")
                        .with_field("rules")
                        .with_error_code("E201")
                        .with_hint("Add at least one rule to the policy"),
                );
            }
        }

        // Validate each rule
        for (idx, rule) in policy.rules.iter().enumerate() {
            let rule_prefix = format!("rules[{}]", idx);

            // Rule name
            if rule.name.is_empty() {
                report.add_error(
                    ValidationError::new("Rule has empty name")
                        .with_field(format!("{}.name", rule_prefix))
                        .with_error_code("E202")
                        .with_hint("Provide a descriptive name for the rule"),
                );
            }

            // Rule conditions
            if rule.conditions.is_empty() {
                report.add_warning(
                    ValidationWarning::new("Rule has no conditions")
                        .with_field(format!("{}.conditions", rule_prefix))
                        .with_warning_code("W204")
                        .with_suggestion("Add at least one condition to the rule or explicitly document that it should always match"),
                );
            }

            // Validate conditions
            for (cond_idx, condition) in rule.conditions.iter().enumerate() {
                let cond_prefix = format!("{}.conditions[{}]", rule_prefix, cond_idx);

                // Check for valid operators
                use crate::engines::policy::parser::{ConditionValue, Operator};
                match condition.operator {
                    Operator::GreaterThan
                    | Operator::LessThan
                    | Operator::GreaterThanOrEqual
                    | Operator::LessThanOrEqual => {
                        // Value should be numeric
                        if !matches!(condition.value, ConditionValue::Number(_)) {
                            report.add_error(
                                ValidationError::new(format!(
                                    "Condition with {:?} operator requires numeric value",
                                    condition.operator
                                ))
                                .with_field(format!("{}.value", cond_prefix))
                                .with_error_code("E204")
                                .with_hint("Use a numeric value for comparison operators"),
                            );
                        }
                    }
                    Operator::Matches => {
                        // Value should be valid regex
                        if let ConditionValue::String(pattern) = &condition.value {
                            if regex::Regex::new(pattern).is_err() {
                                report.add_error(
                                    ValidationError::new(format!(
                                        "Invalid regex pattern: {}",
                                        pattern
                                    ))
                                    .with_field(format!("{}.value", cond_prefix))
                                    .with_error_code("E205")
                                    .with_hint("Provide a valid regular expression"),
                                );
                            }
                        }
                    }
                    _ => {}
                }
            }

            // Rule is enabled but might want to warn about disabled rules
            if !rule.enabled {
                report.add_warning(
                    ValidationWarning::new("Rule is disabled")
                        .with_field(format!("{}.enabled", rule_prefix))
                        .with_warning_code("W201")
                        .with_suggestion("Enable the rule or remove it from the policy file"),
                );
            }
        }

        // Validate exemptions
        for (idx, exemption) in policy.exemptions.iter().enumerate() {
            let exemption_prefix = format!("exemptions[{}]", idx);

            if exemption.resource_pattern.is_empty() {
                report.add_error(
                    ValidationError::new("Exemption has empty resource_pattern")
                        .with_field(format!("{}.resource_pattern", exemption_prefix))
                        .with_error_code("E206")
                        .with_hint("Specify which resource pattern to exempt"),
                );
            }

            if exemption.justification.is_empty() {
                report.add_warning(
                    ValidationWarning::new("Exemption has no justification")
                        .with_field(format!("{}.justification", exemption_prefix))
                        .with_warning_code("W203")
                        .with_suggestion("Provide justification for the exemption"),
                );
            }

            // Validate expiry date
            if let Ok(expiry) = chrono::DateTime::parse_from_rfc3339(&exemption.expires_at) {
                let now = chrono::Utc::now();
                if expiry < now {
                    report.add_error(
                        ValidationError::new("Exemption has already expired")
                            .with_field(format!("{}.expires_at", exemption_prefix))
                            .with_error_code("E207")
                            .with_hint("Remove expired exemptions or update the expiry date"),
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_validate_valid_policy() {
        let yaml = r#"
metadata:
  id: test-policy
  name: Test Policy
  description: Policy for testing
  category: budget
  severity: critical
  status: active
  version: 1.0.0
  ownership:
    author: john.doe
    owner: team-platform
    approvers: []
  lifecycle:
    created_at: "2024-01-01T00:00:00Z"
    updated_at: "2024-01-01T00:00:00Z"

rules:
  - name: budget_limit
    description: Limit monthly cost
    enabled: true
    severity: High
    conditions:
      - condition_type:
          type: monthly_cost
        operator: greater_than
        value: 1000
    action:
      type: block
      message: "Monthly cost exceeds budget"

exemptions: []
"#;
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(yaml.as_bytes()).unwrap();

        let report = PolicyValidator::validate_file(file.path()).unwrap();
        if !report.is_valid {
            eprintln!("Policy validation errors: {:#?}", report.errors);
        }
        assert!(report.is_valid);
        assert_eq!(report.error_count(), 0);
    }

    #[test]
    fn test_validate_empty_rules() {
        let yaml = r#"
metadata:
  id: test-policy
  name: Test Policy
  description: Test policy with no rules
  category: budget
  severity: warning
  status: active
  version: 1.0.0
  ownership:
    author: jane.doe
    owner: team-platform
    approvers: []
  lifecycle:
    created_at: "2024-01-01T00:00:00Z"
    updated_at: "2024-01-01T00:00:00Z"
rules: []
exemptions: []
"#;
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(yaml.as_bytes()).unwrap();

        let report = PolicyValidator::validate_file(file.path()).unwrap();
        assert!(!report.is_valid);
        assert!(report
            .errors
            .iter()
            .any(|e| e.error_code == Some("E201".to_string())));
    }

    #[test]
    fn test_validate_invalid_regex() {
        let yaml = r#"
metadata:
  id: test-policy
  name: Test Policy
  description: Test invalid regex
  category: governance
  severity: error
  status: active
  version: 1.0.0
  ownership:
    author: test.user
    owner: team-platform
    approvers: []
  lifecycle:
    created_at: "2024-01-01T00:00:00Z"
    updated_at: "2024-01-01T00:00:00Z"
rules:
  - name: regex_test
    description: Test invalid regex
    enabled: true
    severity: High
    conditions:
      - condition_type:
          type: resource_type
        operator: matches
        value: "[invalid("
    action:
      type: warn
      message: "Regex validation test"
exemptions: []
"#;
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(yaml.as_bytes()).unwrap();

        let report = PolicyValidator::validate_file(file.path()).unwrap();
        assert!(!report.is_valid);
        assert!(report
            .errors
            .iter()
            .any(|e| e.error_code == Some("E205".to_string())));
    }
}
