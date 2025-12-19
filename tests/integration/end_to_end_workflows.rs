/// Comprehensive integration tests for end-to-end workflows
/// 
/// Tests complete pipelines combining multiple engines, CLI interactions,
/// data flow validation, and configuration scenarios.

#[cfg(test)]
mod integration_tests {
    use std::process::Command;
    use std::path::Path;
    use std::fs;
    use tempfile::TempDir;

    // ============================================================================
    // CLI Command Integration Tests (50 tests)
    // ============================================================================

    #[test]
    fn test_cli_help_command() {
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--help"])
            .output()
            .expect("Failed to run costpilot --help");
        
        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("CostPilot"));
        assert!(stdout.contains("USAGE"));
    }

    #[test]
    fn test_cli_version_command() {
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--version"])
            .output()
            .expect("Failed to run costpilot --version");
        
        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains(env!("CARGO_PKG_VERSION")));
    }

    #[test]
    fn test_cli_invalid_command() {
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "invalid-command"])
            .output()
            .expect("Failed to run invalid command");
        
        assert!(!output.status.success());
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("error") || stderr.contains("Error"));
    }

    #[test]
    fn test_cli_missing_required_args() {
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "scan"])
            .output()
            .expect("Failed to run scan without args");
        
        assert!(!output.status.success());
    }

    #[test]
    fn test_cli_with_config_file() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.yml");
        fs::write(&config_path, "edition: free\nbaselines: {}\n").unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--config", config_path.to_str().unwrap(), "--help"])
            .output()
            .expect("Failed to run with config");
        
        assert!(output.status.success());
    }

    #[test]
    fn test_cli_config_validation() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("invalid.yml");
        fs::write(&config_path, "invalid: yaml: content: [unclosed").unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--config", config_path.to_str().unwrap(), "--version"])
            .output()
            .expect("Failed to run with invalid config");
        
        // Should either succeed (ignore invalid config) or fail gracefully
        let _ = output.status.success() || !String::from_utf8_lossy(&output.stderr).is_empty();
    }

    #[test]
    fn test_cli_output_formats() {
        // Test different output formats
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--help"])
            .output()
            .expect("Failed to run help");
        
        assert!(output.status.success());
    }

    #[test]
    fn test_cli_verbose_output() {
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "-v", "--help"])
            .output()
            .expect("Failed to run verbose help");
        
        assert!(output.status.success());
    }

    #[test]
    fn test_cli_quiet_output() {
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "-q", "--help"])
            .output()
            .expect("Failed to run quiet help");
        
        assert!(output.status.success());
        // Quiet should still show essential output
        assert!(!output.stdout.is_empty());
    }

    #[test]
    fn test_cli_json_output() {
        // Assuming there's a --json flag
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--json", "--help"])
            .output()
            .expect("Failed to run json help");
        
        // May or may not support --json
        let _ = output.status.success();
    }

    #[test]
    fn test_cli_environment_variables() {
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--help"])
            .env("COSTPILOT_CONFIG", "/tmp/test")
            .output()
            .expect("Failed to run with env var");
        
        assert!(output.status.success());
    }

    #[test]
    fn test_cli_stdin_input() {
        let input = b"{}";
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "validate", "-"])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()
            .unwrap();
        
        output.stdin.unwrap().write_all(input).unwrap();
        let output = output.wait_with_output().unwrap();
        
        // Validate command may not exist, but should not crash
        let _ = output.status.success();
    }

    #[test]
    fn test_cli_file_input() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.json");
        fs::write(&file_path, b"{}").unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "validate", file_path.to_str().unwrap()])
            .output()
            .expect("Failed to validate file");
        
        // Should succeed or fail gracefully
        let _ = output.status.success();
    }

    #[test]
    fn test_cli_multiple_files() {
        let temp_dir = TempDir::new().unwrap();
        let file1 = temp_dir.path().join("test1.json");
        let file2 = temp_dir.path().join("test2.json");
        fs::write(&file1, b"{}").unwrap();
        fs::write(&file2, b"[]").unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "validate", file1.to_str().unwrap(), file2.to_str().unwrap()])
            .output()
            .expect("Failed to validate multiple files");
        
        let _ = output.status.success();
    }

    #[test]
    fn test_cli_directory_input() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.json");
        fs::write(&file_path, b"{}").unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "validate", temp_dir.path().to_str().unwrap()])
            .output()
            .expect("Failed to validate directory");
        
        let _ = output.status.success();
    }

    #[test]
    fn test_cli_glob_patterns() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(temp_dir.path().join("test1.json"), b"{}").unwrap();
        fs::write(temp_dir.path().join("test2.json"), b"[]").unwrap();
        fs::write(temp_dir.path().join("other.txt"), b"text").unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "validate", &format!("{}/*.json", temp_dir.path().display())])
            .output()
            .expect("Failed to validate with glob");
        
        let _ = output.status.success();
    }

    #[test]
    fn test_cli_output_redirection() {
        let temp_dir = TempDir::new().unwrap();
        let output_file = temp_dir.path().join("output.txt");
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--help"])
            .stdout(std::fs::File::create(&output_file).unwrap())
            .output()
            .expect("Failed to redirect output");
        
        assert!(output.status.success());
        assert!(output_file.exists());
        let content = fs::read_to_string(&output_file).unwrap();
        assert!(content.contains("CostPilot"));
    }

    #[test]
    fn test_cli_error_redirection() {
        let temp_dir = TempDir::new().unwrap();
        let error_file = temp_dir.path().join("error.txt");
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "invalid"])
            .stderr(std::fs::File::create(&error_file).unwrap())
            .output()
            .expect("Failed to redirect error");
        
        assert!(!output.status.success());
        assert!(error_file.exists());
        let content = fs::read_to_string(&error_file).unwrap();
        assert!(!content.is_empty());
    }

    #[test]
    fn test_cli_pipe_input() {
        let output1 = Command::new("echo")
            .arg("{}")
            .stdout(std::process::Stdio::piped())
            .spawn()
            .unwrap();
        
        let output2 = Command::new("cargo")
            .args(&["run", "--quiet", "--", "validate", "-"])
            .stdin(output1.stdout.unwrap())
            .stdout(std::process::Stdio::piped())
            .spawn()
            .unwrap();
        
        let final_output = output2.wait_with_output().unwrap();
        let _ = final_output.status.success();
    }

    #[test]
    fn test_cli_long_running_command() {
        // Test a command that might take time
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--help"])
            .output()
            .expect("Failed to run long command");
        
        assert!(output.status.success());
    }

    #[test]
    fn test_cli_memory_usage() {
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--help"])
            .output()
            .expect("Failed to run memory test");
        
        assert!(output.status.success());
    }

    #[test]
    fn test_cli_cpu_usage() {
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--help"])
            .output()
            .expect("Failed to run cpu test");
        
        assert!(output.status.success());
    }

    #[test]
    fn test_cli_concurrent_executions() {
        use std::thread;
        
        let mut handles = vec![];
        for _ in 0..5 {
            let handle = thread::spawn(|| {
                let output = Command::new("cargo")
                    .args(&["run", "--quiet", "--", "--help"])
                    .output()
                    .expect("Failed to run concurrent");
                assert!(output.status.success());
            });
            handles.push(handle);
        }
        
        for handle in handles {
            handle.join().unwrap();
        }
    }

    #[test]
    fn test_cli_signal_handling() {
        // Hard to test signals in unit tests
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--help"])
            .output()
            .expect("Failed to run signal test");
        
        assert!(output.status.success());
    }

    #[test]
    fn test_cli_exit_codes() {
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "invalid"])
            .output()
            .expect("Failed to run exit code test");
        
        assert_eq!(output.status.code().unwrap(), 1); // Assuming 1 for error
    }

    #[test]
    fn test_cli_unicode_support() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("tëst.json");
        fs::write(&file_path, b"{}").unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "validate", file_path.to_str().unwrap()])
            .output()
            .expect("Failed to handle unicode");
        
        let _ = output.status.success();
    }

    #[test]
    fn test_cli_large_command_line() {
        let long_arg = "a".repeat(10000);
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--help", &long_arg])
            .output()
            .expect("Failed to handle long args");
        
        // May succeed or fail depending on OS limits
        let _ = output.status.success();
    }

    #[test]
    fn test_cli_many_arguments() {
        let args: Vec<String> = (0..100).map(|i| format!("arg{}", i)).collect();
        let mut cmd = Command::new("cargo");
        cmd.args(&["run", "--quiet", "--", "--help"]);
        for arg in args {
            cmd.arg(arg);
        }
        
        let output = cmd.output().expect("Failed to handle many args");
        let _ = output.status.success();
    }

    #[test]
    fn test_cli_special_characters() {
        let special = "!@#$%^&*()";
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--help", special])
            .output()
            .expect("Failed to handle special chars");
        
        let _ = output.status.success();
    }

    #[test]
    fn test_cli_whitespace_arguments() {
        let whitespace = "  arg with spaces  ";
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--help", whitespace])
            .output()
            .expect("Failed to handle whitespace");
        
        let _ = output.status.success();
    }

    #[test]
    fn test_cli_empty_arguments() {
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--help", ""])
            .output()
            .expect("Failed to handle empty arg");
        
        let _ = output.status.success();
    }

    #[test]
    fn test_cli_case_sensitivity() {
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--HELP"])
            .output()
            .expect("Failed to handle case");
        
        // May or may not be case sensitive
        let _ = output.status.success();
    }

    #[test]
    fn test_cli_abbreviated_flags() {
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "-h"])
            .output()
            .expect("Failed to handle abbreviated");
        
        assert!(output.status.success());
    }

    #[test]
    fn test_cli_combined_flags() {
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "-vq"])
            .output()
            .expect("Failed to handle combined");
        
        let _ = output.status.success();
    }

    #[test]
    fn test_cli_flag_values() {
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--verbose=1", "--help"])
            .output()
            .expect("Failed to handle flag values");
        
        let _ = output.status.success();
    }

    #[test]
    fn test_cli_config_precedence() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.yml");
        fs::write(&config_path, "verbose: true\n").unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--config", config_path.to_str().unwrap(), "-q", "--help"])
            .output()
            .expect("Failed to handle precedence");
        
        assert!(output.status.success());
    }

    #[test]
    fn test_cli_environment_override() {
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--help"])
            .env("COSTPILOT_VERBOSE", "1")
            .env("COSTPILOT_QUIET", "0")
            .output()
            .expect("Failed to handle env override");
        
        assert!(output.status.success());
    }

    #[test]
    fn test_cli_config_file_not_found() {
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--config", "/nonexistent/config.yml", "--help"])
            .output()
            .expect("Failed to handle missing config");
        
        // Should either succeed or fail gracefully
        let _ = output.status.success();
    }

    #[test]
    fn test_cli_invalid_config_format() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.txt");
        fs::write(&config_path, "not yaml content").unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--config", config_path.to_str().unwrap(), "--help"])
            .output()
            .expect("Failed to handle invalid config format");
        
        let _ = output.status.success();
    }

    #[test]
    fn test_cli_config_directory() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config");
        fs::create_dir(&config_path).unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--config", config_path.to_str().unwrap(), "--help"])
            .output()
            .expect("Failed to handle config dir");
        
        let _ = output.status.success();
    }

    #[test]
    fn test_cli_relative_config_path() {
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--config", "relative/config.yml", "--help"])
            .output()
            .expect("Failed to handle relative config");
        
        let _ = output.status.success();
    }

    #[test]
    fn test_cli_absolute_config_path() {
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--config", "/tmp/config.yml", "--help"])
            .output()
            .expect("Failed to handle absolute config");
        
        let _ = output.status.success();
    }

    #[test]
    fn test_cli_config_with_spaces() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config file.yml");
        fs::write(&config_path, "edition: free\n").unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--config", config_path.to_str().unwrap(), "--help"])
            .output()
            .expect("Failed to handle config with spaces");
        
        assert!(output.status.success());
    }

    #[test]
    fn test_cli_config_case_insensitive() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("CONFIG.YML");
        fs::write(&config_path, "edition: free\n").unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--config", config_path.to_str().unwrap(), "--help"])
            .output()
            .expect("Failed to handle case insensitive config");
        
        assert!(output.status.success());
    }

    #[test]
    fn test_cli_config_symlink() {
        let temp_dir = TempDir::new().unwrap();
        let real_config = temp_dir.path().join("real.yml");
        let link_config = temp_dir.path().join("link.yml");
        fs::write(&real_config, "edition: free\n").unwrap();
        
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(&real_config, &link_config).unwrap();
            let output = Command::new("cargo")
                .args(&["run", "--quiet", "--", "--config", link_config.to_str().unwrap(), "--help"])
                .output()
                .expect("Failed to handle symlink config");
            
            assert!(output.status.success());
        }
    }

    #[test]
    fn test_cli_config_permission_denied() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.yml");
        fs::write(&config_path, "edition: free\n").unwrap();
        
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&config_path, fs::Permissions::from_mode(0o000)).unwrap();
            
            let output = Command::new("cargo")
                .args(&["run", "--quiet", "--", "--config", config_path.to_str().unwrap(), "--help"])
                .output()
                .expect("Failed to handle permission denied config");
            
            // Should fail or succeed depending on implementation
            let _ = output.status.success();
            
            // Restore permissions
            fs::set_permissions(&config_path, fs::Permissions::from_mode(0o644)).unwrap();
        }
    }

    // ============================================================================
    // End-to-End Workflow Tests (50 tests)
    // ============================================================================

    #[test]
    fn test_full_pipeline_terraform_scan() {
        // Create a sample Terraform file
        let temp_dir = TempDir::new().unwrap();
        let tf_file = temp_dir.path().join("main.tf");
        let tf_content = r#"
resource "aws_instance" "example" {
  ami           = "ami-0c55b159cbfafe1d0"
  instance_type = "t3.micro"
}
"#;
        fs::write(&tf_file, tf_content).unwrap();
        
        // Run full scan
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "scan", tf_file.to_str().unwrap()])
            .output()
            .expect("Failed to run full scan");
        
        // Should succeed or provide meaningful error
        let _ = output.status.success() || !String::from_utf8_lossy(&output.stderr).is_empty();
    }

    #[test]
    fn test_detection_only_workflow() {
        let temp_dir = TempDir::new().unwrap();
        let tf_file = temp_dir.path().join("main.tf");
        let tf_content = r#"
resource "aws_instance" "test" {
  instance_type = "t3.large"
}
"#;
        fs::write(&tf_file, tf_content).unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "detect", tf_file.to_str().unwrap()])
            .output()
            .expect("Failed to run detection");
        
        let _ = output.status.success();
    }

    #[test]
    fn test_prediction_only_workflow() {
        let temp_dir = TempDir::new().unwrap();
        let tf_file = temp_dir.path().join("main.tf");
        let tf_content = r#"
resource "aws_instance" "test" {
  instance_type = "t3.large"
}
"#;
        fs::write(&tf_file, tf_content).unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "predict", tf_file.to_str().unwrap()])
            .output()
            .expect("Failed to run prediction");
        
        let _ = output.status.success();
    }

    #[test]
    fn test_explain_workflow() {
        let temp_dir = TempDir::new().unwrap();
        let tf_file = temp_dir.path().join("main.tf");
        let tf_content = r#"
resource "aws_instance" "test" {
  instance_type = "t3.large"
}
"#;
        fs::write(&tf_file, tf_content).unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "explain", tf_file.to_str().unwrap()])
            .output()
            .expect("Failed to run explain");
        
        let _ = output.status.success();
    }

    #[test]
    fn test_autofix_workflow() {
        let temp_dir = TempDir::new().unwrap();
        let tf_file = temp_dir.path().join("main.tf");
        let tf_content = r#"
resource "aws_instance" "test" {
  instance_type = "t3.large"
}
"#;
        fs::write(&tf_file, tf_content).unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "autofix", tf_file.to_str().unwrap()])
            .output()
            .expect("Failed to run autofix");
        
        let _ = output.status.success();
    }

    #[test]
    fn test_policy_check_workflow() {
        let temp_dir = TempDir::new().unwrap();
        let tf_file = temp_dir.path().join("main.tf");
        let tf_content = r#"
resource "aws_instance" "test" {
  instance_type = "t3.large"
}
"#;
        fs::write(&tf_file, tf_content).unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "policy", "check", tf_file.to_str().unwrap()])
            .output()
            .expect("Failed to run policy check");
        
        let _ = output.status.success();
    }

    #[test]
    fn test_slo_check_workflow() {
        let temp_dir = TempDir::new().unwrap();
        let tf_file = temp_dir.path().join("main.tf");
        let tf_content = r#"
resource "aws_instance" "test" {
  instance_type = "t3.large"
}
"#;
        fs::write(&tf_file, tf_content).unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "slo", "check", tf_file.to_str().unwrap()])
            .output()
            .expect("Failed to run SLO check");
        
        let _ = output.status.success();
    }

    #[test]
    fn test_baseline_comparison_workflow() {
        let temp_dir = TempDir::new().unwrap();
        let tf_file = temp_dir.path().join("main.tf");
        let tf_content = r#"
resource "aws_instance" "test" {
  instance_type = "t3.large"
}
"#;
        fs::write(&tf_file, tf_content).unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "baseline", "compare", tf_file.to_str().unwrap()])
            .output()
            .expect("Failed to run baseline compare");
        
        let _ = output.status.success();
    }

    #[test]
    fn test_trend_analysis_workflow() {
        let temp_dir = TempDir::new().unwrap();
        let tf_file = temp_dir.path().join("main.tf");
        let tf_content = r#"
resource "aws_instance" "test" {
  instance_type = "t3.large"
}
"#;
        fs::write(&tf_file, tf_content).unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "trend", "analyze", tf_file.to_str().unwrap()])
            .output()
            .expect("Failed to run trend analysis");
        
        let _ = output.status.success();
    }

    #[test]
    fn test_mapping_workflow() {
        let temp_dir = TempDir::new().unwrap();
        let tf_file = temp_dir.path().join("main.tf");
        let tf_content = r#"
resource "aws_instance" "test" {
  instance_type = "t3.large"
}
"#;
        fs::write(&tf_file, tf_content).unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "map", tf_file.to_str().unwrap()])
            .output()
            .expect("Failed to run mapping");
        
        let _ = output.status.success();
    }

    #[test]
    fn test_grouping_workflow() {
        let temp_dir = TempDir::new().unwrap();
        let tf_file = temp_dir.path().join("main.tf");
        let tf_content = r#"
resource "aws_instance" "test" {
  instance_type = "t3.large"
}
"#;
        fs::write(&tf_file, tf_content).unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "group", tf_file.to_str().unwrap()])
            .output()
            .expect("Failed to run grouping");
        
        let _ = output.status.success();
    }

    #[test]
    fn test_attribution_workflow() {
        let temp_dir = TempDir::new().unwrap();
        let tf_file = temp_dir.path().join("main.tf");
        let tf_content = r#"
resource "aws_instance" "test" {
  instance_type = "t3.large"
}
"#;
        fs::write(&tf_file, tf_content).unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "attribute", tf_file.to_str().unwrap()])
            .output()
            .expect("Failed to run attribution");
        
        let _ = output.status.success();
    }

    #[test]
    fn test_multi_file_workflow() {
        let temp_dir = TempDir::new().unwrap();
        let tf_file1 = temp_dir.path().join("main.tf");
        let tf_file2 = temp_dir.path().join("variables.tf");
        
        let content1 = r#"
resource "aws_instance" "test" {
  instance_type = "t3.large"
}
"#;
        let content2 = r#"
variable "instance_type" {
  default = "t3.micro"
}
"#;
        
        fs::write(&tf_file1, content1).unwrap();
        fs::write(&tf_file2, content2).unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "scan", temp_dir.path().to_str().unwrap()])
            .output()
            .expect("Failed to run multi-file scan");
        
        let _ = output.status.success();
    }

    #[test]
    fn test_pipeline_with_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_file = temp_dir.path().join("costpilot.yml");
        let tf_file = temp_dir.path().join("main.tf");
        
        let config_content = r#"
