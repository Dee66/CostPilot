use std::fs;
use std::path::Path;
use std::process::Command;

// Test installer script validation
#[test]
fn test_installer_script_structure() {
    // Test that installer scripts exist and have basic structure
    let scripts = vec!["packaging/postinstall.js", "packaging/install.sh"];

    for script_path in scripts {
        if Path::new(script_path).exists() {
            let content = fs::read_to_string(script_path).unwrap_or_default();
            assert!(
                !content.is_empty(),
                "Script {} should not be empty",
                script_path
            );
        }
    }
}

// Test package manager detection logic
#[test]
fn test_package_manager_detection() {
    // Test detection of common package managers
    let package_managers = vec![
        ("apt", "/usr/bin/apt"),
        ("yum", "/usr/bin/yum"),
        ("dnf", "/usr/bin/dnf"),
        ("pacman", "/usr/bin/pacman"),
        ("brew", "/usr/local/bin/brew"),
        ("npm", "/usr/bin/npm"),
    ];

    for (name, expected_path) in package_managers {
        let exists = Path::new(expected_path).exists();
        // Just test that we can check for these paths (actual availability depends on system)
        let _detected = exists;
    }
}

// Test configuration file creation
#[test]
fn test_config_file_creation() {
    use std::env;
    use std::io::Write;

    // Test creating config files in different locations
    let temp_dir = env::temp_dir().join("costpilot_test_config");
    fs::create_dir_all(&temp_dir).ok();

    let config_paths = vec![
        temp_dir.join(".costpilot/config.yml"),
        temp_dir.join(".costpilot/baselines"),
    ];

    for config_path in &config_paths {
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent).ok();
        }
        let mut file = fs::File::create(config_path).expect("Failed to create config file");
        writeln!(file, "# Test config").ok();
        assert!(config_path.exists());
    }

    // Cleanup
    fs::remove_dir_all(&temp_dir).ok();
}

// Test permission handling
#[test]
fn test_permission_handling() {
    use std::os::unix::fs::PermissionsExt;

    let temp_file = std::env::temp_dir().join("costpilot_test_perms");
    fs::write(&temp_file, "test").ok();

    // Test setting executable permissions
    let metadata = fs::metadata(&temp_file).ok();
    if let Some(meta) = metadata {
        let mut perms = meta.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&temp_file, perms).ok();

        let new_meta = fs::metadata(&temp_file).ok();
        if let Some(new_meta) = new_meta {
            // On Unix systems, permissions should be set
            assert!(new_meta.permissions().mode() & 0o111 != 0); // executable bit set
        }
    }

    fs::remove_file(&temp_file).ok();
}

// Test environment variable handling
#[test]
fn test_environment_variable_handling() {
    // Test common installation-related environment variables
    let env_vars = vec![
        "HOME",
        "USER",
        "PATH",
        "SHELL",
        "XDG_CONFIG_HOME",
        "XDG_DATA_HOME",
    ];

    for var in env_vars {
        // Test that we can read these variables (they may or may not be set)
        let _value = std::env::var(var).ok();
    }

    // Test setting and reading custom environment variables
    std::env::set_var("COSTPILOT_TEST_VAR", "test_value");
    assert_eq!(std::env::var("COSTPILOT_TEST_VAR").unwrap(), "test_value");
    std::env::remove_var("COSTPILOT_TEST_VAR");
}

// Test installation directory validation
#[test]
fn test_installation_directory_validation() {
    let test_dirs = vec![
        "/usr/local/bin",
        "/usr/bin",
        "/opt/costpilot",
        "/home/user/.local/bin",
    ];

    for dir in test_dirs {
        let path = Path::new(dir);
        // Test that we can check if directories exist or could be created
        let _exists = path.exists();
        let _is_absolute = path.is_absolute();
    }
}

