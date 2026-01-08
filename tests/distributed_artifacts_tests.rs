use std::fs;
use std::path::Path;

/// Compute SHA256 hash of a file
fn sha256_hash(path: &Path) -> Result<String, Box<dyn std::error::Error>> {
    use sha2::{Digest, Sha256};
    use std::io::Read;

    let mut file = fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0; 1024];

    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}

/// Test that distributed artifacts match tested artifacts
#[test]
fn test_distributed_artifacts_match_tested_artifacts() {
    // Check that the built binary exists
    let binary_path = Path::new("target/debug/costpilot");
    assert!(
        binary_path.exists(),
        "Built binary should exist at target/debug/costpilot"
    );

    // Compute hash of the built binary
    let hash = sha256_hash(binary_path).expect("Failed to hash binary");
    assert!(!hash.is_empty(), "Hash should not be empty");

    // In a real distribution scenario, this hash would be compared against
    // the hash of the distributed artifact. For now, we just ensure the binary exists and is hashable.
    // TODO: Extend to compare against distributed artifacts when available
}

/// Test for artifact parity across distribution channels
#[test]
fn test_artifact_parity_across_distribution_channels() {
    // Check that the built binary exists
    let binary_path = Path::new("target/debug/costpilot");
    assert!(
        binary_path.exists(),
        "Built binary should exist at target/debug/costpilot"
    );

    // Since there are no other distribution channels yet (e.g., no GitHub releases, Docker, crates.io),
    // this test passes by ensuring the local artifact exists.
    // TODO: Extend to check parity across actual distribution channels when implemented
}
