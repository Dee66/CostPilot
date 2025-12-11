use super::sandbox::{SandboxLimits, SandboxViolation};
use crate::errors::{CostPilotError, ErrorCategory};
use regex::Regex;
use std::path::Path;

/// Security validator for Zero-IAM compliance
pub struct SecurityValidator {
    limits: SandboxLimits,
    network_patterns: Vec<Regex>,
    aws_sdk_patterns: Vec<Regex>,
    secret_patterns: Vec<Regex>,
}

impl SecurityValidator {
    /// Create a new security validator with default limits
    pub fn new() -> Self {
        Self::with_limits(SandboxLimits::default())
    }

    /// Create a validator with custom sandbox limits
    pub fn with_limits(limits: SandboxLimits) -> Self {
        // Patterns to detect network operations
        let network_patterns = vec![
            Regex::new(r"https?://").unwrap(),
            Regex::new(r"TcpStream::connect").unwrap(),
            Regex::new(r"UdpSocket::bind").unwrap(),
            Regex::new(r"reqwest::").unwrap(),
            Regex::new(r"hyper::").unwrap(),
        ];

        // Patterns to detect AWS SDK usage
        let aws_sdk_patterns = vec![
            Regex::new(r"aws_sdk_").unwrap(),
            Regex::new(r"rusoto_").unwrap(),
            Regex::new(r"aws_config::").unwrap(),
            Regex::new(r"\.s3\(\)").unwrap(),
            Regex::new(r"\.ec2\(\)").unwrap(),
            Regex::new(r"\.dynamodb\(\)").unwrap(),
        ];

        // Patterns to detect secrets/tokens
        let secret_patterns = vec![
            Regex::new(r"AKIA[0-9A-Z]{16}").unwrap(), // AWS Access Key ID
            Regex::new(r#"(?i)aws_secret_access_key\s*=\s*['"][^'"]+['"]"#).unwrap(),
            Regex::new(r#"(?i)api[_-]?key\s*[:=]\s*['"][^'"]+['"]"#).unwrap(),
            Regex::new(r#"(?i)token\s*[:=]\s*['"][A-Za-z0-9+/]{20,}['"]"#).unwrap(),
            Regex::new(r"Bearer\s+[A-Za-z0-9\-._~+/]+=*").unwrap(),
        ];

        Self {
            limits,
            network_patterns,
            aws_sdk_patterns,
            secret_patterns,
        }
    }

    /// Validate file size before processing
    pub fn validate_file_size(&self, path: &Path) -> Result<(), CostPilotError> {
        let metadata = std::fs::metadata(path).map_err(|e| {
            CostPilotError::new(
                "SEC_001",
                ErrorCategory::FileSystemError,
                format!("Failed to read file metadata: {}", e),
            )
        })?;

        self.limits.check_file_size(metadata.len()).map_err(|v| {
            CostPilotError::new("SEC_002", ErrorCategory::ValidationError, v.to_string()).with_hint(
                format!(
                    "File size limit is {}MB to ensure WASM sandbox safety",
                    self.limits.max_file_size_mb
                ),
            )
        })
    }

    /// Scan code for network operations
    pub fn scan_for_network_calls(&self, code: &str) -> Result<(), SandboxViolation> {
        for pattern in &self.network_patterns {
            if let Some(matched) = pattern.find(code) {
                return Err(SandboxViolation::NetworkAccessDetected {
                    operation: matched.as_str().to_string(),
                });
            }
        }
        Ok(())
    }

    /// Scan code for AWS SDK usage
    pub fn scan_for_aws_sdk(&self, code: &str) -> Result<(), SandboxViolation> {
        for pattern in &self.aws_sdk_patterns {
            if let Some(matched) = pattern.find(code) {
                return Err(SandboxViolation::AwsSdkDetected {
                    service: matched.as_str().to_string(),
                });
            }
        }
        Ok(())
    }

    /// Scan output for secrets/tokens
    pub fn scan_for_secrets(&self, output: &str) -> Result<(), SandboxViolation> {
        for pattern in &self.secret_patterns {
            if let Some(matched) = pattern.find(output) {
                return Err(SandboxViolation::SecretLeakage {
                    pattern: Self::redact_match(matched.as_str()),
                });
            }
        }
        Ok(())
    }

    /// Redact matched secret for safe error reporting
    fn redact_match(s: &str) -> String {
        if s.len() <= 8 {
            "*".repeat(s.len())
        } else {
            format!("{}***{}", &s[..4], &s[s.len() - 4..])
        }
    }

    /// Perform full Zero-IAM validation on code
    pub fn validate_code(&self, code: &str) -> Result<(), CostPilotError> {
        // Check for network calls
        self.scan_for_network_calls(code).map_err(|v| {
            CostPilotError::new("SEC_003", ErrorCategory::SecurityViolation, v.to_string())
                .with_hint(
                    "CostPilot enforces zero-network policy for WASM sandbox safety".to_string(),
                )
        })?;

        // Check for AWS SDK usage
        self.scan_for_aws_sdk(code).map_err(|v| {
            CostPilotError::new("SEC_004", ErrorCategory::SecurityViolation, v.to_string())
                .with_hint(
                    "CostPilot must not use AWS SDK - all analysis is done on static IaC files"
                        .to_string(),
                )
        })?;

        Ok(())
    }

    /// Validate output for secrets
    pub fn validate_output(&self, output: &str) -> Result<(), CostPilotError> {
        self.scan_for_secrets(output).map_err(|v| {
            CostPilotError::new("SEC_005", ErrorCategory::SecurityViolation, v.to_string())
                .with_hint(
                    "Output contains potential secrets - redact before displaying".to_string(),
                )
        })
    }

    /// Get sandbox limits
    pub fn limits(&self) -> &SandboxLimits {
        &self.limits
    }
}

impl Default for SecurityValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_detect_network_call() {
        let validator = SecurityValidator::new();
        let code = r#"
            let url = "https://api.example.com";
            let response = reqwest::get(url).await?;
        "#;

        assert!(validator.scan_for_network_calls(code).is_err());
    }

    #[test]
    fn test_detect_aws_sdk() {
        let validator = SecurityValidator::new();
        let code = r#"
            use aws_sdk_s3::Client;
            let client = Client::new(&config);
        "#;

        assert!(validator.scan_for_aws_sdk(code).is_err());
    }

    #[test]
    fn test_detect_aws_access_key() {
        let validator = SecurityValidator::new();
        let output = "AWS_ACCESS_KEY_ID=AKIAIOSFODNN7EXAMPLE";

        assert!(validator.scan_for_secrets(output).is_err());
    }

    #[test]
    fn test_detect_bearer_token() {
        let validator = SecurityValidator::new();
        let output = "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9";

        assert!(validator.scan_for_secrets(output).is_err());
    }

    #[test]
    fn test_safe_code_passes() {
        let validator = SecurityValidator::new();
        let code = r#"
            use serde_json::Value;
            let config: Value = serde_json::from_str(json_str)?;
        "#;

        assert!(validator.validate_code(code).is_ok());
    }

    #[test]
    fn test_file_size_validation() {
        let validator = SecurityValidator::new();

        // Create a small temp file
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"test content").unwrap();
        file.flush().unwrap();

        assert!(validator.validate_file_size(file.path()).is_ok());
    }

    #[test]
    fn test_redact_match() {
        assert_eq!(
            SecurityValidator::redact_match("AKIAIOSFODNN7EXAMPLE"),
            "AKIA***MPLE"
        );
        assert_eq!(SecurityValidator::redact_match("short"), "*****");
    }
}
