// License management for CostPilot Premium

use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct License {
    pub email: String,
    pub license_key: String,
    pub expires: String,
    pub signature: String,
}

impl License {
    /// Load license from JSON file
    pub fn load_from_file(path: &Path) -> Result<Self, String> {
        let content =
            std::fs::read_to_string(path).map_err(|e| format!("Failed to read license: {}", e))?;

        let license: License =
            serde_json::from_str(&content).map_err(|e| format!("Invalid license format: {}", e))?;

        Ok(license)
    }

    /// Check if license is expired
    pub fn is_expired(&self) -> bool {
        // Parse ISO 8601 date
        match chrono::DateTime::parse_from_rfc3339(&self.expires) {
            Ok(expiry) => expiry < chrono::Utc::now(),
            Err(_) => true, // Invalid date = expired
        }
    }

    /// Validate license structure
    pub fn validate(&self) -> Result<(), String> {
        if self.email.is_empty() {
            return Err("Email is empty".to_string());
        }
        if self.license_key.is_empty() {
            return Err("License key is empty".to_string());
        }
        if self.signature.is_empty() {
            return Err("Signature is empty".to_string());
        }
        if self.is_expired() {
            return Err("License expired".to_string());
        }
        Ok(())
    }
}
