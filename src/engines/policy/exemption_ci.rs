// CI integration for exemption validation

use std::process;
use chrono::Utc;

use super::exemption_types::{ExemptionStatus, ExemptionsFile};
use super::exemption_validator::ExemptionValidator;
use crate::errors::CostPilotError;

/// Exit codes for CI integration
pub const EXIT_SUCCESS: i32 = 0;
pub const EXIT_EXEMPTION_EXPIRED: i32 = 2;
pub const EXIT_VALIDATION_ERROR: i32 = 1;

/// Result of CI exemption check
#[derive(Debug, Clone)]
pub struct CIExemptionCheck {
    pub total_exemptions: usize,
    pub active_exemptions: usize,
    pub expired_exemptions: usize,
    pub expiring_soon: usize,
    pub invalid_exemptions: usize,
    pub expired_details: Vec<ExpiredExemptionDetail>,
}

#[derive(Debug, Clone)]
pub struct ExpiredExemptionDetail {
    pub id: String,
    pub policy_name: String,
    pub resource_pattern: String,
    pub expired_on: String,
}

impl CIExemptionCheck {
    /// Check if CI should pass (no expired exemptions)
    pub fn should_pass(&self) -> bool {
        self.expired_exemptions == 0
    }

    /// Get appropriate exit code for CI
    pub fn exit_code(&self) -> i32 {
        if self.expired_exemptions > 0 {
            EXIT_EXEMPTION_EXPIRED
        } else if self.invalid_exemptions > 0 {
            EXIT_VALIDATION_ERROR
        } else {
            EXIT_SUCCESS
        }
    }

    /// Generate human-readable summary for CI output
    pub fn summary(&self) -> String {
        let mut output = String::new();

        output.push_str(&"Exemption Check Summary:\n".to_string());
        output.push_str(&format!("  Total exemptions: {}\n", self.total_exemptions));
        output.push_str(&format!("  Active: {}\n", self.active_exemptions));
        output.push_str(&format!("  Expiring soon: {}\n", self.expiring_soon));
        output.push_str(&format!("  Expired: {}\n", self.expired_exemptions));
        output.push_str(&format!("  Invalid: {}\n", self.invalid_exemptions));

        if !self.expired_details.is_empty() {
            output.push_str("\nExpired exemptions (blocking CI):\n");
            for detail in &self.expired_details {
                output.push_str(&format!(
                    "  - {} [{}] for {} (expired: {})\n",
                    detail.id, detail.policy_name, detail.resource_pattern, detail.expired_on
                ));
            }
        }

        output
    }
}

/// Check exemptions for CI execution
pub fn check_exemptions_for_ci(
    exemptions_file: &ExemptionsFile,
) -> Result<CIExemptionCheck, CostPilotError> {
    let validator = ExemptionValidator::new();

    let mut result = CIExemptionCheck {
        total_exemptions: exemptions_file.exemptions.len(),
        active_exemptions: 0,
        expired_exemptions: 0,
        expiring_soon: 0,
        invalid_exemptions: 0,
        expired_details: Vec::new(),
    };

    for exemption in &exemptions_file.exemptions {
        let status = validator.check_status(exemption);

        match status {
            ExemptionStatus::Active => {
                result.active_exemptions += 1;
            }
            ExemptionStatus::ExpiringSoon { .. } => {
                result.expiring_soon += 1;
            }
            ExemptionStatus::Expired { expired_on } => {
                result.expired_exemptions += 1;
                result.expired_details.push(ExpiredExemptionDetail {
                    id: exemption.id.clone(),
                    policy_name: exemption.policy_name.clone(),
                    resource_pattern: exemption.resource_pattern.clone(),
                    expired_on,
                });
            }
            ExemptionStatus::Invalid { .. } => {
                result.invalid_exemptions += 1;
            }
        }
    }

    Ok(result)
}

