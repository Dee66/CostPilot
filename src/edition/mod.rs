pub mod capabilities;
pub mod errors;
pub mod messages;
pub mod pro_handle;

pub use capabilities::Capabilities;
pub use errors::{require_premium, UpgradeRequired};
// Remove the legacy gating import to avoid confusion
// pub use gating::require_premium as legacy_require_premium;
pub use messages::{feature_comparison, upgrade_message};
pub use pro_handle::{ProEngineError, ProEngineHandle};

use crate::pro_engine::License;

use hkdf::SimpleHkdf;
use sha2::Sha256;
use std::path::PathBuf;

/// Detect and initialize edition context
pub fn detect_edition() -> Result<EditionContext, String> {
    // Check for test environment variable to force premium mode
    if std::env::var("COSTPILOT_FORCE_PREMIUM").is_ok() {
        return Ok(EditionContext::premium_for_test());
    }

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
        self.mode == EditionMode::Premium
    }

    /// Check if running in Free mode
    pub fn is_free(&self) -> bool {
        !self.is_premium()
    }

    /// Create premium edition context for testing
    pub fn premium_for_test() -> Self {
        Self {
            mode: EditionMode::Premium,
            license: None,
            pro_engine: None,
            capabilities: Capabilities {
                allow_predict: true,
                allow_explain_full: true,
                allow_autofix: true,
                allow_mapping_deep: true,
                allow_trend: true,
                allow_policy_enforce: true,
                allow_slo_enforce: true,
            },
            pro: None,
            paths: EditionPaths::default(),
        }
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

    /// Derive encryption key from license
    pub fn derive_key(&self) -> anyhow::Result<Vec<u8>> {
        if let Some(ref license) = self.license {
            // Use the same key derivation as pro_loader
            let key = crate::pro_engine::crypto::derive_key(&license.license_key);
            Ok(key.to_vec())
        } else {
            anyhow::bail!("No license available for key derivation");
        }
    }

    /// Derive deterministic nonce from license
    pub fn derive_nonce(&self) -> anyhow::Result<Vec<u8>> {
        if let Some(ref license) = self.license {
            // Derive nonce using HKDF with different salt/info
            let hk = SimpleHkdf::<Sha256>::new(Some(b"costpilot-nonce-v1"), license.license_key.as_bytes());
            let mut nonce = [0u8; 12];
            hk.expand(b"nonce", &mut nonce)
                .map_err(|_| anyhow::anyhow!("HKDF expand failed"))?;
            Ok(nonce.to_vec())
        } else {
            anyhow::bail!("No license available for nonce derivation");
        }
    }

    /// Check if the license is expired
    pub fn is_license_expired(&self) -> bool {
        if let Some(ref license) = self.license {
            license.is_expired()
        } else {
            true // No license = expired
        }
    }

    /// Check if the license is valid (exists and not expired)
    pub fn is_license_valid(&self) -> bool {
        self.license.is_some() && !self.is_license_expired()
    }
}

impl Default for EditionContext {
    fn default() -> Self {
        Self::new()
    }
}
