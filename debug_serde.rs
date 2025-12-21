use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use tempfile::TempDir;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct License {
    pub key: String,
    pub email: String,
    pub expires: String,
    pub signature: String,
}

fn main() {
    let incomplete_json = r#"{"email": "test@example.com", "expires": "2025-12-31"}"#;

    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("license.json");
    fs::write(&file_path, incomplete_json).unwrap();

    let content = fs::read_to_string(&file_path).unwrap();
    println!("Content: {}", content);

    let result: Result<License, serde_json::Error> = serde_json::from_str(&content);
    println!("Result: {:?}", result);
}
