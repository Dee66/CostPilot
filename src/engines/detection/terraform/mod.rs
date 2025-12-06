// Terraform module

pub mod parser;
pub mod normalize;

pub use parser::{parse_terraform_plan, convert_to_resource_changes, TerraformPlan};
pub use normalize::normalize_resource;
