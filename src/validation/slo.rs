// SLO validator
//
// Validates SLO YAML files against the expected schema.

use crate::engines::slo::slo_types::{EnforcementLevel, Slo};
use crate::validation::error::{ValidationError, ValidationResult, ValidationWarning};
use crate::validation::{FileType, ValidationReport};
use std::path::Path;

pub struct SloValidator;

impl SloValidator {
    /// Validate an SLO file
    pub fn validate_file(path: impl AsRef<Path>) -> ValidationResult<ValidationReport> {
        let path = path.as_ref();
        let mut report = ValidationReport::new(path, FileType::Slo);

        // Read file
        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) => {
                report.add_error(
                    ValidationError::new(format!("Failed to read file: {}", e))
                        .with_error_code("E400")
                        .with_hint("Ensure the file exists and is readable"),
                );
                return Ok(report);
            }
        };

        // Parse YAML - expecting a map of SLO name -> Slo
        let slos: std::collections::HashMap<String, Slo> = match serde_yaml::from_str(&content) {
            Ok(s) => s,
            Err(e) => {
                report.add_error(ValidationError::from(e));
                return Ok(report);
            }
        };

        // Validate SLOs
        Self::validate_slos(&slos, &mut report);

        Ok(report)
    }

    fn validate_slos(slos: &std::collections::HashMap<String, Slo>, report: &mut ValidationReport) {
        // Check if SLOs are empty
        if slos.is_empty() {
            report.add_warning(
                ValidationWarning::new("SLO file contains no definitions")
                    .with_warning_code("W400")
                    .with_suggestion("Add at least one SLO definition"),
            );
            return;
        }

        for (slo_name, slo) in slos {
            let prefix = format!("slos.{}", slo_name);

            // Validate SLO ID
            if slo.id.is_empty() {
                report.add_error(
                    ValidationError::new("SLO has empty ID")
                        .with_field(format!("{}.id", prefix))
                        .with_error_code("E401")
                        .with_hint("Provide a unique identifier for the SLO"),
                );
            }

            // Validate name
            if slo.name.is_empty() {
                report.add_warning(
                    ValidationWarning::new("SLO has empty name")
                        .with_field(format!("{}.name", prefix))
                        .with_warning_code("W401")
                        .with_suggestion("Provide a human-readable name for the SLO"),
                );
            }

            // Validate target
            if slo.target.is_empty() {
                report.add_error(
                    ValidationError::new("SLO has empty target")
                        .with_field(format!("{}.target", prefix))
                        .with_error_code("E402")
                        .with_hint(
                            "Specify target entity (module name, service name, or 'global')",
                        ),
                );
            }

            // Validate threshold
            if slo.threshold.max_value <= 0.0 {
                report.add_error(
                    ValidationError::new(format!(
                        "Invalid max threshold: {}",
                        slo.threshold.max_value
                    ))
                    .with_field(format!("{}.threshold.max_value", prefix))
                    .with_error_code("E403")
                    .with_hint("Max value must be positive"),
                );
            }

            if slo.threshold.max_value > 10_000_000.0 {
                report.add_warning(
                    ValidationWarning::new(format!(
                        "Very high threshold: ${:.2}",
                        slo.threshold.max_value
                    ))
                    .with_field(format!("{}.threshold.max_value", prefix))
                    .with_warning_code("W402")
                    .with_suggestion("Verify this threshold is correct"),
                );
            }

            if let Some(min) = slo.threshold.min_value {
                if min < 0.0 {
                    report.add_error(
                        ValidationError::new(format!("Invalid min threshold: {}", min))
                            .with_field(format!("{}.threshold.min_value", prefix))
                            .with_error_code("E404")
                            .with_hint("Min value must be non-negative"),
                    );
                }

                if min >= slo.threshold.max_value {
                    report.add_error(
                        ValidationError::new("Min threshold must be less than max threshold")
                            .with_field(format!("{}.threshold", prefix))
                            .with_error_code("E405")
                            .with_hint(format!("Min: {}, Max: {}", min, slo.threshold.max_value)),
                    );
                }
            }

            // Validate warning threshold percentage
            if slo.threshold.warning_threshold_percent <= 0.0
                || slo.threshold.warning_threshold_percent > 100.0
            {
                report.add_error(
                    ValidationError::new(format!(
                        "Invalid warning threshold: {}%",
                        slo.threshold.warning_threshold_percent
                    ))
                    .with_field(format!("{}.threshold.warning_threshold_percent", prefix))
                    .with_error_code("E406")
                    .with_hint("Warning threshold must be between 0 and 100 percent"),
                );
            }

            // Validate enforcement level
            match slo.enforcement {
                EnforcementLevel::Observe => {
                    // Observe mode is always valid
                }
                EnforcementLevel::Warn => {
                    // Warn mode is always valid
                }
                EnforcementLevel::Block | EnforcementLevel::StrictBlock => {
                    report.add_warning(
                        ValidationWarning::new(format!(
                            "SLO uses {:?} enforcement",
                            slo.enforcement
                        ))
                        .with_field(format!("{}.enforcement", prefix))
                        .with_warning_code("W405")
                        .with_suggestion(
                            "Ensure blocking is intentional; it will prevent deployments",
                        ),
                    );
                }
            }
        }

        // Warn if too many SLOs
        if slos.len() > 50 {
            report.add_warning(
                ValidationWarning::new(format!("Large number of SLOs: {}", slos.len()))
                    .with_warning_code("W406")
                    .with_suggestion("Consider consolidating related SLOs for easier management"),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_validate_valid_slo() {
        let yaml = r#"
monthly_budget:
  id: monthly_budget
  name: Monthly Budget
  description: Monthly cost limit
  slo_type: monthly_budget
  target: global
  threshold:
    max_value: 5000.0
  enforcement: warn
  owner: team-platform
  created_at: "2024-01-01T00:00:00Z"

vpc_budget:
  id: vpc_budget
  name: VPC Budget
  description: VPC cost limit
  slo_type: module_budget
  target: root.vpc
  threshold:
    max_value: 1000.0
  enforcement: block
  owner: team-networking
  created_at: "2024-01-01T00:00:00Z"
"#;
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(yaml.as_bytes()).unwrap();

        let report = SloValidator::validate_file(file.path()).unwrap();
        if !report.is_valid {
            eprintln!("Validation errors: {:#?}", report.errors);
            eprintln!("Validation warnings: {:#?}", report.warnings);
        }
        assert!(report.is_valid);
    }

    #[test]
    fn test_validate_negative_target() {
        let yaml = r#"
monthly_budget:
  id: monthly_budget
  name: Monthly Budget
  description: Test negative threshold
  slo_type: monthly_budget
  target: global
  threshold:
    max_value: -1000.0
  enforcement: warn
  owner: team-platform
  created_at: "2024-01-01T00:00:00Z"
"#;
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(yaml.as_bytes()).unwrap();

        let report = SloValidator::validate_file(file.path()).unwrap();
        assert!(!report.is_valid);
        // Error code for negative threshold
        assert!(report
            .errors
            .iter()
            .any(|e| e.error_code == Some("E403".to_string())));
    }

    #[test]
    fn test_validate_empty_module_path() {
        let yaml = r#"
module_budget:
  id: module_budget
  name: Module Budget
  description: Test empty target
  slo_type: module_budget
  target: ""
  threshold:
    max_value: 100.0
  enforcement: warn
  owner: team-platform
  created_at: "2024-01-01T00:00:00Z"
"#;
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(yaml.as_bytes()).unwrap();

        let report = SloValidator::validate_file(file.path()).unwrap();
        assert!(!report.is_valid);
        // E402 is for empty target
        assert!(report
            .errors
            .iter()
            .any(|e| e.error_code == Some("E402".to_string())));
    }

    #[test]
    fn test_validate_zero_resource_count() {
        let yaml = r#"
instance_limit:
  id: instance_limit
  name: Instance Limit
  description: Zero threshold test
  slo_type: resource_count
  target: aws_instance
  threshold:
    max_value: 0.0
  enforcement: block
  owner: team-platform
  created_at: "2024-01-01T00:00:00Z"
"#;
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(yaml.as_bytes()).unwrap();

        let report = SloValidator::validate_file(file.path()).unwrap();
        assert!(!report.is_valid);
        // E403 for invalid max_value (<=0)
        assert!(report
            .errors
            .iter()
            .any(|e| e.error_code == Some("E403".to_string())));
    }
}