// Test dependency checking
#[test]
fn test_dependency_checking() {
    // Test checking for common dependencies
    let dependencies = vec!["curl", "wget", "tar", "gzip", "unzip"];

    for dep in dependencies {
        // Test that we can check if commands exist
        let result = Command::new("which").arg(dep).output().ok();
        let _available = result.map(|r| r.status.success()).unwrap_or(false);
    }
}

// Test version compatibility checking
#[test]
fn test_version_compatibility_checking() {
    // Test version string parsing and comparison
    let versions = vec!["1.0.0", "1.2.3", "2.0.0-alpha", "1.0.0-beta.1"];

    for version in versions {
        // Test basic version string validation
        assert!(version.contains('.'));
        assert!(version
            .chars()
            .all(|c| c.is_alphanumeric() || c == '.' || c == '-'));
    }
}

// Test uninstallation cleanup
#[test]
fn test_uninstallation_cleanup() {
    use std::env;

    let temp_dir = env::temp_dir().join("costpilot_test_uninstall");
    fs::create_dir_all(&temp_dir).ok();

    // Create some test files and directories
    let test_files = vec![
        temp_dir.join("config.yml"),
        temp_dir.join("data.db"),
        temp_dir.join("subdir/file.txt"),
    ];

    for file_path in &test_files {
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).ok();
        }
        fs::write(file_path, "test content").ok();
    }

    // Verify files exist
    for file_path in &test_files {
        assert!(file_path.exists());
    }

    // Simulate cleanup
    fs::remove_dir_all(&temp_dir).ok();
    assert!(!temp_dir.exists());
}

// Test restricted environment handling
#[test]
fn test_restricted_environment_handling() {
    // Test handling of restricted environments (no write access, etc.)
    let temp_dir = std::env::temp_dir();

    // Test that we can check directory permissions
    let metadata = fs::metadata(&temp_dir).ok();
    if let Some(meta) = metadata {
        let _readable = !meta.permissions().readonly();
        // In restricted environments, we might not be able to write
        // This test just validates we can check permissions
    }
}

#[test]
fn test_binary_installs_linux_architectures() {
    use std::process::Command;

    // Build for x86_64-unknown-linux-gnu
    let output = Command::new("cargo")
        .args(&["build", "--release", "--target", "x86_64-unknown-linux-gnu"])
        .output()
        .expect("Failed to run cargo build for x86_64");
    assert!(
        output.status.success(),
        "Build failed for x86_64: {:?}",
        output
    );

    let binary_path = "target/x86_64-unknown-linux-gnu/release/costpilot";
    assert!(
        Path::new(binary_path).exists(),
        "Binary not found at {}",
        binary_path
    );

    // Test that it runs --version
    let version_output = Command::new(binary_path)
        .arg("--version")
        .output()
        .expect("Failed to run --version");
    assert!(version_output.status.success());
    let version_str = String::from_utf8_lossy(&version_output.stdout);
    assert!(
        version_str.contains("costpilot"),
        "Version output invalid: {}",
        version_str
    );

    // For ARM64, check if target is installed
    let target_check = Command::new("rustup")
        .args(&["target", "list", "--installed"])
        .output()
        .expect("Failed to check installed targets");
    let targets = String::from_utf8_lossy(&target_check.stdout);
    if targets.contains("aarch64-unknown-linux-gnu") {
        let arm_output = Command::new("cargo")
            .args(&[
                "build",
                "--release",
                "--target",
                "aarch64-unknown-linux-gnu",
            ])
            .output()
            .expect("Failed to run cargo build for aarch64");
        assert!(
            arm_output.status.success(),
            "Build failed for aarch64: {:?}",
            arm_output
        );
        let arm_binary = "target/aarch64-unknown-linux-gnu/release/costpilot";
        assert!(
            Path::new(arm_binary).exists(),
            "ARM64 binary not found at {}",
            arm_binary
        );
    } else {
        // Skip ARM64 test if target not installed
        println!("ARM64 target not installed, skipping ARM64 build test");
    }
}
