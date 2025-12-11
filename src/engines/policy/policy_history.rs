// Policy versioning and history tracking

use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Policy version with complete history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyVersion {
    /// Version number (semantic versioning)
    pub version: String,

    /// Policy content snapshot
    pub content: PolicyContent,

    /// Version metadata
    pub metadata: VersionMetadata,

    /// Who created this version
    pub author: String,

    /// When created
    pub created_at: String,

    /// Change description
    pub change_description: String,

    /// Parent version (if any)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_version: Option<String>,
}

/// Policy content snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyContent {
    /// Policy ID
    pub id: String,

    /// Policy name
    pub name: String,

    /// Policy description
    pub description: String,

    /// Policy rules (stored as JSON)
    pub rules: Value,

    /// Policy configuration
    pub config: HashMap<String, Value>,
}

/// Version metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionMetadata {
    /// Checksum of content (SHA-256)
    pub checksum: String,

    /// Size in bytes
    pub size_bytes: usize,

    /// Whether this is a major version change
    pub is_major: bool,

    /// Tags for this version
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub tags: Vec<String>,
}

/// Policy history manager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyHistory {
    /// Policy ID
    pub policy_id: String,

    /// All versions (ordered by creation time)
    pub versions: Vec<PolicyVersion>,

    /// Current active version
    pub current_version: String,
}

impl PolicyHistory {
    /// Create new policy history
    pub fn new(policy_id: String, initial_content: PolicyContent, author: String) -> Self {
        let version = Self::generate_version_number(None, false);
        let checksum = Self::calculate_checksum(&initial_content);

        let initial_version = PolicyVersion {
            version: version.clone(),
            content: initial_content.clone(),
            metadata: VersionMetadata {
                checksum,
                size_bytes: serde_json::to_string(&initial_content).unwrap().len(),
                is_major: true,
                tags: vec!["initial".to_string()],
            },
            author,
            created_at: Utc::now().to_rfc3339(),
            change_description: "Initial policy version".to_string(),
            parent_version: None,
        };

        Self {
            policy_id,
            versions: vec![initial_version],
            current_version: version,
        }
    }

    /// Add a new version
    pub fn add_version(
        &mut self,
        content: PolicyContent,
        author: String,
        change_description: String,
        is_major: bool,
    ) -> Result<String, HistoryError> {
        // Verify content ID matches
        if content.id != self.policy_id {
            return Err(HistoryError::PolicyIdMismatch {
                expected: self.policy_id.clone(),
                got: content.id.clone(),
            });
        }

        // Check if content has actually changed
        let new_checksum = Self::calculate_checksum(&content);
        if let Some(current) = self.get_version(&self.current_version) {
            if current.metadata.checksum == new_checksum {
                return Err(HistoryError::NoChanges);
            }
        }

        // Generate new version number
        let new_version = Self::generate_version_number(Some(&self.current_version), is_major);
        let content_size = serde_json::to_string(&content).unwrap().len();

        let version = PolicyVersion {
            version: new_version.clone(),
            content,
            metadata: VersionMetadata {
                checksum: new_checksum.clone(),
                size_bytes: content_size,
                is_major,
                tags: Vec::new(),
            },
            author,
            created_at: Utc::now().to_rfc3339(),
            change_description,
            parent_version: Some(self.current_version.clone()),
        };

        self.versions.push(version);
        self.current_version = new_version.clone();

        Ok(new_version)
    }

    /// Get a specific version
    pub fn get_version(&self, version: &str) -> Option<&PolicyVersion> {
        self.versions.iter().find(|v| v.version == version)
    }

    /// Get current version
    pub fn get_current(&self) -> Option<&PolicyVersion> {
        self.get_version(&self.current_version)
    }

    /// Get all versions
    pub fn get_all_versions(&self) -> &[PolicyVersion] {
        &self.versions
    }

    /// Rollback to a previous version
    pub fn rollback(
        &mut self,
        target_version: String,
        actor: String,
        reason: String,
    ) -> Result<String, HistoryError> {
        // Verify target version exists
        let target =
            self.get_version(&target_version)
                .ok_or_else(|| HistoryError::VersionNotFound {
                    version: target_version.clone(),
                })?;

        // Create a new version based on the target content
        let rollback_content = target.content.clone();
        let new_version = self.add_version(
            rollback_content,
            actor,
            format!("Rollback to {}: {}", target_version, reason),
            false,
        )?;

        Ok(new_version)
    }

