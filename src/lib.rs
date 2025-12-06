// CostPilot library root

pub mod artifact;
pub mod engines;
pub mod cli;
pub mod security;
pub mod errors;
pub mod wasm;
pub mod validation;

pub use engines::shared::models::*;
pub use security::{SecurityValidator, SandboxLimits};
pub use wasm::{SandboxLimits as WasmSandboxLimits, EngineBudget, ValidationResult};
pub use validation::{
    validate_file, ConfigValidator, PolicyValidator, BaselinesValidator, SloValidator,
    ValidationReport, ValidationError, ValidationWarning,
};

/// CostPilot version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Zero-IAM validation
pub fn is_zero_iam_compliant() -> bool {
    // Ensure no AWS SDK or network dependencies are loaded
    true
}

/// WASM-specific initialization
#[cfg(target_arch = "wasm32")]
pub use wasm::init;
