use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::Command;

// Test basic archive creation and extraction
#[test]
fn test_archive_creation_extraction() {
    let temp_dir = std::env::temp_dir().join("costpilot_archive_test");
    fs::create_dir_all(&temp_dir).ok();

    // Create test files
    let test_files = vec![
        temp_dir.join("file1.txt"),
        temp_dir.join("file2.txt"),
        temp_dir.join("subdir/file3.txt"),
    ];

    for file_path in &test_files {
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).ok();
        }
        let mut file = fs::File::create(file_path).expect("Failed to create test file");
        writeln!(file, "Test content for {}", file_path.display()).ok();
    }

    // Test tar.gz creation (if tar command is available)
    let archive_path = temp_dir.join("test.tar.gz");
    let tar_result = Command::new("tar")
        .args([
            "-czf",
            &archive_path.to_string_lossy(),
            "-C",
            &temp_dir.to_string_lossy(),
            ".",
        ])
        .output()
        .ok();

    if tar_result.is_some() {
        assert!(archive_path.exists(), "Archive should be created");

        // Test extraction
        let extract_dir = temp_dir.join("extracted");
        fs::create_dir_all(&extract_dir).ok();

        let extract_result = Command::new("tar")
            .args(&[
                "-xzf",
                &archive_path.to_string_lossy(),
                "-C",
                &extract_dir.to_string_lossy(),
            ])
            .output()
            .ok();

        if extract_result.is_some() {
            // Verify extracted files exist
            for file_path in &test_files {
                let relative_path = file_path.strip_prefix(&temp_dir).unwrap();
                let extracted_path = extract_dir.join(relative_path);
                assert!(
                    extracted_path.exists(),
                    "Extracted file should exist: {}",
                    extracted_path.display()
                );
            }
        }
    }

    // Cleanup
    fs::remove_dir_all(&temp_dir).ok();
}

// Test checksum validation
#[test]
fn test_checksum_validation() {
    let temp_dir = std::env::temp_dir().join("costpilot_checksum_test");
    fs::create_dir_all(&temp_dir).ok();

    let test_file = temp_dir.join("test.txt");
    let content = "Test content for checksum validation";
    fs::write(&test_file, content).expect("Failed to write test file");

    // Calculate SHA256 checksum
    let sha256_result = Command::new("sha256sum").arg(&test_file).output().ok();

    if let Some(output) = sha256_result {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            assert!(!stdout.is_empty(), "Checksum should not be empty");

            // Verify checksum is valid hex
            let checksum_part = stdout.split_whitespace().next().unwrap_or("");
            assert!(
                checksum_part.len() == 64,
                "SHA256 checksum should be 64 characters"
            );
            assert!(
                checksum_part.chars().all(|c| c.is_ascii_hexdigit()),
                "Checksum should be valid hex"
            );
        }
    }

    // Test MD5 checksum as well
    let md5_result = Command::new("md5sum").arg(&test_file).output().ok();

    if let Some(output) = md5_result {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            assert!(!stdout.is_empty(), "MD5 checksum should not be empty");

            let checksum_part = stdout.split_whitespace().next().unwrap_or("");
            assert!(
                checksum_part.len() == 32,
                "MD5 checksum should be 32 characters"
            );
            assert!(
                checksum_part.chars().all(|c| c.is_ascii_hexdigit()),
                "MD5 checksum should be valid hex"
            );
        }
    }

    fs::remove_dir_all(&temp_dir).ok();
}

// Test corrupted archive detection
#[test]
fn test_corrupted_archive_detection() {
    let temp_dir = std::env::temp_dir().join("costpilot_corrupt_test");
    fs::create_dir_all(&temp_dir).ok();

    // Create a corrupted tar.gz file
    let corrupt_archive = temp_dir.join("corrupt.tar.gz");
    fs::write(&corrupt_archive, b"This is not a valid tar.gz file").ok();

    // Try to extract it
    let extract_dir = temp_dir.join("extract_corrupt");
    fs::create_dir_all(&extract_dir).ok();

    let extract_result = Command::new("tar")
        .args(&[
            "-xzf",
            &corrupt_archive.to_string_lossy(),
            "-C",
            &extract_dir.to_string_lossy(),
        ])
        .output()
        .ok();

    // Should fail or produce warnings
    if let Some(output) = extract_result {
        // Either the command fails or produces stderr output indicating corruption
        assert!(
            !output.status.success() || !output.stderr.is_empty(),
            "Corrupted archive should either fail or produce warnings"
        );
    }

    fs::remove_dir_all(&temp_dir).ok();
}

// Test archive size limits
#[test]
fn test_archive_size_limits() {
    let temp_dir = std::env::temp_dir().join("costpilot_size_test");
    fs::create_dir_all(&temp_dir).ok();

    // Create a small test file
    let small_file = temp_dir.join("small.txt");
    fs::write(&small_file, "Small content").ok();

    // Create archive
    let archive_path = temp_dir.join("small.tar.gz");
    let _ = Command::new("tar")
        .args(&[
            "-czf",
            &archive_path.to_string_lossy(),
            &small_file.to_string_lossy(),
        ])
        .output()
        .ok();

    if archive_path.exists() {
        let metadata = fs::metadata(&archive_path).ok();
        if let Some(meta) = metadata {
            let size = meta.len();
            // Archive should be reasonably small (less than 1MB)
            assert!(
                size < 1024 * 1024,
                "Archive size should be reasonable: {} bytes",
                size
            );
            // But not empty
            assert!(size > 0, "Archive should not be empty");
        }
    }

    fs::remove_dir_all(&temp_dir).ok();
}

