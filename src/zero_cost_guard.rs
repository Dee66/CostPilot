// Zero-cost guard module for enforcing zero_cost_policy invariants

use crate::engines::shared::error_model::{CostPilotError, ErrorCategory};
use std::process::Command;

/// Zero-cost policy violations
#[derive(Debug, Clone, PartialEq)]
pub enum ZeroCostViolation {
    TerraformApplyDetected,
    TerraformPlanExecuteDetected,
    CloudSdkCallDetected(String),
    NetworkCallDetected(String),
    RealDeploymentDetected,
    ChargeableApiCallDetected,
    ForbiddenCommand(String),
}

/// Zero-cost guard that enforces no real cloud costs or network calls
pub struct ZeroCostGuard;

impl ZeroCostGuard {
    /// Create a new zero-cost guard
    pub fn new() -> Self {
        Self
    }

    /// Enforce zero-cost policy before executing any command
    /// This should be called at the start of all CLI commands
    pub fn enforce_zero_cost(&self) -> Result<(), CostPilotError> {
        // Check environment for forbidden variables
        self.check_environment()?;

        // Check for running terraform processes that might be executing
        self.check_running_processes()?;

        // Validate that we're not in a directory with terraform state that suggests real deployments
        self.check_terraform_state()?;

        // Check for network proxy settings that would enable network access
        self.check_network_clients()?;

        Ok(())
    }

    /// Check environment variables for forbidden patterns
    fn check_environment(&self) -> Result<(), CostPilotError> {
        // Check for AWS credentials that would enable real API calls
        if std::env::var("AWS_ACCESS_KEY_ID").is_ok()
            && std::env::var("AWS_SECRET_ACCESS_KEY").is_ok()
        {
            return Err(CostPilotError::new(
                "ZERO_COST_001",
                ErrorCategory::SecurityViolation,
                "AWS credentials detected in environment - CostPilot must run without IAM permissions"
            ).with_hint("Unset AWS_ACCESS_KEY_ID and AWS_SECRET_ACCESS_KEY or use a clean environment"));
        }

        // Check for other cloud provider credentials
        let forbidden_env_vars = [
            "GOOGLE_APPLICATION_CREDENTIALS",
            "AZURE_CLIENT_ID",
            "AZURE_CLIENT_SECRET",
            "AZURE_TENANT_ID",
        ];

        for var in &forbidden_env_vars {
            if std::env::var(var).is_ok() {
                return Err(CostPilotError::new(
                    "ZERO_COST_002",
                    ErrorCategory::SecurityViolation,
                    format!("Cloud credentials detected: {} - CostPilot must run without permissions", var)
                ).with_hint("Unset cloud provider credentials for zero-IAM operation"));
            }
        }

        Ok(())
    }

    /// Check for running terraform processes that might indicate real deployments
    fn check_running_processes(&self) -> Result<(), CostPilotError> {
        // Check if terraform is currently running
        match Command::new("pgrep").arg("-f").arg("terraform").output() {
            Ok(output) if !output.stdout.is_empty() => {
                let processes = String::from_utf8_lossy(&output.stdout);
                return Err(CostPilotError::new(
                    "ZERO_COST_003",
                    ErrorCategory::SecurityViolation,
                    format!("Terraform process detected running: {}", processes.trim())
                ).with_hint("Ensure no terraform apply/plan operations are running during CostPilot analysis"));
            }
            _ => {} // pgrep not available or no terraform processes
        }

        Ok(())
    }

    /// Check for terraform state files that suggest real deployments
    fn check_terraform_state(&self) -> Result<(), CostPilotError> {
        let state_files = ["terraform.tfstate", ".terraform.tfstate", "terraform.tfstate.backup"];

        for state_file in &state_files {
            if std::path::Path::new(state_file).exists() {
                return Err(CostPilotError::new(
                    "ZERO_COST_004",
                    ErrorCategory::SecurityViolation,
                    format!("Terraform state file detected: {} - suggests real infrastructure", state_file)
                ).with_hint("CostPilot analyzes plans only - remove state files or run in clean directory"));
            }
        }

        // Check for .terraform directory (terraform init artifacts)
        if std::path::Path::new(".terraform").exists() {
            return Err(CostPilotError::new(
                "ZERO_COST_005",
                ErrorCategory::SecurityViolation,
                ".terraform directory detected - suggests terraform init has been run"
            ).with_hint("CostPilot analyzes static plans - avoid terraform init in analysis directory"));
        }

        Ok(())
    }

