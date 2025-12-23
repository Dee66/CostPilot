// Runtime feature flags for test-in-production capabilities

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;

/// Feature flag configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureFlags {
    /// Global enable/disable for all feature flags
    pub enabled: bool,
    /// Individual feature flags
    pub flags: HashMap<String, FeatureFlag>,
    /// Canary deployment settings
    pub canary: CanaryConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureFlag {
    /// Whether this feature is enabled
    pub enabled: bool,
    /// Rollout percentage (0.0 to 1.0)
    pub rollout_percentage: f64,
    /// User allowlist (if specified, only these users get the feature)
    pub allowlist: Option<Vec<String>>,
    /// User blocklist
    pub blocklist: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanaryConfig {
    /// Current canary version
    pub version: String,
    /// Percentage of users in canary
    pub percentage: f64,
    /// User seed for deterministic rollout
    pub seed: String,
}

impl Default for FeatureFlags {
    fn default() -> Self {
        let mut flags = HashMap::new();

        // Default feature flags
        flags.insert(
            "experimental_prediction".to_string(),
            FeatureFlag {
                enabled: false,
                rollout_percentage: 0.1, // 10% rollout
                allowlist: None,
                blocklist: None,
            },
        );

        flags.insert(
            "advanced_analytics".to_string(),
            FeatureFlag {
                enabled: false,
                rollout_percentage: 0.05, // 5% rollout
                allowlist: None,
                blocklist: None,
            },
        );

        flags.insert(
            "ai_enhancements".to_string(),
            FeatureFlag {
                enabled: false,
                rollout_percentage: 0.0, // Disabled by default
                allowlist: None,
                blocklist: None,
            },
        );

        Self {
            enabled: true,
            flags,
            canary: CanaryConfig {
                version: env!("CARGO_PKG_VERSION").to_string(),
                percentage: 0.1, // 10% canary
                seed: "costpilot-canary-2024".to_string(),
            },
        }
    }
}

impl FeatureFlags {
    /// Load feature flags from file or environment
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        // Try to load from file first
        let config_path = Self::config_path();
        if config_path.exists() {
            let contents = fs::read_to_string(&config_path)?;
            let mut flags: FeatureFlags = serde_yaml::from_str(&contents)?;

            // Override with environment variables
            flags.override_from_env();

            Ok(flags)
        } else {
            let mut flags = Self::default();
            flags.override_from_env();
            Ok(flags)
        }
    }

    /// Save feature flags to file
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config_path = Self::config_path();
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let yaml = serde_yaml::to_string(self)?;
        fs::write(config_path, yaml)?;
        Ok(())
    }

    /// Get config file path
    fn config_path() -> std::path::PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| Path::new(".").to_path_buf())
            .join("costpilot")
            .join("feature_flags.yml")
    }

    /// Override settings from environment variables
    fn override_from_env(&mut self) {
        if let Ok(enabled) = env::var("COSTPILOT_FEATURE_FLAGS_ENABLED") {
            self.enabled = enabled.parse().unwrap_or(true);
        }

        if let Ok(canary_pct) = env::var("COSTPILOT_CANARY_PERCENTAGE") {
            if let Ok(pct) = canary_pct.parse::<f64>() {
                self.canary.percentage = pct.clamp(0.0, 1.0);
            }
        }
    }

    /// Check if a feature is enabled for a user
    pub fn is_enabled(&self, feature: &str, user_id: Option<&str>) -> bool {
        if !self.enabled {
            return false;
        }

        let Some(flag) = self.flags.get(feature) else {
            return false;
        };

        if !flag.enabled {
            return false;
        }

        // Check blocklist first
        if let Some(blocklist) = &flag.blocklist {
            if let Some(user) = user_id {
                if blocklist.contains(&user.to_string()) {
                    return false;
                }
            }
        }

        // Check allowlist
        if let Some(allowlist) = &flag.allowlist {
            if let Some(user) = user_id {
                return allowlist.contains(&user.to_string());
            } else {
                return false; // Allowlist requires user_id
            }
        }

        // Check rollout percentage
        if flag.rollout_percentage >= 1.0 {
            return true;
        }

        if flag.rollout_percentage <= 0.0 {
            return false;
        }

        // Use user_id or machine ID for deterministic rollout
        let seed = user_id.unwrap_or(&self.canary.seed);
        let hash = Self::simple_hash(seed);
        let normalized = (hash % 10000) as f64 / 10000.0;

        normalized <= flag.rollout_percentage
    }

    /// Check if user is in canary deployment
    pub fn is_canary_user(&self, user_id: Option<&str>) -> bool {
        if self.canary.percentage >= 1.0 {
            return true;
        }

        if self.canary.percentage <= 0.0 {
            return false;
        }

        let seed = user_id.unwrap_or(&self.canary.seed);
        let hash = Self::simple_hash(seed);
        let normalized = (hash % 10000) as f64 / 10000.0;

        normalized <= self.canary.percentage
    }

    /// Simple hash function for deterministic rollout
    fn simple_hash(s: &str) -> u32 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        s.hash(&mut hasher);
        hasher.finish() as u32
    }

    /// Enable a feature for all users
    pub fn enable_feature(&mut self, feature: &str) {
        if let Some(flag) = self.flags.get_mut(feature) {
            flag.enabled = true;
            flag.rollout_percentage = 1.0;
        }
    }

    /// Disable a feature
    pub fn disable_feature(&mut self, feature: &str) {
        if let Some(flag) = self.flags.get_mut(feature) {
            flag.enabled = false;
        }
    }

    /// Set rollout percentage for a feature
    pub fn set_rollout_percentage(&mut self, feature: &str, percentage: f64) {
        if let Some(flag) = self.flags.get_mut(feature) {
            flag.rollout_percentage = percentage.clamp(0.0, 1.0);
        }
    }
}