/// Validate exemptions and exit with appropriate code for CI
/// This function will call process::exit() if exemptions have expired
pub fn validate_and_exit_ci(exemptions_file: &ExemptionsFile) -> ! {
    match check_exemptions_for_ci(exemptions_file) {
        Ok(check_result) => {
            println!("{}", check_result.summary());

            if check_result.expired_exemptions > 0 {
                eprintln!(
                    "\n❌ CI BLOCKED: {} expired exemption(s) found",
                    check_result.expired_exemptions
                );
                eprintln!("Please remove or renew expired exemptions before proceeding.");
                process::exit(EXIT_EXEMPTION_EXPIRED);
            } else if check_result.invalid_exemptions > 0 {
                eprintln!(
                    "\n⚠️  WARNING: {} invalid exemption(s) found",
                    check_result.invalid_exemptions
                );
                process::exit(EXIT_VALIDATION_ERROR);
            } else {
                println!("\n✓ All exemptions are valid");
                process::exit(EXIT_SUCCESS);
            }
        }
        Err(e) => {
            eprintln!("Error checking exemptions: {}", e);
            process::exit(EXIT_VALIDATION_ERROR);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engines::policy::exemption_types::PolicyExemption;

    fn create_test_exemption(id: &str, expires_at: &str) -> PolicyExemption {
        PolicyExemption {
            id: id.to_string(),
            policy_name: "TEST_POLICY".to_string(),
            resource_pattern: "module.test.*".to_string(),
            justification: "Test exemption".to_string(),
            expires_at: expires_at.to_string(),
            approved_by: "test@example.com".to_string(),
            created_at: "2025-01-01T00:00:00Z".to_string(),
            ticket_ref: Some("TEST-001".to_string()),
        }
    }

    #[test]
    fn test_ci_check_with_active_exemptions() {
        let exemptions_file = ExemptionsFile {
            version: "1.0".to_string(),
            exemptions: vec![
                create_test_exemption("EXE-001", "2026-12-31"),
                create_test_exemption("EXE-002", "2027-01-01"),
            ],
            metadata: None,
        };

        let result = check_exemptions_for_ci(&exemptions_file).unwrap();
        assert_eq!(result.total_exemptions, 2);
        assert_eq!(result.active_exemptions, 2);
        assert_eq!(result.expired_exemptions, 0);
        assert!(result.should_pass());
        assert_eq!(result.exit_code(), EXIT_SUCCESS);
    }

    #[test]
    fn test_ci_check_with_expired_exemptions() {
        let exemptions_file = ExemptionsFile {
            version: "1.0".to_string(),
            exemptions: vec![
                create_test_exemption("EXE-001", "2024-01-01"), // Expired
                create_test_exemption("EXE-002", "2026-12-31"), // Active
            ],
            metadata: None,
        };

        let result = check_exemptions_for_ci(&exemptions_file).unwrap();
        assert_eq!(result.total_exemptions, 2);
        assert_eq!(result.active_exemptions, 1);
        assert_eq!(result.expired_exemptions, 1);
        assert!(!result.should_pass());
        assert_eq!(result.exit_code(), EXIT_EXEMPTION_EXPIRED);
        assert_eq!(result.expired_details.len(), 1);
        assert_eq!(result.expired_details[0].id, "EXE-001");
    }

    #[test]
    fn test_ci_check_with_expiring_soon() {
        // Calculate a date that's within 30 days
        let today = Utc::now().date_naive();
        let expiring_date = today + chrono::Duration::days(15);
        let expiring_str = expiring_date.format("%Y-%m-%d").to_string();

        let exemptions_file = ExemptionsFile {
            version: "1.0".to_string(),
            exemptions: vec![create_test_exemption("EXE-001", &expiring_str)],
            metadata: None,
        };

        let result = check_exemptions_for_ci(&exemptions_file).unwrap();
        assert_eq!(result.total_exemptions, 1);
        assert_eq!(result.expiring_soon, 1);
        assert_eq!(result.expired_exemptions, 0);
        assert!(result.should_pass()); // Expiring soon still passes
        assert_eq!(result.exit_code(), EXIT_SUCCESS);
    }

    #[test]
    fn test_ci_check_summary_output() {
        let exemptions_file = ExemptionsFile {
            version: "1.0".to_string(),
            exemptions: vec![
                create_test_exemption("EXE-001", "2024-01-01"), // Expired
                create_test_exemption("EXE-002", "2026-12-31"), // Active
            ],
            metadata: None,
        };

        let result = check_exemptions_for_ci(&exemptions_file).unwrap();
        let summary = result.summary();

        assert!(summary.contains("Total exemptions: 2"));
        assert!(summary.contains("Active: 1"));
        assert!(summary.contains("Expired: 1"));
        assert!(summary.contains("EXE-001"));
        assert!(summary.contains("expired: 2024-01-01"));
    }

    #[test]
    fn test_multiple_expired_exemptions_blocked() {
        let exemptions_file = ExemptionsFile {
            version: "1.0".to_string(),
            exemptions: vec![
                create_test_exemption("EXE-001", "2024-01-01"),
                create_test_exemption("EXE-002", "2024-06-01"),
                create_test_exemption("EXE-003", "2024-12-01"),
            ],
            metadata: None,
        };

        let result = check_exemptions_for_ci(&exemptions_file).unwrap();
        assert_eq!(result.expired_exemptions, 3);
        assert_eq!(result.expired_details.len(), 3);
        assert!(!result.should_pass());
        assert_eq!(result.exit_code(), EXIT_EXEMPTION_EXPIRED);
    }

    #[test]
    fn test_empty_exemptions_file_passes() {
        let exemptions_file = ExemptionsFile {
            version: "1.0".to_string(),
            exemptions: vec![],
            metadata: None,
        };

        let result = check_exemptions_for_ci(&exemptions_file).unwrap();
        assert_eq!(result.total_exemptions, 0);
        assert!(result.should_pass());
        assert_eq!(result.exit_code(), EXIT_SUCCESS);
    }
}
