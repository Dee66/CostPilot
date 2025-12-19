// Terraform HCL configuration parser

use crate::engines::shared::error_model::{CostPilotError, ErrorCategory, Result};
use hcl::Value;

/// Basic Terraform HCL configuration structure
#[derive(Debug, Clone)]
pub struct TerraformConfig {
    pub content: Value,
}

/// Parse Terraform HCL configuration from string
pub fn parse_terraform_config(hcl_content: &str) -> Result<TerraformConfig> {
    // Parse HCL
    let value: Value = hcl::from_str(hcl_content).map_err(|e| {
        CostPilotError::new(
            "PARSE_002",
            ErrorCategory::ParseError,
            format!("Failed to parse Terraform HCL: {}", e),
        )
        .with_hint("Ensure the input is valid Terraform HCL syntax")
    })?;

    Ok(TerraformConfig { content: value })
}