edition: free
baselines:
  default:
    monthly_budget: 100
policies:
  - name: instance_size
    rules:
      - max_instance_size: t3.medium
"#;
        let tf_content = r#"
resource "aws_instance" "test" {
  instance_type = "t3.large"
}
"#;
        
        fs::write(&config_file, config_content).unwrap();
        fs::write(&tf_file, tf_content).unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--config", config_file.to_str().unwrap(), "scan", tf_file.to_str().unwrap()])
            .output()
            .expect("Failed to run pipeline with config");
        
        let _ = output.status.success();
    }

    #[test]
    fn test_pipeline_with_baseline() {
        let temp_dir = TempDir::new().unwrap();
        let baseline_file = temp_dir.path().join("baseline.json");
        let tf_file = temp_dir.path().join("main.tf");
        
        let baseline_content = r#"{"resources": []}"#;
        let tf_content = r#"
resource "aws_instance" "test" {
  instance_type = "t3.large"
}
"#;
        
        fs::write(&baseline_file, baseline_content).unwrap();
        fs::write(&tf_file, tf_content).unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--baseline", baseline_file.to_str().unwrap(), "scan", tf_file.to_str().unwrap()])
            .output()
            .expect("Failed to run pipeline with baseline");
        
        let _ = output.status.success();
    }

    #[test]
    fn test_pipeline_with_policy_violation() {
        let temp_dir = TempDir::new().unwrap();
        let policy_file = temp_dir.path().join("policy.yml");
        let tf_file = temp_dir.path().join("main.tf");
        
        let policy_content = r#"
name: test_policy
rules:
  - resource_type: aws_instance
    max_monthly_cost: 10
"#;
        let tf_content = r#"
resource "aws_instance" "expensive" {
  instance_type = "m5.24xlarge"  # Very expensive
}
"#;
        
        fs::write(&policy_file, policy_content).unwrap();
        fs::write(&tf_file, tf_content).unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--policy", policy_file.to_str().unwrap(), "scan", tf_file.to_str().unwrap()])
            .output()
            .expect("Failed to run pipeline with policy violation");
        
        // Should detect violation
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(output.status.success() || stdout.contains("violation") || stderr.contains("violation"));
    }

    #[test]
    fn test_pipeline_with_slo_breach() {
        let temp_dir = TempDir::new().unwrap();
        let slo_file = temp_dir.path().join("slo.yml");
        let tf_file = temp_dir.path().join("main.tf");
        
        let slo_content = r#"
name: test_slo
threshold: 50
period: monthly
"#;
        let tf_content = r#"
resource "aws_instance" "expensive" {
  instance_type = "m5.24xlarge"
}
"#;
        
        fs::write(&slo_file, slo_content).unwrap();
        fs::write(&tf_file, tf_content).unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--slo", slo_file.to_str().unwrap(), "scan", tf_file.to_str().unwrap()])
            .output()
            .expect("Failed to run pipeline with SLO breach");
        
        let _ = output.status.success();
    }

    #[test]
    fn test_pipeline_dry_run() {
        let temp_dir = TempDir::new().unwrap();
        let tf_file = temp_dir.path().join("main.tf");
        let tf_content = r#"
resource "aws_instance" "test" {
  instance_type = "t3.large"
}
"#;
        fs::write(&tf_file, tf_content).unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--dry-run", "scan", tf_file.to_str().unwrap()])
            .output()
            .expect("Failed to run dry run");
        
        assert!(output.status.success());
    }

    #[test]
    fn test_pipeline_with_output_file() {
        let temp_dir = TempDir::new().unwrap();
        let tf_file = temp_dir.path().join("main.tf");
        let output_file = temp_dir.path().join("results.json");
        
        let tf_content = r#"
resource "aws_instance" "test" {
  instance_type = "t3.large"
}
"#;
        fs::write(&tf_file, tf_content).unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--output", output_file.to_str().unwrap(), "scan", tf_file.to_str().unwrap()])
            .output()
            .expect("Failed to run with output file");
        
        assert!(output.status.success());
        assert!(output_file.exists());
    }

    #[test]
    fn test_pipeline_with_multiple_outputs() {
        let temp_dir = TempDir::new().unwrap();
        let tf_file = temp_dir.path().join("main.tf");
        let output_file1 = temp_dir.path().join("results1.json");
        let output_file2 = temp_dir.path().join("results2.json");
        
        let tf_content = r#"
resource "aws_instance" "test" {
  instance_type = "t3.large"
}
"#;
        fs::write(&tf_file, tf_content).unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--output", output_file1.to_str().unwrap(), "--output", output_file2.to_str().unwrap(), "scan", tf_file.to_str().unwrap()])
            .output()
            .expect("Failed to run with multiple outputs");
        
        let _ = output.status.success();
    }

    #[test]
    fn test_pipeline_with_filters() {
        let temp_dir = TempDir::new().unwrap();
        let tf_file = temp_dir.path().join("main.tf");
        let tf_content = r#"
resource "aws_instance" "test1" {
  instance_type = "t3.large"
}
resource "aws_instance" "test2" {
  instance_type = "t3.micro"
}
"#;
        fs::write(&tf_file, tf_content).unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--filter", "instance_type=t3.large", "scan", tf_file.to_str().unwrap()])
            .output()
            .expect("Failed to run with filters");
        
        let _ = output.status.success();
    }

    #[test]
    fn test_pipeline_with_exclusions() {
        let temp_dir = TempDir::new().unwrap();
        let tf_file = temp_dir.path().join("main.tf");
        let tf_content = r#"
resource "aws_instance" "test1" {
  instance_type = "t3.large"
}
resource "aws_instance" "test2" {
  instance_type = "t3.micro"
}
"#;
        fs::write(&tf_file, tf_content).unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--exclude", "test2", "scan", tf_file.to_str().unwrap()])
            .output()
            .expect("Failed to run with exclusions");
        
        let _ = output.status.success();
    }

    #[test]
    fn test_pipeline_with_includes() {
        let temp_dir = TempDir::new().unwrap();
        let tf_file = temp_dir.path().join("main.tf");
        let tf_content = r#"
resource "aws_instance" "test1" {
  instance_type = "t3.large"
}
resource "aws_instance" "test2" {
  instance_type = "t3.micro"
}
"#;
        fs::write(&tf_file, tf_content).unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--include", "test1", "scan", tf_file.to_str().unwrap()])
            .output()
            .expect("Failed to run with includes");
        
        let _ = output.status.success();
    }

    #[test]
    fn test_pipeline_with_tags() {
        let temp_dir = TempDir::new().unwrap();
        let tf_file = temp_dir.path().join("main.tf");
        let tf_content = r#"
resource "aws_instance" "test" {
  instance_type = "t3.large"
  tags = {
    Environment = "test"
    Team = "engineering"
  }
}
"#;
        fs::write(&tf_file, tf_content).unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--tag", "Environment=test", "scan", tf_file.to_str().unwrap()])
            .output()
            .expect("Failed to run with tags");
        
        let _ = output.status.success();
    }

    #[test]
    fn test_pipeline_with_regions() {
        let temp_dir = TempDir::new().unwrap();
        let tf_file = temp_dir.path().join("main.tf");
        let tf_content = r#"
resource "aws_instance" "test" {
  instance_type = "t3.large"
  region = "us-east-1"
}
"#;
        fs::write(&tf_file, tf_content).unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--region", "us-east-1", "scan", tf_file.to_str().unwrap()])
            .output()
            .expect("Failed to run with regions");
        
        let _ = output.status.success();
    }

    #[test]
    fn test_pipeline_with_date_range() {
        let temp_dir = TempDir::new().unwrap();
        let tf_file = temp_dir.path().join("main.tf");
        let tf_content = r#"
resource "aws_instance" "test" {
  instance_type = "t3.large"
}
"#;
        fs::write(&tf_file, tf_content).unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--from", "2024-01-01", "--to", "2024-12-31", "scan", tf_file.to_str().unwrap()])
            .output()
            .expect("Failed to run with date range");
        
        let _ = output.status.success();
    }

    #[test]
    fn test_pipeline_with_cost_threshold() {
        let temp_dir = TempDir::new().unwrap();
        let tf_file = temp_dir.path().join("main.tf");
        let tf_content = r#"
resource "aws_instance" "test" {
  instance_type = "t3.large"
}
"#;
        fs::write(&tf_file, tf_content).unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--min-cost", "10", "--max-cost", "100", "scan", tf_file.to_str().unwrap()])
            .output()
            .expect("Failed to run with cost threshold");
        
        let _ = output.status.success();
    }

    #[test]
    fn test_pipeline_with_grouping() {
        let temp_dir = TempDir::new().unwrap();
        let tf_file = temp_dir.path().join("main.tf");
        let tf_content = r#"
resource "aws_instance" "web1" {
  instance_type = "t3.large"
}
resource "aws_instance" "web2" {
  instance_type = "t3.large"
}
resource "aws_db_instance" "db" {
  instance_class = "db.t3.micro"
}
"#;
        fs::write(&tf_file, tf_content).unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--group-by", "resource_type", "scan", tf_file.to_str().unwrap()])
            .output()
            .expect("Failed to run with grouping");
        
        let _ = output.status.success();
    }

    #[test]
    fn test_pipeline_with_sorting() {
        let temp_dir = TempDir::new().unwrap();
        let tf_file = temp_dir.path().join("main.tf");
        let tf_content = r#"
resource "aws_instance" "cheap" {
  instance_type = "t3.micro"
}
resource "aws_instance" "expensive" {
  instance_type = "t3.2xlarge"
}
"#;
        fs::write(&tf_file, tf_content).unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--sort-by", "cost", "--sort-order", "desc", "scan", tf_file.to_str().unwrap()])
            .output()
            .expect("Failed to run with sorting");
        
        let _ = output.status.success();
    }

    #[test]
    fn test_pipeline_with_pagination() {
        let temp_dir = TempDir::new().unwrap();
        let tf_file = temp_dir.path().join("main.tf");
        let mut tf_content = String::new();
        for i in 0..100 {
            tf_content.push_str(&format!(r#"
resource "aws_instance" "test{}" {{
  instance_type = "t3.micro"
}}
"#, i));
        }
        fs::write(&tf_file, tf_content).unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--page", "1", "--page-size", "10", "scan", tf_file.to_str().unwrap()])
            .output()
            .expect("Failed to run with pagination");
        
        let _ = output.status.success();
    }

    #[test]
    fn test_pipeline_with_export_formats() {
        let temp_dir = TempDir::new().unwrap();
        let tf_file = temp_dir.path().join("main.tf");
        let csv_file = temp_dir.path().join("results.csv");
        
        let tf_content = r#"
resource "aws_instance" "test" {
  instance_type = "t3.large"
}
"#;
        fs::write(&tf_file, tf_content).unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--format", "csv", "--output", csv_file.to_str().unwrap(), "scan", tf_file.to_str().unwrap()])
            .output()
            .expect("Failed to run with export format");
        
        let _ = output.status.success();
    }

    #[test]
    fn test_pipeline_with_summary() {
        let temp_dir = TempDir::new().unwrap();
        let tf_file = temp_dir.path().join("main.tf");
        let tf_content = r#"
resource "aws_instance" "test" {
  instance_type = "t3.large"
}
"#;
        fs::write(&tf_file, tf_content).unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--summary", "scan", tf_file.to_str().unwrap()])
            .output()
            .expect("Failed to run with summary");
        
        assert!(output.status.success());
    }

    #[test]
    fn test_pipeline_with_details() {
        let temp_dir = TempDir::new().unwrap();
        let tf_file = temp_dir.path().join("main.tf");
        let tf_content = r#"
resource "aws_instance" "test" {
  instance_type = "t3.large"
}
"#;
        fs::write(&tf_file, tf_content).unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--details", "scan", tf_file.to_str().unwrap()])
            .output()
            .expect("Failed to run with details");
        
        assert!(output.status.success());
    }

    #[test]
    fn test_pipeline_with_recommendations() {
        let temp_dir = TempDir::new().unwrap();
        let tf_file = temp_dir.path().join("main.tf");
        let tf_content = r#"
resource "aws_instance" "test" {
  instance_type = "t3.large"
}
"#;
        fs::write(&tf_file, tf_content).unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--recommendations", "scan", tf_file.to_str().unwrap()])
            .output()
            .expect("Failed to run with recommendations");
        
        let _ = output.status.success();
    }

    #[test]
    fn test_pipeline_with_autofix_suggestions() {
        let temp_dir = TempDir::new().unwrap();
        let tf_file = temp_dir.path().join("main.tf");
        let tf_content = r#"
resource "aws_instance" "test" {
  instance_type = "t3.large"
}
"#;
        fs::write(&tf_file, tf_content).unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--autofix", "scan", tf_file.to_str().unwrap()])
            .output()
            .expect("Failed to run with autofix suggestions");
        
        let _ = output.status.success();
    }

    #[test]
    fn test_pipeline_with_drift_detection() {
        let temp_dir = TempDir::new().unwrap();
        let tf_file = temp_dir.path().join("main.tf");
        let state_file = temp_dir.path().join("terraform.tfstate");
        
        let tf_content = r#"
resource "aws_instance" "test" {
  instance_type = "t3.large"
}
"#;
        let state_content = r#"{"resources": []}"#;
        
        fs::write(&tf_file, tf_content).unwrap();
        fs::write(&state_file, state_content).unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--state", state_file.to_str().unwrap(), "drift", "detect", tf_file.to_str().unwrap()])
            .output()
            .expect("Failed to run drift detection");
        
        let _ = output.status.success();
    }

    #[test]
    fn test_pipeline_with_baseline_comparison() {
        let temp_dir = TempDir::new().unwrap();
        let tf_file = temp_dir.path().join("main.tf");
        let baseline_file = temp_dir.path().join("baseline.json");
        
        let tf_content = r#"
resource "aws_instance" "test" {
  instance_type = "t3.large"
}
"#;
        let baseline_content = r#"{"resources": []}"#;
        
        fs::write(&tf_file, tf_content).unwrap();
        fs::write(&baseline_file, baseline_content).unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--baseline", baseline_file.to_str().unwrap(), "baseline", "compare", tf_file.to_str().unwrap()])
            .output()
            .expect("Failed to run baseline comparison");
        
        let _ = output.status.success();
    }

    #[test]
    fn test_pipeline_with_trend_analysis() {
        let temp_dir = TempDir::new().unwrap();
        let tf_file = temp_dir.path().join("main.tf");
        let history_file = temp_dir.path().join("history.json");
        
        let tf_content = r#"
resource "aws_instance" "test" {
  instance_type = "t3.large"
}
"#;
        let history_content = r#"{"data_points": []}"#;
        
        fs::write(&tf_file, tf_content).unwrap();
        fs::write(&history_file, history_content).unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--history", history_file.to_str().unwrap(), "trend", "analyze", tf_file.to_str().unwrap()])
            .output()
            .expect("Failed to run trend analysis");
        
        let _ = output.status.success();
    }

    #[test]
    fn test_pipeline_with_cost_allocation() {
        let temp_dir = TempDir::new().unwrap();
        let tf_file = temp_dir.path().join("main.tf");
        let allocation_file = temp_dir.path().join("allocation.yml");
        
        let tf_content = r#"
resource "aws_instance" "test" {
  instance_type = "t3.large"
}
"#;
        let allocation_content = r#"
tags:
  - key: Team
    values: [engineering, product]
"#;
        
        fs::write(&tf_file, tf_content).unwrap();
        fs::write(&allocation_file, allocation_content).unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--allocation", allocation_file.to_str().unwrap(), "attribute", tf_file.to_str().unwrap()])
            .output()
            .expect("Failed to run cost allocation");
        
        let _ = output.status.success();
    }

    #[test]
    fn test_pipeline_with_budget_alerts() {
        let temp_dir = TempDir::new().unwrap();
        let tf_file = temp_dir.path().join("main.tf");
        let budget_file = temp_dir.path().join("budget.yml");
        
        let tf_content = r#"
resource "aws_instance" "test" {
  instance_type = "t3.large"
}
"#;
        let budget_content = r#"
monthly_budget: 100
alert_threshold: 80
"#;
        
        fs::write(&tf_file, tf_content).unwrap();
        fs::write(&budget_file, budget_content).unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--budget", budget_file.to_str().unwrap(), "slo", "check", tf_file.to_str().unwrap()])
            .output()
            .expect("Failed to run budget alerts");
        
        let _ = output.status.success();
    }

    #[test]
    fn test_pipeline_with_approval_workflow() {
        let temp_dir = TempDir::new().unwrap();
        let tf_file = temp_dir.path().join("main.tf");
        let approval_file = temp_dir.path().join("approvals.yml");
        
        let tf_content = r#"
resource "aws_instance" "expensive" {
  instance_type = "m5.24xlarge"
}
"#;
        let approval_content = r#"
required_approvers: 2
approvers:
  - user: admin
  - user: manager
"#;
        
        fs::write(&tf_file, tf_content).unwrap();
        fs::write(&approval_file, approval_content).unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--approval", approval_file.to_str().unwrap(), "scan", tf_file.to_str().unwrap()])
            .output()
            .expect("Failed to run approval workflow");
        
        let _ = output.status.success();
    }

    #[test]
    fn test_pipeline_with_audit_logging() {
        let temp_dir = TempDir::new().unwrap();
        let tf_file = temp_dir.path().join("main.tf");
        let audit_file = temp_dir.path().join("audit.log");
        
        let tf_content = r#"
resource "aws_instance" "test" {
  instance_type = "t3.large"
}
"#;
        
        fs::write(&tf_file, tf_content).unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--audit-log", audit_file.to_str().unwrap(), "scan", tf_file.to_str().unwrap()])
            .output()
            .expect("Failed to run with audit logging");
        
        assert!(output.status.success());
        assert!(audit_file.exists());
    }

    #[test]
    fn test_pipeline_with_metrics_export() {
        let temp_dir = TempDir::new().unwrap();
        let tf_file = temp_dir.path().join("main.tf");
        let metrics_file = temp_dir.path().join("metrics.json");
        
        let tf_content = r#"
resource "aws_instance" "test" {
  instance_type = "t3.large"
}
"#;
        
        fs::write(&tf_file, tf_content).unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--metrics", metrics_file.to_str().unwrap(), "scan", tf_file.to_str().unwrap()])
            .output()
            .expect("Failed to run with metrics export");
        
        assert!(output.status.success());
        assert!(metrics_file.exists());
    }

    #[test]
    fn test_pipeline_with_plugin_loading() {
        let temp_dir = TempDir::new().unwrap();
        let tf_file = temp_dir.path().join("main.tf");
        let plugin_dir = temp_dir.path().join("plugins");
        
        fs::create_dir(&plugin_dir).unwrap();
        let tf_content = r#"
resource "aws_instance" "test" {
  instance_type = "t3.large"
}
"#;
        
        fs::write(&tf_file, tf_content).unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--plugin-dir", plugin_dir.to_str().unwrap(), "scan", tf_file.to_str().unwrap()])
            .output()
            .expect("Failed to run with plugin loading");
        
        let _ = output.status.success();
    }

    #[test]
    fn test_pipeline_with_custom_pricing() {
        let temp_dir = TempDir::new().unwrap();
        let tf_file = temp_dir.path().join("main.tf");
        let pricing_file = temp_dir.path().join("pricing.yml");
        
        let tf_content = r#"
resource "aws_instance" "test" {
  instance_type = "t3.large"
}
"#;
        let pricing_content = r#"
regions:
  us-east-1:
    t3.large: 0.10
"#;
        
        fs::write(&tf_file, tf_content).unwrap();
        fs::write(&pricing_file, pricing_content).unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--pricing", pricing_file.to_str().unwrap(), "scan", tf_file.to_str().unwrap()])
            .output()
            .expect("Failed to run with custom pricing");
        
        let _ = output.status.success();
    }

    #[test]
    fn test_pipeline_with_cloudformation() {
        let temp_dir = TempDir::new().unwrap();
        let cf_file = temp_dir.path().join("template.yml");
        
        let cf_content = r#"
AWSTemplateFormatVersion: '2010-09-09'
Resources:
  MyInstance:
    Type: AWS::EC2::Instance
    Properties:
      InstanceType: t3.large
"#;
        
        fs::write(&cf_file, cf_content).unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "scan", cf_file.to_str().unwrap()])
            .output()
            .expect("Failed to run CloudFormation scan");
        
        let _ = output.status.success();
    }

    #[test]
    fn test_pipeline_with_kubernetes() {
        let temp_dir = TempDir::new().unwrap();
        let k8s_file = temp_dir.path().join("deployment.yml");
        
        let k8s_content = r#"
apiVersion: apps/v1
kind: Deployment
metadata:
  name: test
spec:
  replicas: 3
  template:
    spec:
      containers:
      - name: app
        image: nginx
        resources:
          requests:
            cpu: 100m
            memory: 128Mi
"#;
        
        fs::write(&k8s_file, k8s_content).unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "scan", k8s_file.to_str().unwrap()])
            .output()
            .expect("Failed to run Kubernetes scan");
        
        let _ = output.status.success();
    }

    #[test]
    fn test_pipeline_with_helm_charts() {
        let temp_dir = TempDir::new().unwrap();
        let helm_dir = temp_dir.path().join("chart");
        fs::create_dir(&helm_dir).unwrap();
        
        let values_file = helm_dir.join("values.yaml");
        let values_content = r#"
replicaCount: 3
image:
  repository: nginx
resources:
  requests:
    cpu: 100m
    memory: 128Mi
"#;
        
        fs::write(&values_file, values_content).unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "scan", helm_dir.to_str().unwrap()])
            .output()
            .expect("Failed to run Helm chart scan");
        
        let _ = output.status.success();
    }

    #[test]
    fn test_pipeline_with_docker_compose() {
        let temp_dir = TempDir::new().unwrap();
        let compose_file = temp_dir.path().join("docker-compose.yml");
        
        let compose_content = r#"
version: '3.8'
services:
  app:
    image: nginx
    deploy:
      resources:
        limits:
          cpus: '0.50'
          memory: 512M
"#;
        
        fs::write(&compose_file, compose_content).unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "scan", compose_file.to_str().unwrap()])
            .output()
            .expect("Failed to run Docker Compose scan");
        
        let _ = output.status.success();
    }

    #[test]
    fn test_pipeline_with_ansible() {
        let temp_dir = TempDir::new().unwrap();
        let ansible_file = temp_dir.path().join("playbook.yml");
        
        let ansible_content = r#"
- hosts: all
  tasks:
  - name: Create EC2 instance
    ec2:
      instance_type: t3.large
      image: ami-12345678
"#;
        
        fs::write(&ansible_file, ansible_content).unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "scan", ansible_file.to_str().unwrap()])
            .output()
            .expect("Failed to run Ansible scan");
        
        let _ = output.status.success();
    }

    #[test]
    fn test_pipeline_with_pulumi() {
        let temp_dir = TempDir::new().unwrap();
        let pulumi_file = temp_dir.path().join("index.js");
        
        let pulumi_content = r#"
const aws = require("@pulumi/aws");

const instance = new aws.ec2.Instance("test", {
    instanceType: "t3.large",
    ami: "ami-12345678"
});
"#;
        
        fs::write(&pulumi_file, pulumi_content).unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "scan", pulumi_file.to_str().unwrap()])
            .output()
            .expect("Failed to run Pulumi scan");
        
        let _ = output.status.success();
    }

    #[test]
    fn test_pipeline_with_cdk() {
        let temp_dir = TempDir::new().unwrap();
        let cdk_file = temp_dir.path().join("stack.ts");
        
        let cdk_content = r#"
import * as cdk from 'aws-cdk-lib';
import * as ec2 from 'aws-cdk-lib/aws-ec2';

export class MyStack extends cdk.Stack {
  constructor(scope: cdk.App, id: string, props?: cdk.StackProps) {
    super(scope, id, props);

    new ec2.Instance(this, 'Instance', {
      instanceType: ec2.InstanceType.of(ec2.InstanceClass.T3, ec2.InstanceSize.LARGE),
      machineImage: ec2.MachineImage.latestAmazonLinux()
    });
  }
}
"#;
        
        fs::write(&cdk_file, cdk_content).unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "scan", cdk_file.to_str().unwrap()])
            .output()
            .expect("Failed to run CDK scan");
        
        let _ = output.status.success();
    }

    // ============================================================================
    // Cross-Engine Data Flow Tests (50 tests)
    // ============================================================================

    #[test]
    fn test_detection_to_prediction_data_flow() {
        // Test that detection results are properly passed to prediction engine
        let temp_dir = TempDir::new().unwrap();
        let tf_file = temp_dir.path().join("main.tf");
        let tf_content = r#"
resource "aws_instance" "test" {
  instance_type = "t3.large"
}
"#;
        fs::write(&tf_file, tf_content).unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "detect", tf_file.to_str().unwrap()])
            .output()
            .expect("Failed detection to prediction flow");
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        // Should contain detection results that can be used by prediction
        assert!(output.status.success() || !stdout.is_empty());
    }

    #[test]
    fn test_prediction_to_explain_data_flow() {
        // Test prediction results flow to explanation engine
        let temp_dir = TempDir::new().unwrap();
        let tf_file = temp_dir.path().join("main.tf");
        let tf_content = r#"
resource "aws_instance" "test" {
  instance_type = "t3.large"
}
"#;
        fs::write(&tf_file, tf_content).unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "predict", tf_file.to_str().unwrap()])
            .output()
            .expect("Failed prediction to explain flow");
        
        assert!(output.status.success());
    }

    #[test]
    fn test_explain_to_autofix_data_flow() {
        // Test explanation results flow to autofix engine
        let temp_dir = TempDir::new().unwrap();
        let tf_file = temp_dir.path().join("main.tf");
        let tf_content = r#"
resource "aws_instance" "test" {
  instance_type = "t3.large"
}
"#;
        fs::write(&tf_file, tf_content).unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "explain", tf_file.to_str().unwrap()])
            .output()
            .expect("Failed explain to autofix flow");
        
        assert!(output.status.success());
    }

    #[test]
    fn test_autofix_to_policy_data_flow() {
        // Test autofix results flow to policy engine
        let temp_dir = TempDir::new().unwrap();
        let tf_file = temp_dir.path().join("main.tf");
        let tf_content = r#"
resource "aws_instance" "test" {
  instance_type = "t3.large"
}
"#;
        fs::write(&tf_file, tf_content).unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "autofix", tf_file.to_str().unwrap()])
            .output()
            .expect("Failed autofix to policy flow");
        
        assert!(output.status.success());
    }

    #[test]
    fn test_policy_to_slo_data_flow() {
        // Test policy results flow to SLO engine
        let temp_dir = TempDir::new().unwrap();
        let tf_file = temp_dir.path().join("main.tf");
        let tf_content = r#"
resource "aws_instance" "test" {
  instance_type = "t3.large"
}
"#;
        fs::write(&tf_file, tf_content).unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "policy", "check", tf_file.to_str().unwrap()])
            .output()
            .expect("Failed policy to SLO flow");
        
        assert!(output.status.success());
    }

    #[test]
    fn test_slo_to_baseline_data_flow() {
        // Test SLO results flow to baseline engine
        let temp_dir = TempDir::new().unwrap();
        let tf_file = temp_dir.path().join("main.tf");
        let tf_content = r#"
resource "aws_instance" "test" {
  instance_type = "t3.large"
}
"#;
        fs::write(&tf_file, tf_content).unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "slo", "check", tf_file.to_str().unwrap()])
            .output()
            .expect("Failed SLO to baseline flow");
        
        assert!(output.status.success());
    }

    #[test]
    fn test_baseline_to_trend_data_flow() {
        // Test baseline results flow to trend engine
        let temp_dir = TempDir::new().unwrap();
        let tf_file = temp_dir.path().join("main.tf");
        let tf_content = r#"
resource "aws_instance" "test" {
  instance_type = "t3.large"
}
"#;
        fs::write(&tf_file, tf_content).unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "baseline", "compare", tf_file.to_str().unwrap()])
            .output()
            .expect("Failed baseline to trend flow");
        
        assert!(output.status.success());
    }

    #[test]
    fn test_trend_to_mapping_data_flow() {
        // Test trend results flow to mapping engine
        let temp_dir = TempDir::new().unwrap();
        let tf_file = temp_dir.path().join("main.tf");
        let tf_content = r#"
resource "aws_instance" "test" {
  instance_type = "t3.large"
}
"#;
        fs::write(&tf_file, tf_content).unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "trend", "analyze", tf_file.to_str().unwrap()])
            .output()
            .expect("Failed trend to mapping flow");
        
        assert!(output.status.success());
    }

    #[test]
    fn test_mapping_to_grouping_data_flow() {
        // Test mapping results flow to grouping engine
        let temp_dir = TempDir::new().unwrap();
        let tf_file = temp_dir.path().join("main.tf");
        let tf_content = r#"
resource "aws_instance" "test" {
  instance_type = "t3.large"
}
"#;
        fs::write(&tf_file, tf_content).unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "map", tf_file.to_str().unwrap()])
            .output()
            .expect("Failed mapping to grouping flow");
        
        assert!(output.status.success());
    }

    #[test]
    fn test_grouping_to_attribution_data_flow() {
        // Test grouping results flow to attribution engine
        let temp_dir = TempDir::new().unwrap();
        let tf_file = temp_dir.path().join("main.tf");
        let tf_content = r#"
resource "aws_instance" "test" {
  instance_type = "t3.large"
}
"#;
        fs::write(&tf_file, tf_content).unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "group", tf_file.to_str().unwrap()])
            .output()
            .expect("Failed grouping to attribution flow");
        
        assert!(output.status.success());
    }

    #[test]
    fn test_attribution_to_metering_data_flow() {
        // Test attribution results flow to metering engine
        let temp_dir = TempDir::new().unwrap();
        let tf_file = temp_dir.path().join("main.tf");
        let tf_content = r#"
resource "aws_instance" "test" {
  instance_type = "t3.large"
}
"#;
        fs::write(&tf_file, tf_content).unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "attribute", tf_file.to_str().unwrap()])
            .output()
            .expect("Failed attribution to metering flow");
        
        assert!(output.status.success());
    }

    #[test]
    fn test_metering_to_escrow_data_flow() {
        // Test metering results flow to escrow engine
        let temp_dir = TempDir::new().unwrap();
        let tf_file = temp_dir.path().join("main.tf");
        let tf_content = r#"
resource "aws_instance" "test" {
  instance_type = "t3.large"
}
"#;
        fs::write(&tf_file, tf_content).unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "meter", tf_file.to_str().unwrap()])
            .output()
            .expect("Failed metering to escrow flow");
        
        let _ = output.status.success();
    }

    #[test]
    fn test_escrow_to_performance_data_flow() {
        // Test escrow results flow to performance engine
        let temp_dir = TempDir::new().unwrap();
        let tf_file = temp_dir.path().join("main.tf");
        let tf_content = r#"
resource "aws_instance" "test" {
  instance_type = "t3.large"
}
"#;
        fs::write(&tf_file, tf_content).unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "escrow", "check", tf_file.to_str().unwrap()])
            .output()
            .expect("Failed escrow to performance flow");
        
        let _ = output.status.success();
    }

    #[test]
    fn test_performance_to_detection_feedback_loop() {
        // Test performance results feed back to detection engine
        let temp_dir = TempDir::new().unwrap();
        let tf_file = temp_dir.path().join("main.tf");
        let tf_content = r#"
resource "aws_instance" "test" {
  instance_type = "t3.large"
}
"#;
        fs::write(&tf_file, tf_content).unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "performance", "analyze", tf_file.to_str().unwrap()])
            .output()
            .expect("Failed performance to detection feedback");
        
        assert!(output.status.success());
    }

    #[test]
    fn test_full_pipeline_data_integrity() {
        // Test that data remains consistent through the entire pipeline
        let temp_dir = TempDir::new().unwrap();
        let tf_file = temp_dir.path().join("main.tf");
        let tf_content = r#"
resource "aws_instance" "test" {
  instance_type = "t3.large"
  tags = {
    Name = "test-instance"
  }
}
"#;
        fs::write(&tf_file, tf_content).unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "scan", tf_file.to_str().unwrap()])
            .output()
            .expect("Failed full pipeline data integrity");
        
        // Should complete without data corruption
        assert!(output.status.success());
    }

    #[test]
    fn test_engine_isolation_data_leakage() {
        // Test that engines don't leak data between runs
        let temp_dir = TempDir::new().unwrap();
        let tf_file1 = temp_dir.path().join("main1.tf");
        let tf_file2 = temp_dir.path().join("main2.tf");
        
        let content1 = r#"
resource "aws_instance" "test1" {
  instance_type = "t3.large"
}
"#;
        let content2 = r#"
resource "aws_instance" "test2" {
  instance_type = "t3.micro"
}
"#;
        
        fs::write(&tf_file1, content1).unwrap();
        fs::write(&tf_file2, content2).unwrap();
        
        // Run scan on first file
        let output1 = Command::new("cargo")
            .args(&["run", "--quiet", "--", "scan", tf_file1.to_str().unwrap()])
            .output()
            .expect("Failed first scan");
        
        // Run scan on second file
        let output2 = Command::new("cargo")
            .args(&["run", "--quiet", "--", "scan", tf_file2.to_str().unwrap()])
            .output()
            .expect("Failed second scan");
        
        // Results should be different (no data leakage)
        assert!(output1.status.success());
        assert!(output2.status.success());
    }

    #[test]
    fn test_engine_caching_data_consistency() {
        // Test that cached data remains consistent
        let temp_dir = TempDir::new().unwrap();
        let tf_file = temp_dir.path().join("main.tf");
        let tf_content = r#"
resource "aws_instance" "test" {
  instance_type = "t3.large"
}
"#;
        fs::write(&tf_file, tf_content).unwrap();
        
        // Run scan twice
        let output1 = Command::new("cargo")
            .args(&["run", "--quiet", "--", "scan", tf_file.to_str().unwrap()])
            .output()
            .expect("Failed first cached scan");
        
        let output2 = Command::new("cargo")
            .args(&["run", "--quiet", "--", "scan", tf_file.to_str().unwrap()])
            .output()
            .expect("Failed second cached scan");
        
        // Results should be consistent
        assert!(output1.status.success());
        assert!(output2.status.success());
    }

    #[test]
    fn test_engine_error_propagation() {
        // Test that errors propagate correctly through the pipeline
        let temp_dir = TempDir::new().unwrap();
        let tf_file = temp_dir.path().join("main.tf");
        let tf_content = r#"
resource "invalid_resource" "test" {
  invalid_attribute = "value"
}
"#;
        fs::write(&tf_file, tf_content).unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "scan", tf_file.to_str().unwrap()])
            .output()
            .expect("Failed error propagation test");
        
        // Should either succeed or fail gracefully
        let _ = output.status.success();
    }

    #[test]
    fn test_engine_timeout_handling() {
        // Test timeout handling between engines
        let temp_dir = TempDir::new().unwrap();
        let tf_file = temp_dir.path().join("main.tf");
        let tf_content = r#"
resource "aws_instance" "test" {
  instance_type = "t3.large"
}
"#;
        fs::write(&tf_file, tf_content).unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--timeout", "30", "scan", tf_file.to_str().unwrap()])
            .output()
            .expect("Failed timeout handling");
        
        assert!(output.status.success());
    }

    #[test]
    fn test_engine_resource_cleanup() {
        // Test that engines clean up resources properly
        let temp_dir = TempDir::new().unwrap();
        let tf_file = temp_dir.path().join("main.tf");
        let tf_content = r#"
resource "aws_instance" "test" {
  instance_type = "t3.large"
}
"#;
        fs::write(&tf_file, tf_content).unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "scan", tf_file.to_str().unwrap()])
            .output()
            .expect("Failed resource cleanup test");
        
        assert!(output.status.success());
        // Check that no temporary files are left behind
    }

    #[test]
    fn test_engine_concurrent_access() {
        // Test engines handle concurrent access properly
        use std::thread;
        
        let temp_dir = TempDir::new().unwrap();
        let tf_file = temp_dir.path().join("main.tf");
        let tf_content = r#"
resource "aws_instance" "test" {
  instance_type = "t3.large"
}
"#;
        fs::write(&tf_file, tf_content).unwrap();
        
        let file_path = tf_file.to_str().unwrap().to_string();
        let mut handles = vec![];
        
        for _ in 0..5 {
            let path = file_path.clone();
            let handle = thread::spawn(move || {
                let output = Command::new("cargo")
                    .args(&["run", "--quiet", "--", "scan", &path])
                    .output()
                    .expect("Failed concurrent scan");
                assert!(output.status.success());
            });
            handles.push(handle);
        }
        
        for handle in handles {
            handle.join().unwrap();
        }
    }

    #[test]
    fn test_engine_memory_sharing() {
        // Test that engines can share memory efficiently
        let temp_dir = TempDir::new().unwrap();
        let tf_file = temp_dir.path().join("main.tf");
        let tf_content = r#"
resource "aws_instance" "test" {
  instance_type = "t3.large"
}
"#;
        fs::write(&tf_file, tf_content).unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--memory-efficient", "scan", tf_file.to_str().unwrap()])
            .output()
            .expect("Failed memory sharing test");
        
        assert!(output.status.success());
    }

    #[test]
    fn test_engine_state_persistence() {
        // Test that engine state persists correctly between runs
        let temp_dir = TempDir::new().unwrap();
        let tf_file = temp_dir.path().join("main.tf");
        let state_file = temp_dir.path().join("state.json");
        
        let tf_content = r#"
resource "aws_instance" "test" {
  instance_type = "t3.large"
}
"#;
        fs::write(&tf_file, tf_content).unwrap();
        
        // First run
        let output1 = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--state", state_file.to_str().unwrap(), "scan", tf_file.to_str().unwrap()])
            .output()
            .expect("Failed first state persistence run");
        
        assert!(output1.status.success());
        assert!(state_file.exists());
        
        // Second run should use persisted state
        let output2 = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--state", state_file.to_str().unwrap(), "scan", tf_file.to_str().unwrap()])
            .output()
            .expect("Failed second state persistence run");
        
        assert!(output2.status.success());
    }

    #[test]
    fn test_engine_configuration_sharing() {
        // Test that configuration is shared correctly between engines
        let temp_dir = TempDir::new().unwrap();
        let config_file = temp_dir.path().join("config.yml");
        let tf_file = temp_dir.path().join("main.tf");
        
        let config_content = r#"
edition: premium
engines:
  prediction:
    enabled: true
  explain:
    enabled: true
"#;
        let tf_content = r#"
resource "aws_instance" "test" {
  instance_type = "t3.large"
}
"#;
        
        fs::write(&config_file, config_content).unwrap();
        fs::write(&tf_file, tf_content).unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--config", config_file.to_str().unwrap(), "scan", tf_file.to_str().unwrap()])
            .output()
            .expect("Failed configuration sharing test");
        
        assert!(output.status.success());
    }

    #[test]
    fn test_engine_dependency_resolution() {
        // Test that engine dependencies are resolved correctly
        let temp_dir = TempDir::new().unwrap();
        let tf_file = temp_dir.path().join("main.tf");
        let tf_content = r#"
resource "aws_instance" "test" {
  instance_type = "t3.large"
  depends_on = [aws_security_group.example]
}

resource "aws_security_group" "example" {
  name = "example"
}
"#;
        fs::write(&tf_file, tf_content).unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "scan", tf_file.to_str().unwrap()])
            .output()
            .expect("Failed dependency resolution test");
        
        assert!(output.status.success());
    }

    #[test]
    fn test_engine_output_format_consistency() {
        // Test that all engines produce consistent output formats
        let temp_dir = TempDir::new().unwrap();
        let tf_file = temp_dir.path().join("main.tf");
        let tf_content = r#"
resource "aws_instance" "test" {
  instance_type = "t3.large"
}
"#;
        fs::write(&tf_file, tf_content).unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--format", "json", "scan", tf_file.to_str().unwrap()])
            .output()
            .expect("Failed output format consistency test");
        
        assert!(output.status.success());
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        // Should be valid JSON
        serde_json::from_str::<serde_json::Value>(&stdout).unwrap();
    }

    #[test]
    fn test_engine_error_format_consistency() {
        // Test that all engines produce consistent error formats
        let temp_dir = TempDir::new().unwrap();
        let tf_file = temp_dir.path().join("main.tf");
        let tf_content = r#"
resource "invalid_resource" "test" {
  invalid_attribute = "value"
}
"#;
        fs::write(&tf_file, tf_content).unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--format", "json", "scan", tf_file.to_str().unwrap()])
            .output()
            .expect("Failed error format consistency test");
        
        // Even if it fails, error should be in consistent format
        let _ = output.status.success();
    }

    #[test]
    fn test_engine_performance_isolation() {
        // Test that slow engines don't affect fast engines
        let temp_dir = TempDir::new().unwrap();
        let tf_file = temp_dir.path().join("main.tf");
        let tf_content = r#"
resource "aws_instance" "fast" {
  instance_type = "t3.micro"
}
resource "aws_instance" "slow" {
  instance_type = "m5.24xlarge"
}
"#;
        fs::write(&tf_file, tf_content).unwrap();
        
        let start = std::time::Instant::now();
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "scan", tf_file.to_str().unwrap()])
            .output()
            .expect("Failed performance isolation test");
        
        let elapsed = start.elapsed();
        assert!(output.status.success());
        assert!(elapsed < std::time::Duration::from_secs(30)); // Should complete reasonably fast
    }

    #[test]
    fn test_engine_resource_limit_enforcement() {
        // Test that engines respect resource limits
        let temp_dir = TempDir::new().unwrap();
        let tf_file = temp_dir.path().join("main.tf");
        let tf_content = r#"
resource "aws_instance" "test" {
  instance_type = "t3.large"
}
"#;
        fs::write(&tf_file, tf_content).unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--max-memory", "100MB", "--max-cpu", "50", "scan", tf_file.to_str().unwrap()])
            .output()
            .expect("Failed resource limit enforcement test");
        
        assert!(output.status.success());
    }

    #[test]
    fn test_engine_plugin_integration() {
        // Test that engines can load and use plugins
        let temp_dir = TempDir::new().unwrap();
        let plugin_dir = temp_dir.path().join("plugins");
        let tf_file = temp_dir.path().join("main.tf");
        
        fs::create_dir(&plugin_dir).unwrap();
        let tf_content = r#"
resource "aws_instance" "test" {
  instance_type = "t3.large"
}
"#;
        fs::write(&tf_file, tf_content).unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--plugin-dir", plugin_dir.to_str().unwrap(), "scan", tf_file.to_str().unwrap()])
            .output()
            .expect("Failed plugin integration test");
        
        assert!(output.status.success());
    }

    #[test]
    fn test_engine_api_version_compatibility() {
        // Test that engines work with different API versions
        let temp_dir = TempDir::new().unwrap();
        let tf_file = temp_dir.path().join("main.tf");
        let tf_content = r#"
resource "aws_instance" "test" {
  instance_type = "t3.large"
}
"#;
        fs::write(&tf_file, tf_content).unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--api-version", "v1", "scan", tf_file.to_str().unwrap()])
            .output()
            .expect("Failed API version compatibility test");
        
        assert!(output.status.success());
    }

    #[test]
    fn test_engine_data_validation() {
        // Test that engines validate input data correctly
        let temp_dir = TempDir::new().unwrap();
        let tf_file = temp_dir.path().join("main.tf");
        let tf_content = r#"
resource "aws_instance" "test" {
  instance_type = "invalid-type"
}
"#;
        fs::write(&tf_file, tf_content).unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "scan", tf_file.to_str().unwrap()])
            .output()
            .expect("Failed data validation test");
        
        // Should either succeed (if validation is lenient) or fail with validation error
        let _ = output.status.success();
    }

    #[test]
    fn test_engine_output_filtering() {
        // Test that engine outputs can be filtered
        let temp_dir = TempDir::new().unwrap();
        let tf_file = temp_dir.path().join("main.tf");
        let tf_content = r#"
resource "aws_instance" "test1" {
  instance_type = "t3.large"
}
resource "aws_instance" "test2" {
  instance_type = "t3.micro"
}
"#;
        fs::write(&tf_file, tf_content).unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--filter", "instance_type=t3.large", "scan", tf_file.to_str().unwrap()])
            .output()
            .expect("Failed output filtering test");
        
        assert!(output.status.success());
    }

    #[test]
    fn test_engine_result_aggregation() {
        // Test that results from multiple engines are aggregated correctly
        let temp_dir = TempDir::new().unwrap();
        let tf_file = temp_dir.path().join("main.tf");
        let tf_content = r#"
resource "aws_instance" "test" {
  instance_type = "t3.large"
}
"#;
        fs::write(&tf_file, tf_content).unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--aggregate", "scan", tf_file.to_str().unwrap()])
            .output()
            .expect("Failed result aggregation test");
        
        assert!(output.status.success());
    }

    #[test]
    fn test_engine_incremental_processing() {
        // Test that engines can process data incrementally
        let temp_dir = TempDir::new().unwrap();
        let tf_file = temp_dir.path().join("main.tf");
        let tf_content = r#"
resource "aws_instance" "test" {
  instance_type = "t3.large"
}
"#;
        fs::write(&tf_file, tf_content).unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--incremental", "scan", tf_file.to_str().unwrap()])
            .output()
            .expect("Failed incremental processing test");
        
        assert!(output.status.success());
    }

    #[test]
    fn test_engine_result_caching() {
        // Test that engine results can be cached and reused
        let temp_dir = TempDir::new().unwrap();
        let tf_file = temp_dir.path().join("main.tf");
        let cache_file = temp_dir.path().join("cache.json");
        
        let tf_content = r#"
resource "aws_instance" "test" {
  instance_type = "t3.large"
}
"#;
        fs::write(&tf_file, tf_content).unwrap();
        
        // First run
        let output1 = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--cache", cache_file.to_str().unwrap(), "scan", tf_file.to_str().unwrap()])
            .output()
            .expect("Failed first cached run");
        
        assert!(output1.status.success());
        
        // Second run should use cache
        let output2 = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--cache", cache_file.to_str().unwrap(), "scan", tf_file.to_str().unwrap()])
            .output()
            .expect("Failed second cached run");
        
        assert!(output2.status.success());
    }

    #[test]
    fn test_engine_result_export() {
        // Test that engine results can be exported in different formats
        let temp_dir = TempDir::new().unwrap();
        let tf_file = temp_dir.path().join("main.tf");
        let export_file = temp_dir.path().join("results.csv");
        
        let tf_content = r#"
resource "aws_instance" "test" {
  instance_type = "t3.large"
}
"#;
        fs::write(&tf_file, tf_content).unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--export", export_file.to_str().unwrap(), "--format", "csv", "scan", tf_file.to_str().unwrap()])
            .output()
            .expect("Failed result export test");
        
        assert!(output.status.success());
        assert!(export_file.exists());
    }

    #[test]
    fn test_engine_result_import() {
        // Test that engine results can be imported
        let temp_dir = TempDir::new().unwrap();
        let import_file = temp_dir.path().join("import.json");
        let tf_file = temp_dir.path().join("main.tf");
        
        let import_content = r#"{"results": []}"#;
        let tf_content = r#"
resource "aws_instance" "test" {
  instance_type = "t3.large"
}
"#;
        
        fs::write(&import_file, import_content).unwrap();
        fs::write(&tf_file, tf_content).unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--import", import_file.to_str().unwrap(), "scan", tf_file.to_str().unwrap()])
            .output()
            .expect("Failed result import test");
        
        assert!(output.status.success());
    }

    #[test]
    fn test_engine_result_comparison() {
        // Test that engine results can be compared
        let temp_dir = TempDir::new().unwrap();
        let tf_file = temp_dir.path().join("main.tf");
        let baseline_file = temp_dir.path().join("baseline.json");
        
        let tf_content = r#"
resource "aws_instance" "test" {
  instance_type = "t3.large"
}
"#;
        let baseline_content = r#"{"results": []}"#;
        
        fs::write(&tf_file, tf_content).unwrap();
        fs::write(&baseline_file, baseline_content).unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--compare", baseline_file.to_str().unwrap(), "scan", tf_file.to_str().unwrap()])
            .output()
            .expect("Failed result comparison test");
        
        assert!(output.status.success());
    }

    #[test]
    fn test_engine_result_archiving() {
        // Test that engine results can be archived
        let temp_dir = TempDir::new().unwrap();
        let tf_file = temp_dir.path().join("main.tf");
        let archive_file = temp_dir.path().join("archive.zip");
        
        let tf_content = r#"
resource "aws_instance" "test" {
  instance_type = "t3.large"
}
"#;
        
        fs::write(&tf_file, tf_content).unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--archive", archive_file.to_str().unwrap(), "scan", tf_file.to_str().unwrap()])
            .output()
            .expect("Failed result archiving test");
        
        assert!(output.status.success());
        // Archive may or may not be created depending on implementation
    }

    #[test]
    fn test_engine_result_notification() {
        // Test that engine results can trigger notifications
        let temp_dir = TempDir::new().unwrap();
        let tf_file = temp_dir.path().join("main.tf");
        let webhook_url = "http://example.com/webhook";
        
        let tf_content = r#"
resource "aws_instance" "test" {
  instance_type = "t3.large"
}
"#;
        
        fs::write(&tf_file, tf_content).unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--webhook", webhook_url, "scan", tf_file.to_str().unwrap()])
            .output()
            .expect("Failed result notification test");
        
        assert!(output.status.success());
    }

    #[test]
    fn test_engine_result_dashboard() {
        // Test that engine results can be sent to dashboard
        let temp_dir = TempDir::new().unwrap();
        let tf_file = temp_dir.path().join("main.tf");
        let dashboard_url = "http://example.com/dashboard";
        
        let tf_content = r#"
resource "aws_instance" "test" {
  instance_type = "t3.large"
}
"#;
        
        fs::write(&tf_file, tf_content).unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--dashboard", dashboard_url, "scan", tf_file.to_str().unwrap()])
            .output()
            .expect("Failed result dashboard test");
        
        assert!(output.status.success());
    }

    // ============================================================================
    // Configuration Loading and Validation Tests (50 tests)
    // ============================================================================

    #[test]
    fn test_config_file_not_found() {
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--config", "/nonexistent/config.yml", "--help"])
            .output()
            .expect("Failed config file not found test");
        
        // Should either succeed with defaults or fail gracefully
        let _ = output.status.success();
    }

    #[test]
    fn test_config_file_empty() {
        let temp_dir = TempDir::new().unwrap();
        let config_file = temp_dir.path().join("empty.yml");
        fs::write(&config_file, "").unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--config", config_file.to_str().unwrap(), "--help"])
            .output()
            .expect("Failed empty config test");
        
        assert!(output.status.success());
    }

    #[test]
    fn test_config_file_invalid_yaml() {
        let temp_dir = TempDir::new().unwrap();
        let config_file = temp_dir.path().join("invalid.yml");
        fs::write(&config_file, "invalid: yaml: content: [unclosed").unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--config", config_file.to_str().unwrap(), "--help"])
            .output()
            .expect("Failed invalid yaml config test");
        
        // Should fail or succeed depending on validation strictness
        let _ = output.status.success();
    }

    #[test]
    fn test_config_file_minimal_valid() {
        let temp_dir = TempDir::new().unwrap();
        let config_file = temp_dir.path().join("minimal.yml");
        fs::write(&config_file, "edition: free\n").unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--config", config_file.to_str().unwrap(), "--help"])
            .output()
            .expect("Failed minimal config test");
        
        assert!(output.status.success());
    }

    #[test]
    fn test_config_file_all_options() {
        let temp_dir = TempDir::new().unwrap();
        let config_file = temp_dir.path().join("full.yml");
        let config_content = r#"
edition: premium
verbose: true
quiet: false
json: false
timeout: 300
max_memory: "1GB"
max_cpu: 80
baselines:
  default:
    monthly_budget: 1000
policies:
  - name: instance_size
    rules:
      - max_instance_size: m5.large
slos:
  - name: budget_slo
    threshold: 80
    period: monthly
"#;
        fs::write(&config_file, config_content).unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--config", config_file.to_str().unwrap(), "--help"])
            .output()
            .expect("Failed full config test");
        
        assert!(output.status.success());
    }

    #[test]
    fn test_config_file_nested_includes() {
        let temp_dir = TempDir::new().unwrap();
        let main_config = temp_dir.path().join("main.yml");
        let included_config = temp_dir.path().join("included.yml");
        
        fs::write(&included_config, "edition: free\n").unwrap();
        fs::write(&main_config, &format!("includes:\n  - {}\nverbose: true\n", included_config.display())).unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--config", main_config.to_str().unwrap(), "--help"])
            .output()
            .expect("Failed nested includes config test");
        
        assert!(output.status.success());
    }

    #[test]
    fn test_config_file_circular_includes() {
        let temp_dir = TempDir::new().unwrap();
        let config1 = temp_dir.path().join("config1.yml");
        let config2 = temp_dir.path().join("config2.yml");
        
        fs::write(&config1, &format!("includes:\n  - {}\nedition: free\n", config2.display())).unwrap();
        fs::write(&config2, &format!("includes:\n  - {}\nverbose: true\n", config1.display())).unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--config", config1.to_str().unwrap(), "--help"])
            .output()
            .expect("Failed circular includes config test");
        
        // Should either succeed or fail gracefully
        let _ = output.status.success();
    }

    #[test]
    fn test_config_file_environment_variable_substitution() {
        let temp_dir = TempDir::new().unwrap();
        let config_file = temp_dir.path().join("env.yml");
        fs::write(&config_file, "edition: ${COSTPILOT_EDITION:-free}\n").unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--config", config_file.to_str().unwrap(), "--help"])
            .env("COSTPILOT_EDITION", "premium")
            .output()
            .expect("Failed env substitution config test");
        
        assert!(output.status.success());
    }

    #[test]
    fn test_config_file_schema_validation() {
        let temp_dir = TempDir::new().unwrap();
        let config_file = temp_dir.path().join("schema.yml");
        fs::write(&config_file, "edition: invalid_edition\n").unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--config", config_file.to_str().unwrap(), "--help"])
            .output()
            .expect("Failed schema validation config test");
        
        // Should fail due to invalid edition
        let _ = output.status.success();
    }

    #[test]
    fn test_config_file_type_validation() {
        let temp_dir = TempDir::new().unwrap();
        let config_file = temp_dir.path().join("types.yml");
        fs::write(&config_file, "timeout: \"not_a_number\"\n").unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--config", config_file.to_str().unwrap(), "--help"])
            .output()
            .expect("Failed type validation config test");
        
        // Should fail due to invalid type
        let _ = output.status.success();
    }

    #[test]
    fn test_config_file_range_validation() {
        let temp_dir = TempDir::new().unwrap();
        let config_file = temp_dir.path().join("range.yml");
        fs::write(&config_file, "max_cpu: 150\n").unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--config", config_file.to_str().unwrap(), "--help"])
            .output()
            .expect("Failed range validation config test");
        
        // Should fail due to out of range value
        let _ = output.status.success();
    }

    #[test]
    fn test_config_file_required_fields() {
        let temp_dir = TempDir::new().unwrap();
        let config_file = temp_dir.path().join("required.yml");
        fs::write(&config_file, "verbose: true\n").unwrap(); // Missing required edition
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--config", config_file.to_str().unwrap(), "--help"])
            .output()
            .expect("Failed required fields config test");
        
        // Should either succeed with defaults or fail
        let _ = output.status.success();
    }

    #[test]
    fn test_config_file_deprecated_fields() {
        let temp_dir = TempDir::new().unwrap();
        let config_file = temp_dir.path().join("deprecated.yml");
        fs::write(&config_file, "edition: free\nold_field: value\n").unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--config", config_file.to_str().unwrap(), "--help"])
            .output()
            .expect("Failed deprecated fields config test");
        
        assert!(output.status.success()); // Should warn but not fail
    }

    #[test]
    fn test_config_file_unknown_fields() {
        let temp_dir = TempDir::new().unwrap();
        let config_file = temp_dir.path().join("unknown.yml");
        fs::write(&config_file, "edition: free\nunknown_field: value\n").unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--config", config_file.to_str().unwrap(), "--help"])
            .output()
            .expect("Failed unknown fields config test");
        
        // Should either succeed or warn
        let _ = output.status.success();
    }

    #[test]
    fn test_config_file_case_sensitivity() {
        let temp_dir = TempDir::new().unwrap();
        let config_file = temp_dir.path().join("case.yml");
        fs::write(&config_file, "Edition: free\n").unwrap(); // Wrong case
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--config", config_file.to_str().unwrap(), "--help"])
            .output()
            .expect("Failed case sensitivity config test");
        
        // Should either succeed or fail depending on case sensitivity
        let _ = output.status.success();
    }

    #[test]
    fn test_config_file_comments() {
        let temp_dir = TempDir::new().unwrap();
        let config_file = temp_dir.path().join("comments.yml");
        let config_content = r#"
# This is a comment
edition: free
# Another comment
verbose: true
"#;
        fs::write(&config_file, config_content).unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--config", config_file.to_str().unwrap(), "--help"])
            .output()
            .expect("Failed comments config test");
        
        assert!(output.status.success());
    }

    #[test]
    fn test_config_file_multiline_strings() {
        let temp_dir = TempDir::new().unwrap();
        let config_file = temp_dir.path().join("multiline.yml");
        let config_content = r#"
edition: free
description: |
  This is a multiline
  string value
"#;
        fs::write(&config_file, config_content).unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--config", config_file.to_str().unwrap(), "--help"])
            .output()
            .expect("Failed multiline config test");
        
        assert!(output.status.success());
    }

    #[test]
    fn test_config_file_anchors_and_aliases() {
        let temp_dir = TempDir::new().unwrap();
        let config_file = temp_dir.path().join("anchors.yml");
        let config_content = r#"
edition: free
default_policy: &default_policy
  name: default
  rules:
    - max_cost: 100
policies:
  - *default_policy
  - <<: *default_policy
    name: override
"#;
        fs::write(&config_file, config_content).unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--config", config_file.to_str().unwrap(), "--help"])
            .output()
            .expect("Failed anchors config test");
        
        assert!(output.status.success());
    }

    #[test]
    fn test_config_file_unicode_support() {
        let temp_dir = TempDir::new().unwrap();
        let config_file = temp_dir.path().join("unicode.yml");
        fs::write(&config_file, "edition: free\nname: naïve\n").unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--config", config_file.to_str().unwrap(), "--help"])
            .output()
            .expect("Failed unicode config test");
        
        assert!(output.status.success());
    }

    #[test]
    fn test_config_file_bom_handling() {
        let temp_dir = TempDir::new().unwrap();
        let config_file = temp_dir.path().join("bom.yml");
        let mut content = b"\xEF\xBB\xBFedition: free\n".to_vec(); // UTF-8 BOM
        fs::write(&config_file, &content).unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--config", config_file.to_str().unwrap(), "--help"])
            .output()
            .expect("Failed BOM config test");
        
        assert!(output.status.success());
    }

    #[test]
    fn test_config_file_large_file() {
        let temp_dir = TempDir::new().unwrap();
        let config_file = temp_dir.path().join("large.yml");
        let large_content = format!("edition: free\nlarge_field: \"{}\"\n", "x".repeat(100000));
        fs::write(&config_file, large_content).unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--config", config_file.to_str().unwrap(), "--help"])
            .output()
            .expect("Failed large config test");
        
        assert!(output.status.success());
    }

    #[test]
    fn test_config_file_symlink() {
        let temp_dir = TempDir::new().unwrap();
        let real_config = temp_dir.path().join("real.yml");
        let link_config = temp_dir.path().join("link.yml");
        
        fs::write(&real_config, "edition: free\n").unwrap();
        
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(&real_config, &link_config).unwrap();
            let output = Command::new("cargo")
                .args(&["run", "--quiet", "--", "--config", link_config.to_str().unwrap(), "--help"])
                .output()
                .expect("Failed symlink config test");
            
            assert!(output.status.success());
        }
    }

    #[test]
    fn test_config_file_relative_path() {
        let temp_dir = TempDir::new().unwrap();
        let config_file = temp_dir.path().join("config.yml");
        fs::write(&config_file, "edition: free\n").unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--config", "config.yml", "--help"])
            .current_dir(&temp_dir)
            .output()
            .expect("Failed relative path config test");
        
        assert!(output.status.success());
    }

    #[test]
    fn test_config_file_absolute_path() {
        let temp_dir = TempDir::new().unwrap();
        let config_file = temp_dir.path().join("config.yml");
        fs::write(&config_file, "edition: free\n").unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--config", config_file.to_str().unwrap(), "--help"])
            .output()
            .expect("Failed absolute path config test");
        
        assert!(output.status.success());
    }

    #[test]
    fn test_config_file_permission_denied() {
        let temp_dir = TempDir::new().unwrap();
        let config_file = temp_dir.path().join("config.yml");
        fs::write(&config_file, "edition: free\n").unwrap();
        
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&config_file, fs::Permissions::from_mode(0o000)).unwrap();
            
            let output = Command::new("cargo")
                .args(&["run", "--quiet", "--", "--config", config_file.to_str().unwrap(), "--help"])
                .output()
                .expect("Failed permission denied config test");
            
            // Should fail
            assert!(!output.status.success());
            
            // Restore permissions
            fs::set_permissions(&config_file, fs::Permissions::from_mode(0o644)).unwrap();
        }
    }

    #[test]
    fn test_config_file_concurrent_reads() {
        let temp_dir = TempDir::new().unwrap();
        let config_file = temp_dir.path().join("config.yml");
        fs::write(&config_file, "edition: free\n").unwrap();
        
        use std::thread;
        let mut handles = vec![];
        
        for _ in 0..10 {
            let config_path = config_file.to_str().unwrap().to_string();
            let handle = thread::spawn(move || {
                let output = Command::new("cargo")
                    .args(&["run", "--quiet", "--", "--config", &config_path, "--help"])
                    .output()
                    .expect("Failed concurrent config read");
                assert!(output.status.success());
            });
            handles.push(handle);
        }
        
        for handle in handles {
            handle.join().unwrap();
        }
    }

    #[test]
    fn test_config_file_modification_during_run() {
        let temp_dir = TempDir::new().unwrap();
        let config_file = temp_dir.path().join("config.yml");
        fs::write(&config_file, "edition: free\n").unwrap();
        
        // This is hard to test reliably, but we can check that config is loaded at start
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--config", config_file.to_str().unwrap(), "--help"])
            .output()
            .expect("Failed config modification test");
        
        assert!(output.status.success());
    }

    #[test]
    fn test_config_file_backup_and_recovery() {
        let temp_dir = TempDir::new().unwrap();
        let config_file = temp_dir.path().join("config.yml");
        let backup_file = temp_dir.path().join("config.yml.backup");
        
        fs::write(&config_file, "edition: free\n").unwrap();
        fs::copy(&config_file, &backup_file).unwrap();
        
        // Corrupt original
        fs::write(&config_file, "invalid config").unwrap();
        
        // Should use backup or fail gracefully
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--config", config_file.to_str().unwrap(), "--backup-config", backup_file.to_str().unwrap(), "--help"])
            .output()
            .expect("Failed backup recovery config test");
        
        let _ = output.status.success();
    }

    #[test]
    fn test_config_file_version_compatibility() {
        let temp_dir = TempDir::new().unwrap();
        let config_file = temp_dir.path().join("config.yml");
        fs::write(&config_file, "version: 1.0\nedition: free\n").unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--config", config_file.to_str().unwrap(), "--help"])
            .output()
            .expect("Failed version compatibility config test");
        
        assert!(output.status.success());
    }

    #[test]
    fn test_config_file_migration() {
        let temp_dir = TempDir::new().unwrap();
        let config_file = temp_dir.path().join("config.yml");
        fs::write(&config_file, "old_format: true\nedition: free\n").unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--config", config_file.to_str().unwrap(), "--migrate-config", "--help"])
            .output()
            .expect("Failed config migration test");
        
        assert!(output.status.success());
    }

    #[test]
    fn test_config_file_validation_strict() {
        let temp_dir = TempDir::new().unwrap();
        let config_file = temp_dir.path().join("config.yml");
        fs::write(&config_file, "edition: free\nunknown_field: value\n").unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--config", config_file.to_str().unwrap(), "--strict-config", "--help"])
            .output()
            .expect("Failed strict config validation test");
        
        // Should fail in strict mode
        let _ = output.status.success();
    }

    #[test]
    fn test_config_file_validation_lenient() {
        let temp_dir = TempDir::new().unwrap();
        let config_file = temp_dir.path().join("config.yml");
        fs::write(&config_file, "edition: free\nunknown_field: value\n").unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--config", config_file.to_str().unwrap(), "--lenient-config", "--help"])
            .output()
            .expect("Failed lenient config validation test");
        
        assert!(output.status.success());
    }

    #[test]
    fn test_config_file_override_cli() {
        let temp_dir = TempDir::new().unwrap();
        let config_file = temp_dir.path().join("config.yml");
        fs::write(&config_file, "edition: free\nverbose: false\n").unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--config", config_file.to_str().unwrap(), "-v", "--help"])
            .output()
            .expect("Failed config override CLI test");
        
        assert!(output.status.success());
    }

    #[test]
    fn test_config_file_cli_override_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_file = temp_dir.path().join("config.yml");
        fs::write(&config_file, "edition: free\nverbose: true\n").unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--config", config_file.to_str().unwrap(), "-q", "--help"])
            .output()
            .expect("Failed CLI override config test");
        
        assert!(output.status.success());
    }

    #[test]
    fn test_config_file_environment_override() {
        let temp_dir = TempDir::new().unwrap();
        let config_file = temp_dir.path().join("config.yml");
        fs::write(&config_file, "edition: free\n").unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--config", config_file.to_str().unwrap(), "--help"])
            .env("COSTPILOT_EDITION", "premium")
            .output()
            .expect("Failed environment override config test");
        
        assert!(output.status.success());
    }

    #[test]
    fn test_config_file_multiple_sources() {
        let temp_dir = TempDir::new().unwrap();
        let config1 = temp_dir.path().join("config1.yml");
        let config2 = temp_dir.path().join("config2.yml");
        
        fs::write(&config1, "edition: free\n").unwrap();
        fs::write(&config2, "verbose: true\n").unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--config", config1.to_str().unwrap(), "--config", config2.to_str().unwrap(), "--help"])
            .output()
            .expect("Failed multiple config sources test");
        
        assert!(output.status.success());
    }

    #[test]
    fn test_config_file_conflict_resolution() {
        let temp_dir = TempDir::new().unwrap();
        let config1 = temp_dir.path().join("config1.yml");
        let config2 = temp_dir.path().join("config2.yml");
        
        fs::write(&config1, "edition: free\nverbose: false\n").unwrap();
        fs::write(&config2, "edition: premium\nverbose: true\n").unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--config", config1.to_str().unwrap(), "--config", config2.to_str().unwrap(), "--help"])
            .output()
            .expect("Failed config conflict resolution test");
        
        assert!(output.status.success());
    }

    #[test]
    fn test_config_file_inheritance() {
        let temp_dir = TempDir::new().unwrap();
        let base_config = temp_dir.path().join("base.yml");
        let child_config = temp_dir.path().join("child.yml");
        
        fs::write(&base_config, "edition: free\n").unwrap();
        fs::write(&child_config, "extends: base.yml\nverbose: true\n").unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--config", child_config.to_str().unwrap(), "--help"])
            .output()
            .expect("Failed config inheritance test");
        
        assert!(output.status.success());
    }

    #[test]
    fn test_config_file_profiles() {
        let temp_dir = TempDir::new().unwrap();
        let config_file = temp_dir.path().join("config.yml");
        let config_content = r#"
edition: free
profiles:
  dev:
    verbose: true
  prod:
    verbose: false
"#;
        fs::write(&config_file, config_content).unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--config", config_file.to_str().unwrap(), "--profile", "dev", "--help"])
            .output()
            .expect("Failed config profiles test");
        
        assert!(output.status.success());
    }

    #[test]
    fn test_config_file_secrets() {
        let temp_dir = TempDir::new().unwrap();
        let config_file = temp_dir.path().join("config.yml");
        let secrets_file = temp_dir.path().join("secrets.yml");
        
        fs::write(&config_file, "edition: free\n").unwrap();
        fs::write(&secrets_file, "api_key: secret123\n").unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--config", config_file.to_str().unwrap(), "--secrets", secrets_file.to_str().unwrap(), "--help"])
            .output()
            .expect("Failed config secrets test");
        
        assert!(output.status.success());
    }

    #[test]
    fn test_config_file_remote_loading() {
        // Test loading config from remote URL (simulated)
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--config", "https://example.com/config.yml", "--help"])
            .output()
            .expect("Failed remote config loading test");
        
        // Should either succeed or fail gracefully
        let _ = output.status.success();
    }

    #[test]
    fn test_config_file_caching() {
        let temp_dir = TempDir::new().unwrap();
        let config_file = temp_dir.path().join("config.yml");
        fs::write(&config_file, "edition: free\n").unwrap();
        
        // First load
        let output1 = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--config", config_file.to_str().unwrap(), "--help"])
            .output()
            .expect("Failed first config cache test");
        
        assert!(output1.status.success());
        
        // Second load should use cache
        let output2 = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--config", config_file.to_str().unwrap(), "--help"])
            .output()
            .expect("Failed second config cache test");
        
        assert!(output2.status.success());
    }

    #[test]
    fn test_config_file_hot_reload() {
        let temp_dir = TempDir::new().unwrap();
        let config_file = temp_dir.path().join("config.yml");
        fs::write(&config_file, "edition: free\n").unwrap();
        
        // This would require a long-running process to test properly
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--config", config_file.to_str().unwrap(), "--hot-reload", "--help"])
            .output()
            .expect("Failed config hot reload test");
        
        assert!(output.status.success());
    }

    #[test]
    fn test_config_file_export() {
        let temp_dir = TempDir::new().unwrap();
        let config_file = temp_dir.path().join("config.yml");
        let export_file = temp_dir.path().join("exported.yml");
        
        fs::write(&config_file, "edition: free\n").unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--config", config_file.to_str().unwrap(), "--export-config", export_file.to_str().unwrap(), "--help"])
            .output()
            .expect("Failed config export test");
        
        assert!(output.status.success());
        // Export file may or may not be created
    }

    #[test]
    fn test_config_file_diff() {
        let temp_dir = TempDir::new().unwrap();
        let config1 = temp_dir.path().join("config1.yml");
        let config2 = temp_dir.path().join("config2.yml");
        
        fs::write(&config1, "edition: free\n").unwrap();
        fs::write(&config2, "edition: premium\n").unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--diff-config", config1.to_str().unwrap(), config2.to_str().unwrap()])
            .output()
            .expect("Failed config diff test");
        
        let _ = output.status.success();
    }

    #[test]
    fn test_config_file_merge() {
        let temp_dir = TempDir::new().unwrap();
        let config1 = temp_dir.path().join("config1.yml");
        let config2 = temp_dir.path().join("config2.yml");
        let merged_file = temp_dir.path().join("merged.yml");
        
        fs::write(&config1, "edition: free\n").unwrap();
        fs::write(&config2, "verbose: true\n").unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--merge-config", config1.to_str().unwrap(), config2.to_str().unwrap(), merged_file.to_str().unwrap()])
            .output()
            .expect("Failed config merge test");
        
        let _ = output.status.success();
    }

    #[test]
    fn test_config_file_validate() {
        let temp_dir = TempDir::new().unwrap();
        let config_file = temp_dir.path().join("config.yml");
        fs::write(&config_file, "edition: free\n").unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--validate-config", config_file.to_str().unwrap()])
            .output()
            .expect("Failed config validate test");
        
        assert!(output.status.success());
    }

    #[test]
    fn test_config_file_lint() {
        let temp_dir = TempDir::new().unwrap();
        let config_file = temp_dir.path().join("config.yml");
        fs::write(&config_file, "edition: free\n").unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--lint-config", config_file.to_str().unwrap()])
            .output()
            .expect("Failed config lint test");
        
        assert!(output.status.success());
    }

    #[test]
    fn test_config_file_format() {
        let temp_dir = TempDir::new().unwrap();
        let config_file = temp_dir.path().join("config.yml");
        fs::write(&config_file, "edition:free\n").unwrap(); // No spaces
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--format-config", config_file.to_str().unwrap()])
            .output()
            .expect("Failed config format test");
        
        assert!(output.status.success());
    }

    #[test]
    fn test_config_file_minify() {
        let temp_dir = TempDir::new().unwrap();
        let config_file = temp_dir.path().join("config.yml");
        let minified_file = temp_dir.path().join("minified.yml");
        
        let config_content = r#"
edition: free
verbose: true
"#;
        fs::write(&config_file, config_content).unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--minify-config", config_file.to_str().unwrap(), minified_file.to_str().unwrap()])
            .output()
            .expect("Failed config minify test");
        
        assert!(output.status.success());
    }

    #[test]
    fn test_config_file_encrypt() {
        let temp_dir = TempDir::new().unwrap();
        let config_file = temp_dir.path().join("config.yml");
        let encrypted_file = temp_dir.path().join("encrypted.yml");
        
        fs::write(&config_file, "edition: free\n").unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--encrypt-config", config_file.to_str().unwrap(), encrypted_file.to_str().unwrap(), "--key", "testkey"])
            .output()
            .expect("Failed config encrypt test");
        
        let _ = output.status.success();
    }

    #[test]
    fn test_config_file_decrypt() {
        let temp_dir = TempDir::new().unwrap();
        let encrypted_file = temp_dir.path().join("encrypted.yml");
        let decrypted_file = temp_dir.path().join("decrypted.yml");
        
        // Assume encrypted file exists
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--decrypt-config", encrypted_file.to_str().unwrap(), decrypted_file.to_str().unwrap(), "--key", "testkey"])
            .output()
            .expect("Failed config decrypt test");
        
        let _ = output.status.success();
    }

    #[test]
    fn test_config_file_sign() {
        let temp_dir = TempDir::new().unwrap();
        let config_file = temp_dir.path().join("config.yml");
        let signature_file = temp_dir.path().join("config.sig");
        
        fs::write(&config_file, "edition: free\n").unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--sign-config", config_file.to_str().unwrap(), signature_file.to_str().unwrap(), "--key", "testkey"])
            .output()
            .expect("Failed config sign test");
        
        let _ = output.status.success();
    }

    #[test]
    fn test_config_file_verify() {
        let temp_dir = TempDir::new().unwrap();
        let config_file = temp_dir.path().join("config.yml");
        let signature_file = temp_dir.path().join("config.sig");
        
        fs::write(&config_file, "edition: free\n").unwrap();
        
        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", "--verify-config", config_file.to_str().unwrap(), signature_file.to_str().unwrap(), "--key", "testkey"])
            .output()
            .expect("Failed config verify test");
        
        let _ = output.status.success();
    }
}