    /// Check for network client libraries that could make HTTP calls
    fn check_network_clients(&self) -> Result<(), CostPilotError> {
        // Check if reqwest or other HTTP clients are loaded
        // This is a runtime check - in a real implementation, this would check
        // if the libraries are linked or if network syscalls are available

        // For now, we'll do a basic check by looking for network-related environment variables
        // that might indicate network access is expected
        let network_env_vars = [
            "http_proxy", "https_proxy", "HTTP_PROXY", "HTTPS_PROXY",
            "no_proxy", "NO_PROXY", "all_proxy", "ALL_PROXY"
        ];

        for var in &network_env_vars {
            if std::env::var(var).is_ok() {
                return Err(CostPilotError::new(
                    "ZERO_COST_006",
                    ErrorCategory::SecurityViolation,
                    format!("Network proxy detected: {} - CostPilot must run without network access", var)
                ).with_hint("Unset proxy environment variables for offline operation"));
            }
        }

        Ok(())
    }

    /// Validate that a command string doesn't contain forbidden operations
    pub fn validate_command(&self, command: &str) -> Result<(), ZeroCostViolation> {
        let command_lower = command.to_lowercase();

        // Check for terraform apply
        if command_lower.contains("terraform apply") {
            return Err(ZeroCostViolation::TerraformApplyDetected);
        }

        // Check for terraform plan execution (not just parsing)
        if command_lower.contains("terraform plan") && !command_lower.contains("--out") {
            // Allow terraform plan -out file.json for generating plans, but not bare plan
            return Err(ZeroCostViolation::TerraformPlanExecuteDetected);
        }

        // Check for cloud SDK calls
        let cloud_sdk_patterns = [
            "aws s3", "aws ec2", "aws lambda", "aws dynamodb",
            "gcloud ", "az ", "kubectl apply", "kubectl create"
        ];

        for pattern in &cloud_sdk_patterns {
            if command_lower.contains(pattern) {
                return Err(ZeroCostViolation::CloudSdkCallDetected(pattern.to_string()));
            }
        }

        // Check for network calls
        if command_lower.contains("curl ") || command_lower.contains("wget ") {
            return Err(ZeroCostViolation::NetworkCallDetected(command.to_string()));
        }

        Ok(())
    }

    /// Validate file content for zero-cost violations
    pub fn validate_file_content(&self, content: &str) -> Result<(), ZeroCostViolation> {
        let content_lower = content.to_lowercase();

        // Check for terraform apply blocks
        if content_lower.contains("terraform") && content_lower.contains("apply") {
            return Err(ZeroCostViolation::RealDeploymentDetected);
        }

        // Check for cloud SDK imports/calls
        let sdk_patterns = [
            "boto3", "aws-sdk", "@aws-sdk", "google-cloud", "@azure",
            "aws_s3", "aws_ec2", "rusoto", "kube"
        ];

        for pattern in &sdk_patterns {
            if content_lower.contains(pattern) {
                return Err(ZeroCostViolation::CloudSdkCallDetected(pattern.to_string()));
            }
        }

        // Check for network calls
        if content_lower.contains("http") || content_lower.contains("requests") {
            return Err(ZeroCostViolation::NetworkCallDetected("network client usage".to_string()));
        }

        Ok(())
    }
}

impl Default for ZeroCostGuard {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_command_allows_safe_commands() {
        let guard = ZeroCostGuard::new();
        assert!(guard.validate_command("ls -la").is_ok());
        assert!(guard.validate_command("cat file.json").is_ok());
    }

    #[test]
    fn test_validate_command_blocks_terraform_apply() {
        let guard = ZeroCostGuard::new();
        assert_eq!(
            guard.validate_command("terraform apply"),
            Err(ZeroCostViolation::TerraformApplyDetected)
        );
    }

    #[test]
    fn test_validate_command_blocks_cloud_sdk() {
        let guard = ZeroCostGuard::new();
        assert_eq!(
            guard.validate_command("aws s3 ls"),
            Err(ZeroCostViolation::CloudSdkCallDetected("aws s3".to_string()))
        );
    }
}