    /// Get version diff
    pub fn diff(&self, from_version: &str, to_version: &str) -> Result<VersionDiff, HistoryError> {
        let from = self
            .get_version(from_version)
            .ok_or_else(|| HistoryError::VersionNotFound {
                version: from_version.to_string(),
            })?;

        let to = self
            .get_version(to_version)
            .ok_or_else(|| HistoryError::VersionNotFound {
                version: to_version.to_string(),
            })?;

        Ok(VersionDiff {
            from_version: from_version.to_string(),
            to_version: to_version.to_string(),
            name_changed: from.content.name != to.content.name,
            description_changed: from.content.description != to.content.description,
            rules_changed: from.content.rules != to.content.rules,
            config_changed: from.content.config != to.content.config,
            checksum_from: from.metadata.checksum.clone(),
            checksum_to: to.metadata.checksum.clone(),
            author_from: from.author.clone(),
            author_to: to.author.clone(),
            created_at_from: from.created_at.clone(),
            created_at_to: to.created_at.clone(),
        })
    }

    /// Tag a version
    pub fn tag_version(&mut self, version: &str, tag: String) -> Result<(), HistoryError> {
        let version_obj = self
            .versions
            .iter_mut()
            .find(|v| v.version == version)
            .ok_or_else(|| HistoryError::VersionNotFound {
                version: version.to_string(),
            })?;

        if !version_obj.metadata.tags.contains(&tag) {
            version_obj.metadata.tags.push(tag);
        }

        Ok(())
    }

    /// Get version count
    pub fn version_count(&self) -> usize {
        self.versions.len()
    }

    /// Get version by tag
    pub fn get_version_by_tag(&self, tag: &str) -> Option<&PolicyVersion> {
        self.versions
            .iter()
            .find(|v| v.metadata.tags.contains(&tag.to_string()))
    }

    /// Calculate content checksum
    fn calculate_checksum(content: &PolicyContent) -> String {
        use sha2::{Digest, Sha256};
        let json = serde_json::to_string(content).unwrap();
        let hash = Sha256::digest(json.as_bytes());
        format!("{:x}", hash)
    }

    /// Generate version number
    fn generate_version_number(current: Option<&str>, is_major: bool) -> String {
        match current {
            None => "1.0.0".to_string(),
            Some(version) => {
                let parts: Vec<&str> = version.split('.').collect();
                if parts.len() != 3 {
                    return "1.0.0".to_string();
                }

                let major: u32 = parts[0].parse().unwrap_or(0);
                let minor: u32 = parts[1].parse().unwrap_or(0);
                let patch: u32 = parts[2].parse().unwrap_or(0);

                if is_major {
                    format!("{}.0.0", major + 1)
                } else {
                    format!("{}.{}.{}", major, minor, patch + 1)
                }
            }
        }
    }
}

/// Version diff information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionDiff {
    pub from_version: String,
    pub to_version: String,
    pub name_changed: bool,
    pub description_changed: bool,
    pub rules_changed: bool,
    pub config_changed: bool,
    pub checksum_from: String,
    pub checksum_to: String,
    pub author_from: String,
    pub author_to: String,
    pub created_at_from: String,
    pub created_at_to: String,
}

impl VersionDiff {
    /// Check if any content changed
    pub fn has_changes(&self) -> bool {
        self.name_changed || self.description_changed || self.rules_changed || self.config_changed
    }

    /// Get summary of changes
    pub fn summary(&self) -> String {
        let mut changes = Vec::new();
        if self.name_changed {
            changes.push("name");
        }
        if self.description_changed {
            changes.push("description");
        }
        if self.rules_changed {
            changes.push("rules");
        }
        if self.config_changed {
            changes.push("config");
        }

        if changes.is_empty() {
            "No changes".to_string()
        } else {
            format!("Changed: {}", changes.join(", "))
        }
    }
}

/// History errors
#[derive(Debug, Clone, thiserror::Error)]
pub enum HistoryError {
    #[error("Policy ID mismatch: expected {expected}, got {got}")]
    PolicyIdMismatch { expected: String, got: String },

    #[error("No changes detected in policy content")]
    NoChanges,

    #[error("Version not found: {version}")]
    VersionNotFound { version: String },
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn create_test_content(id: &str, name: &str, rule_count: i32) -> PolicyContent {
        PolicyContent {
            id: id.to_string(),
            name: name.to_string(),
            description: "Test policy".to_string(),
            rules: json!({ "count": rule_count }),
            config: HashMap::new(),
        }
    }

    #[test]
    fn test_new_history() {
        let content = create_test_content("test-policy", "Test Policy", 1);
        let history = PolicyHistory::new(
            "test-policy".to_string(),
            content,
            "author@example.com".to_string(),
        );

        assert_eq!(history.version_count(), 1);
        assert_eq!(history.current_version, "1.0.0");
    }

    #[test]
    fn test_add_version() {
        let content = create_test_content("test-policy", "Test Policy", 1);
        let mut history = PolicyHistory::new(
            "test-policy".to_string(),
            content,
            "author@example.com".to_string(),
        );

        let new_content = create_test_content("test-policy", "Test Policy Updated", 2);
        let new_version = history
            .add_version(
                new_content,
                "author@example.com".to_string(),
                "Updated rules".to_string(),
                false,
            )
            .unwrap();

        assert_eq!(new_version, "1.0.1");
        assert_eq!(history.version_count(), 2);
        assert_eq!(history.current_version, "1.0.1");
    }

