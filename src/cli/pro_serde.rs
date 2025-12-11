// Serialization helpers for ProEngine JSON communication

use anyhow::{Context, Result};
use serde::{de::DeserializeOwned, Serialize};

/// Serialize value to JSON string for ProEngine
pub fn serialize<T: Serialize>(v: &T) -> Result<String> {
    serde_json::to_string(v).context("Failed to serialize to JSON")
}

/// Deserialize JSON string from ProEngine
pub fn deserialize<T: DeserializeOwned>(s: &str) -> Result<T> {
    serde_json::from_str(s).context("Failed to deserialize from JSON")
}
