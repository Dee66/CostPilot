pub mod capabilities;
pub mod errors;
pub mod license;
pub mod messages;
pub mod pro_handle;

pub use capabilities::Capabilities;
pub use errors::{require_premium, UpgradeRequired};
// Remove the legacy gating import to avoid confusion
// pub use gating::require_premium as legacy_require_premium;
pub use license::{License, LicenseError};
pub use messages::{feature_comparison, upgrade_message};
pub use pro_handle::{ProEngineError, ProEngineHandle};

use std::path::PathBuf;

/// Detect and initialize edition context
pub fn detect_edition() -> Result<EditionContext, String> {
    let mut edition = EditionContext::free();

    // Attempt to load ProEngine (fails silently for Free mode)
    #[cfg(not(target_arch = "wasm32"))]
    if let Err(e) = crate::pro_engine::load_pro_engine(&mut edition) {
        eprintln!("⚠️  Failed to load Premium engine: {}", e);
        eprintln!("   Running in Free mode");
    }

    Ok(edition)
}

/// Edition mode for CostPilot
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditionMode {
    Free,
    Premium,
}

/// Edition context that determines available features
pub struct EditionContext {
    pub mode: EditionMode,
    pub license: Option<License>,
    pub pro_engine: Option<ProEngineHandle>,
    pub capabilities: Capabilities,
    pub pro: Option<ProEngineHandle>,
    pub paths: EditionPaths,
}

#[derive(Debug, Clone)]
pub struct EditionPaths {
    pub config_dir: PathBuf,
}

impl Default for EditionPaths {
    fn default() -> Self {
        let config_dir = dirs::home_dir()
            .map(|h| h.join(".costpilot"))
            .unwrap_or_else(|| PathBuf::from(".costpilot"));

        Self { config_dir }
    }
}

impl EditionPaths {
    pub fn pro_wasm_path(&self) -> PathBuf {
        self.config_dir.join("pro_engine.wasm.enc")
    }

    pub fn license_path(&self) -> PathBuf {
        self.config_dir.join("license.json")
    }
}

impl Clone for EditionContext {
    fn clone(&self) -> Self {
        Self {
            mode: self.mode,
            license: self.license.clone(),
            pro_engine: self.pro_engine.clone(),
            capabilities: self.capabilities.clone(),
            pro: self.pro.clone(),
            paths: self.paths.clone(),
        }
    }
}

/// Capability flags for edition-specific features
#[derive(Debug, Clone)]
pub struct EditionCapabilities {
    pub autofix_allowed: bool,
    pub trend_allowed: bool,
    pub deep_map_allowed: bool,
    pub enforce_slo_allowed: bool,
    pub enforce_policy_allowed: bool,
    pub explain_advanced_allowed: bool,
}

impl EditionContext {
    /// Create free edition context (default)
    pub fn free() -> Self {
        Self {
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
            paths: EditionPaths::default(),
        }
    }

    /// Create new edition context (legacy)
    pub fn new() -> Self {
        Self::free()
    }

    /// Check if running in Premium mode
    pub fn is_premium(&self) -> bool {
        self.mode == EditionMode::Premium && self.pro.is_some()
    }

    /// Check if running in Free mode
    pub fn is_free(&self) -> bool {
        !self.is_premium()
    }

    /// Require Premium edition, returning ProEngineHandle reference
    pub fn require_pro(
        &self,
        feature: &'static str,
    ) -> Result<&pro_handle::ProEngineHandle, Box<dyn std::error::Error>> {
        if let Some(ref p) = self.pro {
            Ok(p)
        } else {
            Err(Box::new(UpgradeRequired { feature }))
        }
    }

    /// Derive encryption key from license (stub)
    pub fn derive_key(&self) -> anyhow::Result<Vec<u8>> {
        // TODO: Implement HKDF-SHA256 key derivation from license
        Ok(vec![0u8; 32])
    }

    /// Derive deterministic nonce (stub)
    pub fn derive_nonce(&self) -> anyhow::Result<Vec<u8>> {
        // TODO: Implement deterministic nonce generation
        Ok(vec![0u8; 12])
    }

    /// Get path to encrypted WASM file
    pub fn pro_wasm_path(&self) -> std::path::PathBuf {
        self.paths.pro_wasm_path()
    }

    /// Get edition capabilities as a structured object
    pub fn capabilities(&self) -> EditionCapabilities {
        match self.mode {
            EditionMode::Premium => EditionCapabilities {
                autofix_allowed: true,
                trend_allowed: true,
                deep_map_allowed: true,
                enforce_slo_allowed: true,
                enforce_policy_allowed: true,
                explain_advanced_allowed: true,
            },
            EditionMode::Free => EditionCapabilities {
                autofix_allowed: false,
                trend_allowed: false,
                deep_map_allowed: false,
                enforce_slo_allowed: false,
                enforce_policy_allowed: false,
                explain_advanced_allowed: false,
            },
        }
    }
}

impl Default for EditionContext {
    fn default() -> Self {
        Self::new()
    }
}
