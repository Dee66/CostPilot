use chrono::{NaiveDate, Utc};
use serde_yaml;
use std::fs;
use std::path::Path;

use super::exemption_types::{ExemptionConfig, ExemptionStatus, ExemptionsFile, PolicyExemption};
use crate::errors::CostPilotError;

/// Validates and manages policy exemptions
pub struct ExemptionValidator {
    config: ExemptionConfig,
}

impl ExemptionValidator {
    /// Create a new exemption validator with default configuration
    pub fn new() -> Self {
        Self {
            config: ExemptionConfig::default(),
        }
    }

    /// Create a new exemption validator with custom configuration
    pub fn with_config(config: ExemptionConfig) -> Self {
        Self { config }
    }

    /// Load exemptions from a YAML file
    pub fn load_from_file<P: AsRef<Path>>(
        &self,
        path: P,
    ) -> Result<ExemptionsFile, CostPilotError> {
        let path = path.as_ref();

        if !path.exists() {
            return Err(CostPilotError::file_not_found(
                path.to_string_lossy().to_string(),
            ));
        }

        let contents = fs::read_to_string(path).map_err(|e| {
            CostPilotError::io_error(format!("Failed to read exemptions file: {}", e))
        })?;

        self.parse_yaml(&contents)
    }

    /// Parse exemptions from YAML string
    pub fn parse_yaml(&self, yaml: &str) -> Result<ExemptionsFile, CostPilotError> {
        let exemptions: ExemptionsFile = serde_yaml::from_str(yaml).map_err(|e| {
            CostPilotError::parse_error(format!("Failed to parse exemptions YAML: {}", e))
        })?;

        self.validate_exemptions_file(&exemptions)?;

        Ok(exemptions)
    }

    /// Validate the entire exemptions file structure
    fn validate_exemptions_file(&self, file: &ExemptionsFile) -> Result<(), CostPilotError> {
        // Check version format (must be semver-like)
        if !file.version.contains('.') {
            return Err(CostPilotError::validation_error(
                "Exemptions file version must be in semver format (e.g., '1.0')".to_string(),
            ));
        }

        // Validate each exemption
        for (idx, exemption) in file.exemptions.iter().enumerate() {
            self.validate_exemption(exemption).map_err(|e| {
                CostPilotError::validation_error(format!(
                    "Invalid exemption at index {}: {}",
                    idx, e
                ))
            })?;
        }

        // Check for duplicate IDs
        let mut ids = std::collections::HashSet::new();
        for exemption in &file.exemptions {
            if !ids.insert(&exemption.id) {
                return Err(CostPilotError::validation_error(format!(
                    "Duplicate exemption ID: {}",
                    exemption.id
                )));
            }
        }

        Ok(())
    }

    /// Validate a single exemption
    pub fn validate_exemption(&self, exemption: &PolicyExemption) -> Result<(), CostPilotError> {
        // Check required fields are non-empty
        if exemption.id.is_empty() {
            return Err(CostPilotError::validation_error(
                "Exemption ID cannot be empty".to_string(),
            ));
        }

        if exemption.policy_name.is_empty() {
            return Err(CostPilotError::validation_error(
                "Policy name cannot be empty".to_string(),
            ));
        }

        if exemption.resource_pattern.is_empty() {
            return Err(CostPilotError::validation_error(
                "Resource pattern cannot be empty".to_string(),
            ));
        }

        if exemption.justification.is_empty() {
            return Err(CostPilotError::validation_error(
                "Justification cannot be empty".to_string(),
            ));
        }

        if exemption.approved_by.is_empty() {
            return Err(CostPilotError::validation_error(
                "Approved by cannot be empty".to_string(),
            ));
        }

        // Validate date formats
        self.validate_date(&exemption.expires_at, "expires_at")?;
        self.validate_iso8601_timestamp(&exemption.created_at, "created_at")?;

        // Check expiration is not too far in future
        let expires =
            NaiveDate::parse_from_str(&exemption.expires_at, "%Y-%m-%d").map_err(|_| {
                CostPilotError::validation_error(format!(
                    "Invalid expiration date format: {}",
                    exemption.expires_at
                ))
            })?;

        let created = self.parse_created_date(&exemption.created_at)?;
        let duration_days = (expires - created).num_days();

        if duration_days > self.config.max_duration_days as i64 {
            return Err(CostPilotError::validation_error(format!(
                "Exemption duration {} days exceeds maximum allowed {} days",
                duration_days, self.config.max_duration_days
            )));
        }

        if duration_days < 0 {
            return Err(CostPilotError::validation_error(
                "Expiration date must be after creation date".to_string(),
            ));
        }

        Ok(())
    }

