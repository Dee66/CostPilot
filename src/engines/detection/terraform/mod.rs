// Terraform module

pub mod normalize;
pub mod parser;

pub use parser::{convert_to_resource_changes, parse_terraform_plan, TerraformPlan};
// pub use normalize::normalize_resource; // TODO: Fix module structure
