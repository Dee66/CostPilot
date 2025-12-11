// WASM module exports

pub mod runtime;

pub use runtime::{
    validate_input_size, validate_json_depth, EngineBudget, MemoryTracker, SandboxLimits,
    ValidationResult,
};

#[cfg(target_arch = "wasm32")]
pub use runtime::init;
