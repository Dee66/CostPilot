/// Test helpers and utilities for CostPilot test suite
/// 
/// This module provides common testing utilities, fixtures, and assertions
/// used across unit, integration, and E2E tests.

pub mod builders;
pub mod fixtures;
pub mod assertions;
pub mod generators;
pub mod models;
pub mod wasm_helpers;
pub mod compat_models;

pub use fixtures::*;
pub use assertions::*;
pub use generators::*;
pub use models::*;
pub use compat_models::*;

use costpilot::edition::{EditionContext, EditionMode, Capabilities};

/// Create a deterministic free-mode EditionContext for testing
pub fn free() -> EditionContext {
    EditionContext {
        mode: EditionMode::Free,
        license: None,
        pro_engine: None,
        capabilities: Capabilities {
            allow_predict: false,
            allow_explain_full: false,
            allow_autofix: false,
            allow_mapping_deep: false,
            allow_trend: false,
            allow_policy_enforce: false,
            allow_slo_enforce: false,
        },
        pro: None,
        paths: costpilot::edition::EditionPaths::default(),
    }
}

/// Create a mocked premium EditionContext for testing
/// Note: Uses stub ProEngine - actual WASM not loaded
pub fn premium() -> EditionContext {
    use costpilot::edition::pro_handle::ProEngineHandle;
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
        paths: costpilot::edition::EditionPaths::default(),
    }
}

/// Skip test if premium edition not available
#[macro_export]
macro_rules! require_premium {
    () => {
        if !cfg!(feature = "premium") {
            eprintln!("⚠️  Test requires Premium edition - skipping");
            return;
        }
    };
}

