// Policy version tracking and increment logic

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::Path;

/// Policy version metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyVersion {
    /// Semantic version (e.g., "1.0.0", "1.2.5")
    pub version: String,

    /// SHA-256 hash of policy content for change detection
    pub content_hash: String,

    /// Timestamp when version was created
    pub created_at: String,

    /// Optional description of changes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub changelog: Option<String>,
}

/// Policy version manager
pub struct PolicyVersionManager {
    version_file_path: std::path::PathBuf,
}

impl PolicyVersionManager {
    /// Create new version manager
    pub fn new<P: AsRef<Path>>(version_file: P) -> Self {
        Self {
            version_file_path: version_file.as_ref().to_path_buf(),
        }
    }

    /// Calculate SHA-256 hash of policy content
    pub fn calculate_hash(content: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Load current policy version from file
    pub fn load_version(&self) -> Result<PolicyVersion, std::io::Error> {
        let content = std::fs::read_to_string(&self.version_file_path)?;
        serde_json::from_str(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }

    /// Save policy version to file
    pub fn save_version(&self, version: &PolicyVersion) -> Result<(), std::io::Error> {
        let json = serde_json::to_string_pretty(version)?;
        std::fs::write(&self.version_file_path, json)
    }

    /// Check if policy content has changed
    pub fn has_changed(&self, current_content: &str) -> Result<bool, std::io::Error> {
        let current_hash = Self::calculate_hash(current_content);

        match self.load_version() {
            Ok(stored_version) => Ok(stored_version.content_hash != current_hash),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                // No version file exists yet, consider it changed
                Ok(true)
            }
            Err(e) => Err(e),
        }
    }

    /// Increment policy version
    pub fn increment_version(
        &self,
        current_content: &str,
        changelog: Option<String>,
    ) -> Result<PolicyVersion, std::io::Error> {
        let new_hash = Self::calculate_hash(current_content);
        let timestamp = chrono::Utc::now().to_rfc3339();

        let new_version = match self.load_version() {
            Ok(stored_version) => {
                // Parse and increment version
                let incremented = Self::increment_semver(&stored_version.version);

                PolicyVersion {
                    version: incremented,
                    content_hash: new_hash,
                    created_at: timestamp,
                    changelog,
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                // First version
                PolicyVersion {
                    version: "1.0.0".to_string(),
                    content_hash: new_hash,
                    created_at: timestamp,
                    changelog,
                }
            }
            Err(e) => return Err(e),
        };

        self.save_version(&new_version)?;
        Ok(new_version)
    }

    /// Increment semantic version (patch version)
    fn increment_semver(version: &str) -> String {
        let parts: Vec<&str> = version.split('.').collect();

        if parts.len() == 3 {
            if let (Ok(major), Ok(minor), Ok(patch)) = (
                parts[0].parse::<u32>(),
                parts[1].parse::<u32>(),
                parts[2].parse::<u32>(),
            ) {
                return format!("{}.{}.{}", major, minor, patch + 1);
            }
        }

        // Fallback: append .1 or increment last number
        if let Some(last_dot) = version.rfind('.') {
            let prefix = &version[..last_dot];
            let suffix = &version[last_dot + 1..];

            if let Ok(num) = suffix.parse::<u32>() {
                return format!("{}.{}", prefix, num + 1);
            }
        }

        // Last resort
        format!("{}.1", version)
    }

    /// Require version increment when policy changes
    pub fn require_increment_on_change(
        &self,
        current_content: &str,
    ) -> Result<Option<String>, std::io::Error> {
        if self.has_changed(current_content)? {
            let current_version = self
                .load_version()
                .map(|v| v.version)
                .unwrap_or_else(|_| "0.0.0".to_string());

            Ok(Some(format!(
                "Policy content has changed. Version must be incremented from {}. Run: costpilot policy increment",
                current_version
            )))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_calculate_hash() {
        let content = "test policy content";
        let hash1 = PolicyVersionManager::calculate_hash(content);
        let hash2 = PolicyVersionManager::calculate_hash(content);

        // Same content produces same hash
        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 64); // SHA-256 produces 64 hex chars

        // Different content produces different hash
        let hash3 = PolicyVersionManager::calculate_hash("different content");
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_increment_semver() {
        assert_eq!(PolicyVersionManager::increment_semver("1.0.0"), "1.0.1");
        assert_eq!(PolicyVersionManager::increment_semver("1.2.5"), "1.2.6");
        assert_eq!(
            PolicyVersionManager::increment_semver("2.10.99"),
            "2.10.100"
        );
        assert_eq!(PolicyVersionManager::increment_semver("1.0"), "1.1");
    }

    #[test]
    fn test_version_lifecycle() {
        let temp_dir = TempDir::new().unwrap();
        let version_file = temp_dir.path().join("policy_version.json");

        let manager = PolicyVersionManager::new(&version_file);

        // First increment creates version 1.0.0
        let content1 = "initial policy content";
        let version1 = manager
            .increment_version(content1, Some("Initial version".to_string()))
            .unwrap();

        assert_eq!(version1.version, "1.0.0");
        assert_eq!(
            version1.content_hash,
            PolicyVersionManager::calculate_hash(content1)
        );
        assert_eq!(version1.changelog, Some("Initial version".to_string()));

        // No change detected
        assert!(!manager.has_changed(content1).unwrap());

        // Changed content detected
        let content2 = "modified policy content";
        assert!(manager.has_changed(content2).unwrap());

        // Second increment creates version 1.0.1
        let version2 = manager
            .increment_version(content2, Some("Updated rules".to_string()))
            .unwrap();

        assert_eq!(version2.version, "1.0.1");
        assert_eq!(
            version2.content_hash,
            PolicyVersionManager::calculate_hash(content2)
        );
        assert_eq!(version2.changelog, Some("Updated rules".to_string()));
    }

    #[test]
    fn test_require_increment_on_change() {
        let temp_dir = TempDir::new().unwrap();
        let version_file = temp_dir.path().join("policy_version.json");

        let manager = PolicyVersionManager::new(&version_file);

        let content1 = "policy v1";
        manager.increment_version(content1, None).unwrap();

        // No change, no error
        let result = manager.require_increment_on_change(content1).unwrap();
        assert!(result.is_none());

        // Changed, requires increment
        let content2 = "policy v2";
        let result = manager.require_increment_on_change(content2).unwrap();
        assert!(result.is_some());
        assert!(result
            .unwrap()
            .contains("Version must be incremented from 1.0.0"));
    }

    #[test]
    fn test_load_and_save_version() {
        let temp_dir = TempDir::new().unwrap();
        let version_file = temp_dir.path().join("policy_version.json");

        let manager = PolicyVersionManager::new(&version_file);

        let version = PolicyVersion {
            version: "2.5.3".to_string(),
            content_hash: "abc123".to_string(),
            created_at: "2024-01-01T00:00:00Z".to_string(),
            changelog: Some("Test version".to_string()),
        };

        // Save
        manager.save_version(&version).unwrap();

        // Load
        let loaded = manager.load_version().unwrap();

        assert_eq!(loaded.version, version.version);
        assert_eq!(loaded.content_hash, version.content_hash);
        assert_eq!(loaded.created_at, version.created_at);
        assert_eq!(loaded.changelog, version.changelog);
    }
}