    /// Check the status of an exemption (active, expired, expiring soon)
    pub fn check_status(&self, exemption: &PolicyExemption) -> ExemptionStatus {
        // First validate the exemption
        if let Err(e) = self.validate_exemption(exemption) {
            return ExemptionStatus::Invalid {
                reason: e.to_string(),
            };
        }

        let expires = match NaiveDate::parse_from_str(&exemption.expires_at, "%Y-%m-%d") {
            Ok(date) => date,
            Err(_) => {
                return ExemptionStatus::Invalid {
                    reason: "Invalid expiration date format".to_string(),
                }
            }
        };

        let today = Utc::now().date_naive();
        let days_until_expiry = (expires - today).num_days();

        if days_until_expiry < 0 {
            ExemptionStatus::Expired {
                expired_on: exemption.expires_at.clone(),
            }
        } else if days_until_expiry <= self.config.warning_threshold_days as i64 {
            ExemptionStatus::ExpiringSoon {
                expires_in_days: days_until_expiry as u32,
            }
        } else {
            ExemptionStatus::Active
        }
    }

    /// Check if an exemption applies to a policy violation
    pub fn is_exempted(
        &self,
        exemption: &PolicyExemption,
        policy_name: &str,
        resource_id: &str,
    ) -> bool {
        // Check if exemption is active
        if self.config.enforce_expiration {
            match self.check_status(exemption) {
                ExemptionStatus::Active | ExemptionStatus::ExpiringSoon { .. } => {
                    // Continue to check if it matches
                }
                _ => return false, // Expired or invalid
            }
        }

        // Check if exemption matches the policy and resource
        exemption.matches(policy_name, resource_id)
    }

    /// Find all active exemptions for a given policy and resource
    pub fn find_exemptions<'a>(
        &self,
        exemptions: &'a ExemptionsFile,
        policy_name: &str,
        resource_id: &str,
    ) -> Vec<&'a PolicyExemption> {
        exemptions
            .exemptions
            .iter()
            .filter(|e| self.is_exempted(e, policy_name, resource_id))
            .collect()
    }

    /// Validate date format (YYYY-MM-DD)
    fn validate_date(&self, date_str: &str, field_name: &str) -> Result<(), CostPilotError> {
        NaiveDate::parse_from_str(date_str, "%Y-%m-%d").map_err(|_| {
            CostPilotError::validation_error(format!(
                "Invalid date format for {}: expected YYYY-MM-DD, got '{}'",
                field_name, date_str
            ))
        })?;
        Ok(())
    }

    /// Validate ISO 8601 timestamp format
    fn validate_iso8601_timestamp(
        &self,
        timestamp: &str,
        field_name: &str,
    ) -> Result<(), CostPilotError> {
        // Simple validation - check for ISO 8601 format (YYYY-MM-DDTHH:MM:SSZ or with +00:00)
        if !timestamp.contains('T') {
            return Err(CostPilotError::validation_error(format!(
                "Invalid ISO 8601 timestamp for {}: expected format YYYY-MM-DDTHH:MM:SSZ",
                field_name
            )));
        }
        Ok(())
    }

    /// Parse created_at date to NaiveDate for comparison
    fn parse_created_date(&self, created_at: &str) -> Result<NaiveDate, CostPilotError> {
        // Extract date part from ISO 8601 timestamp (YYYY-MM-DD)
        let date_part = created_at.split('T').next().ok_or_else(|| {
            CostPilotError::validation_error("Invalid created_at timestamp".to_string())
        })?;

        NaiveDate::parse_from_str(date_part, "%Y-%m-%d").map_err(|_| {
            CostPilotError::validation_error("Invalid created_at date format".to_string())
        })
    }
}

impl Default for ExemptionValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_valid_exemption(expires_at: &str) -> PolicyExemption {
        // Use a created_at that's reasonable for the expires_at
        let created_at = if expires_at.starts_with("2024") {
            "2024-01-01T00:00:00Z"
        } else {
            "2025-12-01T00:00:00Z"
        };
        
