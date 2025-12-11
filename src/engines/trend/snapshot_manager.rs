use chrono::{Duration, Utc};
use serde_json;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

use super::snapshot_types::{CostSnapshot, TrendConfig, TrendHistory};
use crate::errors::CostPilotError;

/// Manages snapshot storage and rotation
pub struct SnapshotManager {
    storage_dir: PathBuf,
    config: TrendConfig,
}

impl SnapshotManager {
    /// Create a new snapshot manager
    pub fn new<P: AsRef<Path>>(storage_dir: P) -> Self {
        Self {
            storage_dir: storage_dir.as_ref().to_path_buf(),
            config: TrendConfig::default(),
        }
    }

    /// Create a new snapshot manager with custom config
    pub fn with_config<P: AsRef<Path>>(storage_dir: P, config: TrendConfig) -> Self {
        Self {
            storage_dir: storage_dir.as_ref().to_path_buf(),
            config,
        }
    }

    /// Initialize storage directory
    pub fn init(&self) -> Result<(), CostPilotError> {
        if !self.storage_dir.exists() {
            fs::create_dir_all(&self.storage_dir).map_err(|e| {
                CostPilotError::io_error(format!("Failed to create storage directory: {}", e))
            })?;
        }
        Ok(())
    }

    /// Write a snapshot to storage
    pub fn write_snapshot(&self, snapshot: &CostSnapshot) -> Result<PathBuf, CostPilotError> {
        self.init()?;

        // Validate snapshot
        self.validate_snapshot(snapshot)?;

        // Generate filename: snapshot_{id}.json
        let filename = format!("snapshot_{}.json", snapshot.id);
        let filepath = self.storage_dir.join(&filename);

        // Serialize to pretty JSON
        let json = serde_json::to_string_pretty(snapshot).map_err(|e| {
            CostPilotError::serialization_error(format!("Failed to serialize snapshot: {}", e))
        })?;

        // Write to file
        let mut file = File::create(&filepath).map_err(|e| {
            CostPilotError::io_error(format!("Failed to create snapshot file: {}", e))
        })?;

        file.write_all(json.as_bytes())
            .map_err(|e| CostPilotError::io_error(format!("Failed to write snapshot: {}", e)))?;

        Ok(filepath)
    }

    /// Read a snapshot from storage
    pub fn read_snapshot(&self, id: &str) -> Result<CostSnapshot, CostPilotError> {
        let filename = format!("snapshot_{}.json", id);
        let filepath = self.storage_dir.join(&filename);

        if !filepath.exists() {
            return Err(CostPilotError::file_not_found(
                filepath.to_string_lossy().to_string(),
            ));
        }

        let contents = fs::read_to_string(&filepath)
            .map_err(|e| CostPilotError::io_error(format!("Failed to read snapshot: {}", e)))?;

        let snapshot: CostSnapshot = serde_json::from_str(&contents)
            .map_err(|e| CostPilotError::parse_error(format!("Failed to parse snapshot: {}", e)))?;

        Ok(snapshot)
    }

    /// Load all snapshots from storage
    pub fn load_history(&self) -> Result<TrendHistory, CostPilotError> {
        let mut history = TrendHistory::new();
        history.config = Some(self.config.clone());

        if !self.storage_dir.exists() {
            return Ok(history);
        }

        // Read all snapshot files
        let entries = fs::read_dir(&self.storage_dir).map_err(|e| {
            CostPilotError::io_error(format!("Failed to read storage directory: {}", e))
        })?;

        for entry in entries {
            let entry = entry.map_err(|e| {
                CostPilotError::io_error(format!("Failed to read directory entry: {}", e))
            })?;

            let path = entry.path();
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("json") {
                let filename = path.file_name().unwrap().to_string_lossy();
                if filename.starts_with("snapshot_") {
                    // Extract ID from filename
                    let id = filename
                        .trim_start_matches("snapshot_")
                        .trim_end_matches(".json");

                    match self.read_snapshot(id) {
                        Ok(snapshot) => history.add_snapshot(snapshot),
                        Err(e) => {
                            eprintln!("Warning: Failed to load snapshot {}: {}", id, e);
                        }
                    }
                }
            }
        }

        Ok(history)
    }

