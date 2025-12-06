// WASM module exports

pub mod runtime;

pub use runtime::{
    SandboxLimits, EngineBudget, ValidationResult,
    validate_input_size, validate_json_depth, MemoryTracker,
};

#[cfg(target_arch = "wasm32")]
pub use runtime::init;
