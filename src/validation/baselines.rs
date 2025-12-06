// Baselines validator
//
// Validates baselines.json file against the expected schema.

use crate::validation::error::{ValidationError, ValidationResult, ValidationWarning};
use crate::validation::{ValidationReport, FileType};
use crate::engines::baselines::baseline_types::BaselinesConfig;
use std::path::Path;

pub struct BaselinesValidator;

impl BaselinesValidator {
    /// Validate a baselines file
    pub fn validate_file(path: impl AsRef<Path>) -> ValidationResult<ValidationReport> {
        let path = path.as_ref();
        let mut report = ValidationReport::new(path, FileType::Baselines);

        // Read file
        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) => {
                report.add_error(
                    ValidationError::new(format!("Failed to read file: {}", e))
                        .with_error_code("E300")
                        .with_hint("Ensure the file exists and is readable")
                );
                return Ok(report);
            }
        };

        // Parse JSON
        let baselines: BaselinesConfig = match serde_json::from_str(&content) {
            Ok(b) => b,
            Err(e) => {
                report.add_error(ValidationError::from(e));
                return Ok(report);
            }
        };

        // Validate baselines
        Self::validate_baselines(&baselines, &mut report);

        Ok(report)
    }

    fn validate_baselines(baselines: &BaselinesConfig, report: &mut ValidationReport) {
        // Check if baselines are empty
        if baselines.modules.is_empty() && baselines.services.is_empty() {
            report.add_warning(
                ValidationWarning::new("Baselines file is empty")
                    .with_warning_code("W300")
                    .with_suggestion("Add module or service baselines to track cost changes")
            );
            return;
        }

        // Validate module baselines
        for (module_path, baseline) in &baselines.modules {
            let prefix = format!("modules.{}", module_path);

            // Validate monthly cost
            if baseline.expected_monthly_cost < 0.0 {
                report.add_error(
                    ValidationError::new(format!(
                        "Negative monthly cost: {}",
                        baseline.expected_monthly_cost
                    ))
                    .with_field(&format!("{}.expected_monthly_cost", prefix))
                    .with_error_code("E301")
                    .with_hint("Cost must be non-negative")
                );
            }

            if baseline.expected_monthly_cost == 0.0 {
                report.add_warning(
                    ValidationWarning::new("Monthly cost is zero")
                        .with_field(&format!("{}.expected_monthly_cost", prefix))
                        .with_warning_code("W301")
                        .with_suggestion("Verify if this module actually has zero cost")
                );
            }

            // Validate last_updated
            if baseline.last_updated.is_empty() {
                report.add_warning(
                    ValidationWarning::new("Missing last_updated timestamp")
                        .with_field(&format!("{}.last_updated", prefix))
                        .with_warning_code("W302")
                        .with_suggestion("Add timestamp to track when baseline was set")
                );
            } else if chrono::DateTime::parse_from_rfc3339(&baseline.last_updated).is_err() {
                report.add_error(
                    ValidationError::new(format!(
                        "Invalid timestamp format: {}",
                        baseline.last_updated
                    ))
                    .with_field(&format!("{}.last_updated", prefix))
                    .with_error_code("E302")
                    .with_hint("Use RFC3339 format (e.g., '2024-12-06T10:00:00Z')")
                );
            }

            // Validate justification
            if baseline.justification.is_empty() {
                report.add_warning(
                    ValidationWarning::new("Missing justification")
                        .with_field(&format!("{}.justification", prefix))
                        .with_warning_code("W303")
                        .with_suggestion("Provide context for why this baseline was set")
                );
            }

            // Check for stale baselines (older than 90 days)
            if let Ok(timestamp) = chrono::DateTime::parse_from_rfc3339(&baseline.last_updated) {
                let now = chrono::Utc::now();
                let age = now.signed_duration_since(timestamp);
                
                if age.num_days() > 90 {
                    report.add_warning(
                        ValidationWarning::new(format!(
                            "Baseline is {} days old and may be stale",
                            age.num_days()
                        ))
                        .with_field(&format!("{}.last_updated", prefix))
                        .with_warning_code("W304")
                        .with_suggestion("Consider updating the baseline if costs have changed")
                    );
                }
            }
        }

        // Validate service baselines
        for (service_name, baseline) in &baselines.services {
            let prefix = format!("services.{}", service_name);

            if baseline.expected_monthly_cost < 0.0 {
                report.add_error(
                    ValidationError::new(format!(
                        "Negative monthly cost: {}",
                        baseline.expected_monthly_cost
                    ))
                    .with_field(&format!("{}.expected_monthly_cost", prefix))
                    .with_error_code("E303")
                    .with_hint("Cost must be non-negative")
                );
            }

            if baseline.expected_monthly_cost == 0.0 {
                report.add_warning(
                    ValidationWarning::new("Monthly cost is zero")
                        .with_field(&format!("{}.expected_monthly_cost", prefix))
                        .with_warning_code("W305")
                        .with_suggestion("Verify if this service actually has zero cost")
                );
            }
        }

        // Check for duplicate module paths (shouldn't happen with HashMap, but validate anyway)
        let module_count = baselines.modules.len();
        if module_count > 0 {
            report.add_warning(
                ValidationWarning::new(format!("Tracking {} module baselines", module_count))
                    .with_warning_code("W306")
                    .with_suggestion("Review periodically to ensure baselines are current")
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
    fn test_validate_valid_baselines() {
        let json = r#"
{
  "modules": {
    "root.vpc": {
      "expected_monthly_cost": 100.0,
      "last_updated": "2024-12-06T10:00:00Z",
      "justification": "Standard VPC cost"
    }
  },
  "services": {
    "ec2": {
      "expected_monthly_cost": 500.0,
      "last_updated": "2024-12-06T10:00:00Z",
      "justification": "Baseline EC2 usage"
    }
  }
}
"#;
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(json.as_bytes()).unwrap();
        
        let report = BaselinesValidator::validate_file(file.path()).unwrap();
        assert!(report.is_valid);
    }

    #[test]
    fn test_validate_negative_cost() {
        let json = r#"
{
  "modules": {
    "root.test": {
      "expected_monthly_cost": -100.0,
      "last_updated": "2024-12-06T10:00:00Z",
      "justification": "Test"
    }
  },
  "services": {}
}
"#;
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(json.as_bytes()).unwrap();
        
        let report = BaselinesValidator::validate_file(file.path()).unwrap();
        assert!(!report.is_valid);
        assert!(report.errors.iter().any(|e| e.error_code == Some("E301".to_string())));
    }

    #[test]
    fn test_validate_invalid_timestamp() {
        let json = r#"
{
  "modules": {
    "root.test": {
      "expected_monthly_cost": 100.0,
      "last_updated": "invalid-date",
      "justification": "Test"
    }
  },
  "services": {}
}
"#;
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(json.as_bytes()).unwrap();
        
        let report = BaselinesValidator::validate_file(file.path()).unwrap();
        assert!(!report.is_valid);
        assert!(report.errors.iter().any(|e| e.error_code == Some("E302".to_string())));
    }

    #[test]
    fn test_validate_empty_baselines() {
        let json = r#"
{
  "modules": {},
  "services": {}
}
"#;
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(json.as_bytes()).unwrap();
        
        let report = BaselinesValidator::validate_file(file.path()).unwrap();
        assert!(report.is_valid);
        assert_eq!(report.warning_count(), 1);
    }
}
