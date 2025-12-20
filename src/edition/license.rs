use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

/// Rate limiting state for license validation
#[derive(Debug, Clone, Serialize, Deserialize)]
struct RateLimitState {
    attempts: u32,
    last_attempt: u64,
    blocked_until: Option<u64>,
}

impl RateLimitState {
    const MAX_ATTEMPTS: u32 = 5;  // Max 5 attempts per minute
    const WINDOW_SECS: u64 = 60;  // 1 minute window
    const BLOCK_SECS: u64 = 300;  // 5 minute block after exceeding limit

    fn new() -> Self {
        Self {
            attempts: 0,
            last_attempt: 0,
            blocked_until: None,
        }
    }

    fn load() -> Self {
        let path = Path::new(".costpilot/rate_limit.json");
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(path) {
                if let Ok(state) = serde_json::from_str(&content) {
                    return state;
                }
            }
        }
        Self::new()
    }

    fn save(&self) {
        let path = Path::new(".costpilot/rate_limit.json");
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(path, serde_json::to_string(self).unwrap_or_default());
    }

    fn is_blocked(&self) -> bool {
        if let Some(blocked_until) = self.blocked_until {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            now < blocked_until
        } else {
            false
        }
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
    }
}

/// License information for Premium edition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct License {
    pub key: String,
    pub email: String,
    pub expires: String,
    pub signature: String,
}

impl License {
    /// Load license from JSON file
    pub fn load_from_file(path: &Path) -> Result<Option<Self>, LicenseError> {
        if !path.exists() {
            return Ok(None);
        }

        let content =
            std::fs::read_to_string(path).map_err(|e| LicenseError::IoError(e.to_string()))?;

        let license: License =
            serde_json::from_str(&content).map_err(|e| LicenseError::ParseError(e.to_string()))?;

        Ok(Some(license))
    }

    /// Verify license signature with rate limiting
    pub fn verify_signature(&self) -> bool {
        let mut rate_limit = RateLimitState::load();

        // Check if currently blocked
        if rate_limit.is_blocked() {
            return false;
        }

        // Record this attempt
        rate_limit.record_attempt();
        rate_limit.save();

        // Stub: return true for now (actual signature verification would go here)
        true
    }
}

/// License loading errors
#[derive(Debug)]
pub enum LicenseError {
    IoError(String),
    ParseError(String),
    InvalidSignature,
}

impl std::fmt::Display for LicenseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LicenseError::IoError(msg) => write!(f, "License I/O error: {}", msg),
            LicenseError::ParseError(msg) => write!(f, "License parse error: {}", msg),
            LicenseError::InvalidSignature => write!(f, "Invalid license signature"),
        }
    }
}

impl std::error::Error for LicenseError {}