    /// Rotate snapshots based on retention policy
    pub fn rotate_snapshots(&self) -> Result<usize, CostPilotError> {
        if !self.config.enable_rotation {
            return Ok(0);
        }

        let history = self.load_history()?;
        let now = Utc::now();
        let cutoff = now - Duration::days(self.config.retention_days as i64);
        let mut deleted_count = 0;

        // Find snapshots to delete (older than retention period)
        let mut to_delete = Vec::new();
        for snapshot in &history.snapshots {
            if let Ok(timestamp) = snapshot.get_timestamp() {
                if timestamp < cutoff {
                    to_delete.push(snapshot.id.clone());
                }
            }
        }

        // Also enforce max_snapshots limit
        if history.snapshots.len() > self.config.max_snapshots {
            let excess = history.snapshots.len() - self.config.max_snapshots;
            for i in 0..excess {
                if i < history.snapshots.len() {
                    to_delete.push(history.snapshots[i].id.clone());
                }
            }
        }

        // Delete old snapshots
        for id in to_delete {
            if self.delete_snapshot(&id).is_ok() {
                deleted_count += 1;
            }
        }

        Ok(deleted_count)
    }

    /// Delete a snapshot from storage
    pub fn delete_snapshot(&self, id: &str) -> Result<(), CostPilotError> {
        let filename = format!("snapshot_{}.json", id);
        let filepath = self.storage_dir.join(&filename);

        if filepath.exists() {
            fs::remove_file(&filepath).map_err(|e| {
                CostPilotError::io_error(format!("Failed to delete snapshot: {}", e))
            })?;
        }

        Ok(())
    }

    /// Validate snapshot structure
    fn validate_snapshot(&self, snapshot: &CostSnapshot) -> Result<(), CostPilotError> {
        // Check ID is not empty
        if snapshot.id.is_empty() {
            return Err(CostPilotError::validation_error(
                "Snapshot ID cannot be empty".to_string(),
            ));
        }

        // Validate timestamp format
        snapshot.get_timestamp().map_err(|e| {
            CostPilotError::validation_error(format!("Invalid timestamp format: {}", e))
        })?;

        // Check cost is non-negative
        if snapshot.total_monthly_cost < 0.0 {
            return Err(CostPilotError::validation_error(
                "Total monthly cost cannot be negative".to_string(),
            ));
        }

        // Validate module costs
        for (name, module) in &snapshot.modules {
            if module.monthly_cost < 0.0 {
                return Err(CostPilotError::validation_error(format!(
                    "Module '{}' has negative cost",
                    name
                )));
            }
        }

        // Validate service costs
        for (service, cost) in &snapshot.services {
            if *cost < 0.0 {
                return Err(CostPilotError::validation_error(format!(
                    "Service '{}' has negative cost",
                    service
                )));
            }
        }

        Ok(())
    }

    /// Generate a unique snapshot ID
    pub fn generate_snapshot_id() -> String {
        let now = Utc::now();
        format!(
            "{}-{}",
            now.format("%Y%m%d-%H%M%S"),
            now.timestamp_millis() % 10000
        )
    }

    /// Check if snapshot exists
    pub fn snapshot_exists(&self, id: &str) -> bool {
        let filename = format!("snapshot_{}.json", id);
        let filepath = self.storage_dir.join(&filename);
        filepath.exists()
    }

    /// Get snapshot count
    pub fn count_snapshots(&self) -> Result<usize, CostPilotError> {
        let history = self.load_history()?;
        Ok(history.snapshots.len())
    }