/// Global feature flag manager
pub struct FeatureFlagManager {
    flags: FeatureFlags,
}

impl FeatureFlagManager {
    /// Create new manager
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            flags: FeatureFlags::load()?,
        })
    }

    /// Check if feature is enabled
    pub fn is_enabled(&self, feature: &str) -> bool {
        self.flags.is_enabled(feature, None)
    }

    /// Check if feature is enabled for user
    pub fn is_enabled_for_user(&self, feature: &str, user_id: &str) -> bool {
        self.flags.is_enabled(feature, Some(user_id))
    }

    /// Check if user is in canary
    pub fn is_canary_user(&self, user_id: Option<&str>) -> bool {
        self.flags.is_canary_user(user_id)
    }

    /// Get canary version
    pub fn canary_version(&self) -> &str {
        &self.flags.canary.version
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_feature_flags() {
        let flags = FeatureFlags::default();
        assert!(flags.enabled);
        assert!(flags.flags.contains_key("experimental_prediction"));
        assert!(flags.flags.contains_key("advanced_analytics"));
    }

    #[test]
    fn test_feature_disabled_by_default() {
        let flags = FeatureFlags::default();
        assert!(!flags.is_enabled("experimental_prediction", None));
    }

    #[test]
    fn test_canary_rollout() {
        let flags = FeatureFlags::default();
        // With 10% canary, some users should be in canary
        let mut canary_count = 0;
        for i in 0..100 {
            if flags.is_canary_user(Some(&format!("user{}", i))) {
                canary_count += 1;
            }
        }
        // Should be roughly 10 users in canary
        assert!(canary_count > 5 && canary_count < 20);
    }

    #[test]
    fn test_feature_rollout() {
        let mut flags = FeatureFlags::default();
        flags
            .flags
            .get_mut("experimental_prediction")
            .unwrap()
            .enabled = true;
        flags
            .flags
            .get_mut("experimental_prediction")
            .unwrap()
            .rollout_percentage = 0.5;

        let mut enabled_count = 0;
        for i in 0..100 {
            if flags.is_enabled("experimental_prediction", Some(&format!("user{}", i))) {
                enabled_count += 1;
            }
        }
        // Should be roughly 50 users with feature enabled
        assert!(enabled_count > 30 && enabled_count < 70);
    }

    #[test]
    fn test_allowlist() {
        let mut flags = FeatureFlags::default();
        flags
            .flags
            .get_mut("experimental_prediction")
            .unwrap()
            .enabled = true;
        flags
            .flags
            .get_mut("experimental_prediction")
            .unwrap()
            .allowlist = Some(vec!["user1".to_string()]);

        assert!(flags.is_enabled("experimental_prediction", Some("user1")));
        assert!(!flags.is_enabled("experimental_prediction", Some("user2")));
    }
}