        PolicyExemption {
            id: "EXE-001".to_string(),
            policy_name: "NAT_GATEWAY_LIMIT".to_string(),
            resource_pattern: "module.vpc.*".to_string(),
            justification: "Production requirement".to_string(),
            expires_at: expires_at.to_string(),
            approved_by: "ops@example.com".to_string(),
            created_at: created_at.to_string(),
            ticket_ref: Some("JIRA-123".to_string()),
        }
    }

    #[test]
    fn test_validate_exemption_valid() {
        let validator = ExemptionValidator::new();
        let exemption = create_valid_exemption("2026-06-01");

        assert!(validator.validate_exemption(&exemption).is_ok());
    }

    #[test]
    fn test_validate_exemption_empty_id() {
        let validator = ExemptionValidator::new();
        let mut exemption = create_valid_exemption("2026-06-01");
        exemption.id = "".to_string();

        assert!(validator.validate_exemption(&exemption).is_err());
    }

    #[test]
    fn test_validate_exemption_empty_justification() {
        let validator = ExemptionValidator::new();
        let mut exemption = create_valid_exemption("2026-06-01");
        exemption.justification = "".to_string();

        assert!(validator.validate_exemption(&exemption).is_err());
    }

    #[test]
    fn test_validate_exemption_invalid_date_format() {
        let validator = ExemptionValidator::new();
        let exemption = create_valid_exemption("2026/06/01"); // Wrong format

        assert!(validator.validate_exemption(&exemption).is_err());
    }

    #[test]
    fn test_validate_exemption_expired_before_created() {
        let validator = ExemptionValidator::new();
        let exemption = create_valid_exemption("2025-11-01"); // Before created_at

        assert!(validator.validate_exemption(&exemption).is_err());
    }

    #[test]
    fn test_check_status_active() {
        let validator = ExemptionValidator::new();
        let exemption = create_valid_exemption("2026-06-01");

        match validator.check_status(&exemption) {
            ExemptionStatus::Active => {}
            _ => panic!("Expected Active status"),
        }
    }

    #[test]
    fn test_check_status_expired() {
        let validator = ExemptionValidator::new();
        let exemption = create_valid_exemption("2024-06-01");

        match validator.check_status(&exemption) {
            ExemptionStatus::Expired { .. } => {}
            status => panic!("Expected Expired status, got {:?}", status),
        }
    }

    #[test]
    fn test_is_exempted_with_enforcement() {
        let validator = ExemptionValidator::new();
        let active_exemption = create_valid_exemption("2026-06-01");
        let expired_exemption = create_valid_exemption("2024-06-01");

        assert!(validator.is_exempted(&active_exemption, "NAT_GATEWAY_LIMIT", "module.vpc.nat[0]"));
        assert!(!validator.is_exempted(
            &expired_exemption,
            "NAT_GATEWAY_LIMIT",
            "module.vpc.nat[0]"
        ));
    }

    #[test]
    fn test_parse_yaml_valid() {
        let validator = ExemptionValidator::new();
        let yaml = r#"
version: "1.0"
exemptions:
  - id: "EXE-001"
    policy_name: "NAT_GATEWAY_LIMIT"
    resource_pattern: "module.vpc.*"
    justification: "Production requirement"
    expires_at: "2026-06-01"
    approved_by: "ops@example.com"
    created_at: "2025-12-01T00:00:00Z"
    ticket_ref: "JIRA-123"
"#;

        let result = validator.parse_yaml(yaml);
        assert!(result.is_ok());
        let file = result.unwrap();
        assert_eq!(file.exemptions.len(), 1);
        assert_eq!(file.exemptions[0].id, "EXE-001");
    }

    #[test]
    fn test_parse_yaml_duplicate_ids() {
        let validator = ExemptionValidator::new();
        let yaml = r#"
version: "1.0"
exemptions:
  - id: "EXE-001"
    policy_name: "NAT_GATEWAY_LIMIT"
    resource_pattern: "module.vpc.*"
    justification: "Production requirement"
    expires_at: "2026-06-01"
    approved_by: "ops@example.com"
    created_at: "2025-12-01T00:00:00Z"
  - id: "EXE-001"
    policy_name: "EC2_INSTANCE_TYPE"
    resource_pattern: "module.app.*"
    justification: "Another requirement"
    expires_at: "2026-06-01"
    approved_by: "dev@example.com"
    created_at: "2025-12-01T00:00:00Z"
"#;

        let result = validator.parse_yaml(yaml);
        assert!(result.is_err());
    }

    #[test]
    fn test_find_exemptions() {
        let validator = ExemptionValidator::new();
        let file = ExemptionsFile {
            version: "1.0".to_string(),
            exemptions: vec![
                create_valid_exemption("2026-06-01"),
                PolicyExemption {
                    id: "EXE-002".to_string(),
                    policy_name: "EC2_INSTANCE_TYPE".to_string(),
                    resource_pattern: "module.app.*".to_string(),
                    justification: "Legacy app".to_string(),
                    expires_at: "2026-06-01".to_string(),
                    approved_by: "dev@example.com".to_string(),
                    created_at: "2025-12-01T00:00:00Z".to_string(),
                    ticket_ref: None,
                },
            ],
            metadata: None,
        };

        let matches = validator.find_exemptions(&file, "NAT_GATEWAY_LIMIT", "module.vpc.nat[0]");
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].id, "EXE-001");
    }
}