    /// Detect corrupted snapshots
    pub fn detect_corruption(&self) -> Result<Vec<String>, CostPilotError> {
        let mut corrupted = Vec::new();

        if !self.storage_dir.exists() {
            return Ok(corrupted);
        }

        let entries = fs::read_dir(&self.storage_dir).map_err(|e| {
            CostPilotError::io_error(format!("Failed to read storage directory: {}", e))
        })?;

        for entry in entries {
            let entry = entry.map_err(|e| {
                CostPilotError::io_error(format!("Failed to read directory entry: {}", e))
            })?;

            let path = entry.path();
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("json") {
                let filename = path.file_name().unwrap().to_string_lossy();
                if filename.starts_with("snapshot_") {
                    let id = filename
                        .trim_start_matches("snapshot_")
                        .trim_end_matches(".json");

                    // Try to read and validate
                    if self.read_snapshot(id).is_err() {
                        corrupted.push(id.to_string());
                    }
                }
            }
        }

        Ok(corrupted)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_snapshot_manager_init() {
        let temp_dir = TempDir::new().unwrap();
        let manager = SnapshotManager::new(temp_dir.path());

        assert!(manager.init().is_ok());
        assert!(temp_dir.path().exists());
    }

    #[test]
    fn test_write_and_read_snapshot() {
        let temp_dir = TempDir::new().unwrap();
        let manager = SnapshotManager::new(temp_dir.path());

        let snapshot = CostSnapshot::new("test-001".to_string(), 1234.56);

        let write_result = manager.write_snapshot(&snapshot);
        assert!(write_result.is_ok());

        let read_result = manager.read_snapshot("test-001");
        assert!(read_result.is_ok());

        let loaded = read_result.unwrap();
        assert_eq!(loaded.id, "test-001");
        assert_eq!(loaded.total_monthly_cost, 1234.56);
    }

    #[test]
    fn test_load_history() {
        let temp_dir = TempDir::new().unwrap();
        let manager = SnapshotManager::new(temp_dir.path());

        let snapshot1 = CostSnapshot::new("test-001".to_string(), 1000.0);
        let snapshot2 = CostSnapshot::new("test-002".to_string(), 1200.0);

        manager.write_snapshot(&snapshot1).unwrap();
        manager.write_snapshot(&snapshot2).unwrap();

        let history = manager.load_history().unwrap();
        assert_eq!(history.snapshots.len(), 2);
    }

    #[test]
    fn test_generate_snapshot_id() {
        let id1 = SnapshotManager::generate_snapshot_id();
        let id2 = SnapshotManager::generate_snapshot_id();

        assert!(!id1.is_empty());
        assert!(!id2.is_empty());
        // IDs should be different (timestamp-based)
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_snapshot_exists() {
        let temp_dir = TempDir::new().unwrap();
        let manager = SnapshotManager::new(temp_dir.path());

        let snapshot = CostSnapshot::new("test-001".to_string(), 1234.56);
        manager.write_snapshot(&snapshot).unwrap();

        assert!(manager.snapshot_exists("test-001"));
        assert!(!manager.snapshot_exists("nonexistent"));
    }

    #[test]
    fn test_validate_snapshot_negative_cost() {
        let temp_dir = TempDir::new().unwrap();
        let manager = SnapshotManager::new(temp_dir.path());

        let mut snapshot = CostSnapshot::new("test-001".to_string(), -100.0);

        let result = manager.validate_snapshot(&snapshot);
        assert!(result.is_err());
    }

    #[test]
    fn test_delete_snapshot() {
        let temp_dir = TempDir::new().unwrap();
        let manager = SnapshotManager::new(temp_dir.path());

        let snapshot = CostSnapshot::new("test-001".to_string(), 1234.56);
        manager.write_snapshot(&snapshot).unwrap();

        assert!(manager.snapshot_exists("test-001"));

        manager.delete_snapshot("test-001").unwrap();
        assert!(!manager.snapshot_exists("test-001"));
    }

    #[test]
    fn test_count_snapshots() {
        let temp_dir = TempDir::new().unwrap();
        let manager = SnapshotManager::new(temp_dir.path());

        assert_eq!(manager.count_snapshots().unwrap(), 0);

        let snapshot1 = CostSnapshot::new("test-001".to_string(), 1000.0);
        let snapshot2 = CostSnapshot::new("test-002".to_string(), 1200.0);

        manager.write_snapshot(&snapshot1).unwrap();
        manager.write_snapshot(&snapshot2).unwrap();

        assert_eq!(manager.count_snapshots().unwrap(), 2);
    }
}
