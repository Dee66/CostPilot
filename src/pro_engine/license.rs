// License management for CostPilot Premium

use crate::pro_engine::loader::{EncryptedBundle, LoaderError};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Rate limiting state for license validation
#[derive(Debug, Clone, Serialize, Deserialize)]
struct RateLimitState {
    attempts: u32,
    last_attempt: u64,
    blocked_until: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    hmac: Option<String>,
}

impl RateLimitState {
    const MAX_ATTEMPTS: u32 = 5; // Max 5 attempts per minute
    const WINDOW_SECS: u64 = 60; // 1 minute window
    const BLOCK_SECS: u64 = 300; // 5 minute block after exceeding limit

    fn new() -> Self {
        let mut state = Self {
            attempts: 0,
            last_attempt: 0,
            blocked_until: None,
            hmac: None,
        };
        state.compute_hmac();
        state
    }

    /// Compute HMAC of state data to prevent tampering
    fn compute_hmac(&mut self) {
        let data = format!(
            "{}:{}:{:?}",
            self.attempts, self.last_attempt, self.blocked_until
        );
        let mut hasher = Sha256::new();
        hasher.update(data.as_bytes());
        hasher.update(b"costpilot-rate-limit-v1"); // Secret salt
        let result = hasher.finalize();
        self.hmac = Some(hex::encode(result));
    }

    /// Verify HMAC to detect tampering
    fn verify_hmac(&self) -> bool {
        let expected_data = format!(
            "{}:{}:{:?}",
            self.attempts, self.last_attempt, self.blocked_until
        );
        let mut hasher = Sha256::new();
        hasher.update(expected_data.as_bytes());
        hasher.update(b"costpilot-rate-limit-v1");
        let expected = hex::encode(hasher.finalize());

        match &self.hmac {
            Some(stored) => stored == &expected,
            None => false, // No HMAC = invalid (legacy files)
        }
    }

    fn load() -> Self {
        // Use HOME environment variable if set, otherwise use relative path
        let home_dir = std::env::var("HOME").ok().map(PathBuf::from);
        Self::load_from_home(home_dir.as_deref())
    }

    fn load_from_home(home_override: Option<&Path>) -> Self {
        let rate_limit_path = if let Some(home) = home_override {
            home.join(".costpilot").join("rate_limit.json")
        } else {
            PathBuf::from(".costpilot/rate_limit.json")
        };

        if rate_limit_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&rate_limit_path) {
                if let Ok(state) = serde_json::from_str::<RateLimitState>(&content) {
                    // Verify HMAC to detect tampering
                    if state.verify_hmac() {
                        return state;
                    }
                    // Tampered file detected - reset to clean state
                    eprintln!("Warning: Rate limit file integrity check failed. Resetting.");
                }
            }
        }
        Self::new()
    }

    fn save(&self) {
        // Use HOME environment variable if set, otherwise use relative path
        let home_dir = std::env::var("HOME").ok().map(PathBuf::from);
        self.save_to_home(home_dir.as_deref())
    }

    fn save_to_home(&self, home_override: Option<&Path>) {
        let rate_limit_path = if let Some(home) = home_override {
            home.join(".costpilot").join("rate_limit.json")
        } else {
            PathBuf::from(".costpilot/rate_limit.json")
        };

        if let Some(parent) = rate_limit_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(
            &rate_limit_path,
            serde_json::to_string(self).unwrap_or_default(),
        );
    }

    fn is_blocked(&self) -> bool {
        if let Some(blocked_until) = self.blocked_until {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            let is_blocked = now < blocked_until;

            // internal rate limit check (no debug output in production)

            return is_blocked;
        }
        false
    }

    fn record_attempt(&mut self) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // Reset if outside window
        if now - self.last_attempt > Self::WINDOW_SECS {
            self.attempts = 0;
        }

        self.attempts += 1;
        self.last_attempt = now;

        // Block if exceeded limit
        if self.attempts >= Self::MAX_ATTEMPTS {
            self.blocked_until = Some(now + Self::BLOCK_SECS);
        }

        // Recompute HMAC after state change
        self.compute_hmac();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct License {
    pub email: String,
    pub license_key: String,
    pub expires: String,
    pub signature: String,
    pub issuer: String,
}

impl License {
    /// Load license from JSON file
    pub fn load_from_file(path: &Path) -> Result<Self, String> {
        let content =
            std::fs::read_to_string(path).map_err(|e| format!("Failed to read license: {}", e))?;

        let value: serde_json::Value =
            serde_json::from_str(&content).map_err(|e| format!("Invalid license format: {}", e))?;

        let email = value["email"].as_str().unwrap_or("").to_string();
        let license_key = value["license_key"].as_str().unwrap_or("").to_string();
        let expires = value["expires"].as_str().unwrap_or("").to_string();
        let signature = value["signature"].as_str().unwrap_or("").to_string();
        let issuer = value["issuer"].as_str().unwrap_or("").to_string();

        // license loaded (sensitive content omitted from logs)

        if email.is_empty() {
            return Err("Missing required field: email".to_string());
        }
        if license_key.is_empty() {
            return Err("Missing required field: license_key".to_string());
        }
        if expires.is_empty() {
            return Err("Missing required field: expires".to_string());
        }
        if signature.is_empty() {
            return Err("Missing required field: signature".to_string());
        }
        if issuer.is_empty() {
            return Err("Missing required field: issuer".to_string());
        }

        Ok(License {
            email,
            license_key,
            expires,
            signature,
            issuer,
        })
    }

    /// Check if license is expired
    pub fn is_expired(&self) -> bool {
        // Parse ISO 8601 date
        match chrono::DateTime::parse_from_rfc3339(&self.expires) {
            Ok(expiry) => expiry < chrono::Utc::now(),
            Err(_) => true, // Invalid date = expired
        }
    }

    /// Validate license structure and cryptographic signature
    pub fn validate(&self) -> Result<(), String> {
        let mut rate_limit = RateLimitState::load();

        // Check if currently blocked
        if rate_limit.is_blocked() {
            return Err("Rate limit exceeded. Try again later".to_string());
        }

        // Record this attempt
        rate_limit.record_attempt();
        rate_limit.save();

        if self.email.is_empty() {
            return Err("Email is empty".to_string());
        }
        if self.license_key.is_empty() {
            return Err("License key is empty".to_string());
        }
        if self.signature.is_empty() {
            return Err("Signature is empty".to_string());
        }
        if self.issuer.is_empty() {
            return Err("Issuer is empty".to_string());
        }
        if self.is_expired() {
            return Err("License expired".to_string());
        }

        // Verify cryptographic signature
        #[cfg(not(target_arch = "wasm32"))]
        {
            use crate::pro_engine::crypto;
            crypto::verify_license_signature(self)?;
        }

        Ok(())
    }

    pub fn verify_signature(
        &self,
        bundle: &EncryptedBundle,
        public_key: &[u8],
    ) -> Result<(), LoaderError> {
        crate::pro_engine::loader::verify_signature(bundle, public_key)
    }
}
