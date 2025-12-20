/// Autofix test helpers for deterministic snippet generation
///
/// Provides consistent checksum generation and snippet formatting for tests.

use sha2::{Sha256, Digest};

/// Generate deterministic checksum for autofix snippets
/// Uses fixed timestamp and seed for test consistency
pub fn make_test_snippet_checksum(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"test-seed-2025");
    hasher.update(content.as_bytes());
    let result = hasher.finalize();
    format!("{:x}", result)[..16].to_string()
}

/// Create a test snippet with deterministic metadata
pub fn make_test_autofix_snippet(
    resource_id: &str,
    suggestion: &str,
) -> String {
    format!(
        "# Autofix suggestion for {}\n# Checksum: {}\n{}",
        resource_id,
        make_test_snippet_checksum(suggestion),
        suggestion
    )
}