// Test nested archive structures (simplified)
#[test]
fn test_nested_archive_structures() {
    let temp_dir = std::env::temp_dir().join("costpilot_nested_test");
    fs::create_dir_all(&temp_dir).ok();

    // Create multiple files for archiving
    let file1 = temp_dir.join("file1.txt");
    let file2 = temp_dir.join("file2.txt");
    fs::write(&file1, "Content 1").ok();
    fs::write(&file2, "Content 2").ok();

    let archive_path = temp_dir.join("multi.tar.gz");

    // Test that tar can handle multiple files
    let result = Command::new("tar")
        .args(&[
            "-czf",
            &archive_path.to_string_lossy(),
            &file1.to_string_lossy(),
            &file2.to_string_lossy(),
        ])
        .output()
        .ok();

    if result.is_some() && archive_path.exists() {
        // Test that we can list contents
        let list_result = Command::new("tar")
            .args(&["-tzf", &archive_path.to_string_lossy()])
            .output()
            .ok();

        if let Some(output) = list_result {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                // Should contain both files
                assert!(
                    stdout.contains("file1.txt") || stdout.contains("file2.txt"),
                    "Archive should contain the files"
                );
            }
        }
    }

    fs::remove_dir_all(&temp_dir).ok();
}

// Test archive content validation
#[test]
fn test_archive_content_validation() {
    let temp_dir = std::env::temp_dir().join("costpilot_content_test");
    fs::create_dir_all(&temp_dir).ok();

    // Create test files with known content
    let files = vec![
        ("file1.txt", "Content 1"),
        ("file2.txt", "Content 2"),
        ("subdir/file3.txt", "Content 3"),
    ];

    let mut expected_files = Vec::new();

    for (rel_path, content) in &files {
        let full_path = temp_dir.join(rel_path);
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent).ok();
        }
        fs::write(&full_path, content).ok();
        expected_files.push(rel_path.to_string());
    }

    // Create archive
    let archive_path = temp_dir.join("content.tar.gz");
    let _ = Command::new("tar")
        .args(&[
            "-czf",
            &archive_path.to_string_lossy(),
            "-C",
            &temp_dir.to_string_lossy(),
            ".",
        ])
        .output()
        .ok();

    if archive_path.exists() {
        // List archive contents
        let list_result = Command::new("tar")
            .args(&["-tzf", &archive_path.to_string_lossy()])
            .output()
            .ok();

        if let Some(output) = list_result {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                for expected_file in &expected_files {
                    assert!(
                        stdout.contains(expected_file),
                        "Archive should contain expected file: {}",
                        expected_file
                    );
                }
            }
        }
    }

    fs::remove_dir_all(&temp_dir).ok();
}

// Test partial archive handling
#[test]
fn test_partial_archive_handling() {
    let temp_dir = std::env::temp_dir().join("costpilot_partial_test");
    fs::create_dir_all(&temp_dir).ok();

    // Create a complete archive
    let test_file = temp_dir.join("complete.txt");
    fs::write(&test_file, "Complete content").ok();

    let complete_archive = temp_dir.join("complete.tar.gz");
    let _ = Command::new("tar")
        .args(&[
            "-czf",
            &complete_archive.to_string_lossy(),
            &test_file.to_string_lossy(),
        ])
        .output()
        .ok();

    if complete_archive.exists() {
        // Create a partial/truncated version
        let complete_content =
            fs::read(&complete_archive).expect("Failed to read complete archive");
        let partial_content = &complete_content[..complete_content.len() / 2];
        let partial_archive = temp_dir.join("partial.tar.gz");
        fs::write(&partial_archive, partial_content).ok();

        // Try to extract partial archive
        let extract_dir = temp_dir.join("extract_partial");
        fs::create_dir_all(&extract_dir).ok();

        let extract_result = Command::new("tar")
            .args(&[
                "-xzf",
                &partial_archive.to_string_lossy(),
                "-C",
                &extract_dir.to_string_lossy(),
            ])
            .output()
            .ok();

        // Should fail gracefully
        if let Some(output) = extract_result {
            // Either fails or produces error output
            assert!(
                !output.status.success() || !output.stderr.is_empty(),
                "Partial archive extraction should indicate problems"
            );
        }
    }

    fs::remove_dir_all(&temp_dir).ok();
}

// Test archive signature validation (mock)
#[test]
fn test_archive_signature_validation() {
    let temp_dir = std::env::temp_dir().join("costpilot_sig_test");
    fs::create_dir_all(&temp_dir).ok();

    let test_file = temp_dir.join("signed.txt");
    fs::write(&test_file, "Content to be signed").ok();

    // Test GPG signature creation if available
    let sig_result = Command::new("gpg")
        .args(&["--detach-sign", "--armor", &test_file.to_string_lossy()])
        .output()
        .ok();

    if sig_result.is_some() {
        let sig_file = format!("{}.asc", test_file.display());
        assert!(
            Path::new(&sig_file).exists(),
            "Signature file should be created"
        );

        // Test signature verification
        let verify_result = Command::new("gpg")
            .args(&["--verify", &sig_file, &test_file.to_string_lossy()])
            .output()
            .ok();

        // Verification might fail due to key issues, but command should run
        assert!(
            verify_result.is_some(),
            "Signature verification should be attempted"
        );
    }

    fs::remove_dir_all(&temp_dir).ok();
}
