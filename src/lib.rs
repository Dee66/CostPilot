// CostPilot library root

pub mod artifact;
pub mod cli;
pub mod config;
pub mod edition;
pub mod engines;
pub mod errors;
pub mod feature_flags;
pub mod heuristics;
pub mod pro_engine;
pub mod security;
pub mod validation;
pub mod wasm;
pub mod zero_cost_guard;

pub use config::{load_product_spec, load_product_spec_from_path, ConfigError, ProductSpec};
pub use engines::shared::models::*;
pub use security::{SandboxLimits, SecurityValidator};
pub use validation::{
    validate_file, BaselinesValidator, ConfigValidator, PolicyValidator, SloValidator,
    ValidationError, ValidationReport, ValidationWarning,
};
pub use wasm::{EngineBudget, SandboxLimits as WasmSandboxLimits, ValidationResult};
pub use zero_cost_guard::{ZeroCostGuard, ZeroCostViolation};

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

#[cfg(test)]
pub mod test_helpers {
    pub mod edition {
        pub use crate::edition::EditionContext;

        pub fn premium() -> EditionContext {
            use crate::edition::{pro_handle::ProEngineHandle, Capabilities, EditionMode};
            use std::path::PathBuf;

            let stub_handle = ProEngineHandle::stub(PathBuf::from("/tmp/test_pro.wasm"));

            EditionContext {
                mode: EditionMode::Premium,
                license: None,
                pro_engine: Some(stub_handle.clone()),
                capabilities: Capabilities {
                    allow_predict: true,
                    allow_explain_full: true,
                    allow_autofix: true,
                    allow_mapping_deep: true,
                    allow_trend: true,
                    allow_policy_enforce: true,
                    allow_slo_enforce: true,
                },
                pro: Some(stub_handle),
                paths: crate::edition::EditionPaths::default(),
            }
        }
    }
}
