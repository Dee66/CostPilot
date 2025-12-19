// Terraform module

pub mod hcl_parser;
pub mod normalize;
pub mod parser;

pub use hcl_parser::{parse_terraform_config, TerraformConfig};
pub use parser::{convert_to_resource_changes, parse_terraform_plan, TerraformPlan};
// pub use normalize::normalize_resource; // TODO: Fix module structure