    #[test]
    fn test_major_version() {
        let content = create_test_content("test-policy", "Test Policy", 1);
        let mut history = PolicyHistory::new(
            "test-policy".to_string(),
            content,
            "author@example.com".to_string(),
        );

        let new_content = create_test_content("test-policy", "Test Policy v2", 2);
        let new_version = history
            .add_version(
                new_content,
                "author@example.com".to_string(),
                "Major rewrite".to_string(),
                true,
            )
            .unwrap();

        assert_eq!(new_version, "2.0.0");
    }

    #[test]
    fn test_no_changes_error() {
        let content = create_test_content("test-policy", "Test Policy", 1);
        let mut history = PolicyHistory::new(
            "test-policy".to_string(),
            content.clone(),
            "author@example.com".to_string(),
        );

        // Try to add same content again
        let result = history.add_version(
            content,
            "author@example.com".to_string(),
            "No changes".to_string(),
            false,
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_get_version() {
        let content = create_test_content("test-policy", "Test Policy", 1);
        let history = PolicyHistory::new(
            "test-policy".to_string(),
            content,
            "author@example.com".to_string(),
        );

        let version = history.get_version("1.0.0");
        assert!(version.is_some());
        assert_eq!(version.unwrap().version, "1.0.0");

        let missing = history.get_version("2.0.0");
        assert!(missing.is_none());
    }

    #[test]
    fn test_version_diff() {
        let content1 = create_test_content("test-policy", "Test Policy", 1);
        let mut history = PolicyHistory::new(
            "test-policy".to_string(),
            content1,
            "author@example.com".to_string(),
        );

        let content2 = create_test_content("test-policy", "Test Policy Updated", 2);
        history
            .add_version(
                content2,
                "author2@example.com".to_string(),
                "Updated".to_string(),
                false,
            )
            .unwrap();

        let diff = history.diff("1.0.0", "1.0.1").unwrap();
        assert!(diff.name_changed);
        assert!(diff.rules_changed);
        assert!(!diff.description_changed);
        assert!(diff.has_changes());
    }

    #[test]
    fn test_rollback() {
        let content1 = create_test_content("test-policy", "Version 1", 1);
        let mut history = PolicyHistory::new(
            "test-policy".to_string(),
            content1,
            "author@example.com".to_string(),
        );

        let content2 = create_test_content("test-policy", "Version 2", 2);
        history
            .add_version(
                content2,
                "author@example.com".to_string(),
                "v2".to_string(),
                false,
            )
            .unwrap();

        // Rollback to 1.0.0
        let rollback_version = history
            .rollback(
                "1.0.0".to_string(),
                "admin@example.com".to_string(),
                "Revert changes".to_string(),
            )
            .unwrap();

        assert_eq!(rollback_version, "1.0.2");
        assert_eq!(history.version_count(), 3);

        let current = history.get_current().unwrap();
        assert_eq!(current.content.name, "Version 1");
    }

    #[test]
    fn test_tag_version() {
        let content = create_test_content("test-policy", "Test Policy", 1);
        let mut history = PolicyHistory::new(
            "test-policy".to_string(),
            content,
            "author@example.com".to_string(),
        );

        history
            .tag_version("1.0.0", "production".to_string())
            .unwrap();

        let version = history.get_version("1.0.0").unwrap();
        assert!(version.metadata.tags.contains(&"production".to_string()));

        let tagged = history.get_version_by_tag("production");
        assert!(tagged.is_some());
        assert_eq!(tagged.unwrap().version, "1.0.0");
    }

    #[test]
    fn test_version_numbering() {
        let content = create_test_content("test-policy", "Test", 1);
        let mut history = PolicyHistory::new(
            "test-policy".to_string(),
            content.clone(),
            "author@example.com".to_string(),
        );

        // Patch version
        let v2 = history
            .add_version(
                create_test_content("test-policy", "Test v2", 2),
                "author".to_string(),
                "patch".to_string(),
                false,
            )
            .unwrap();
        assert_eq!(v2, "1.0.1");

        // Another patch
        let v3 = history
            .add_version(
                create_test_content("test-policy", "Test v3", 3),
                "author".to_string(),
                "patch".to_string(),
                false,
            )
            .unwrap();
        assert_eq!(v3, "1.0.2");

        // Major version
        let v4 = history
            .add_version(
                create_test_content("test-policy", "Test v4", 4),
                "author".to_string(),
                "major".to_string(),
                true,
            )
            .unwrap();
        assert_eq!(v4, "2.0.0");
    }
}
