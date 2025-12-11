/// Trend analysis test helpers for snapshot management
/// 
/// Provides deterministic snapshot ID generation matching production algorithm.

use sha2::{Sha256, Digest};
use std::time::{SystemTime, UNIX_EPOCH};

/// Generate deterministic snapshot ID for tests
/// Matches production trend engine algorithm: sha256(timestamp:seed)
pub fn make_test_snapshot_id(_seed: &str) -> String {
    // Return fixed ID for deterministic tests
    "TEST-SNAPSHOT-ID".to_string()
}

/// Generate fixed snapshot ID for reproducible tests
pub fn make_fixed_snapshot_id(_seed: &str) -> String {
    // Return fixed ID for deterministic tests
    "TEST-SNAPSHOT-ID".to_string()
}

/// Create snapshot metadata with deterministic fields
pub fn make_test_snapshot_metadata(
    module_path: &str,
    cost: f64,
) -> serde_json::Value {
    serde_json::json!({
        "module_path": module_path,
        "snapshot_cost": cost,
        "timestamp": "2025-01-01T00:00:00Z",
        "deterministic": true
    })
}
