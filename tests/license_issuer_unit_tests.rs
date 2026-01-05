#[cfg(test)]
mod tests {
    use clap::{Arg, ArgMatches, Command};
    use costpilot::license_issuer::{generate_keypair, generate_license};
    use std::fs;

    // Helper to create mock ArgMatches for generate_keypair
    fn mock_keypair_matches(key_name: &str) -> ArgMatches {
        Command::new("test")
            .arg(Arg::new("key-name").default_value("test_key"))
            .get_matches_from(vec!["test", key_name])
    }

    // Helper to create mock ArgMatches for generate_license
    fn mock_license_matches(
        email: &str,
        license_key: &str,
        expires: &str,
        private_key_path: &str,
        issuer: Option<&str>,
        output: &str,
    ) -> ArgMatches {
        let mut args = vec![
            "test".to_string(),
            "--email".to_string(),
            email.to_string(),
            "--license-key".to_string(),
            license_key.to_string(),
            "--expires".to_string(),
            expires.to_string(),
            "--private-key".to_string(),
            private_key_path.to_string(),
            "--output".to_string(),
            output.to_string(),
        ];
        if let Some(iss) = issuer {
            args.push("--issuer".to_string());
            args.push(iss.to_string());
        }
        Command::new("test")
            .arg(Arg::new("email").short('e').long("email").required(true))
            .arg(
                Arg::new("license-key")
                    .short('k')
                    .long("license-key")
                    .required(true),
            )
            .arg(
                Arg::new("expires")
                    .short('x')
                    .long("expires")
                    .required(true),
            )
            .arg(
                Arg::new("private-key")
                    .short('p')
                    .long("private-key")
                    .required(true),
            )
            .arg(
                Arg::new("issuer")
                    .short('i')
                    .long("issuer")
                    .default_value("costpilot-v1"),
            )
            .arg(
                Arg::new("output")
                    .short('o')
                    .long("output")
                    .default_value("license.json"),
            )
            .get_matches_from(args)
    }

    #[test]
    fn test_generate_keypair_creates_files() {
        let temp_dir = tempfile::tempdir().unwrap();

        let matches = mock_keypair_matches("test_key");
        let result = generate_keypair(&matches, temp_dir.path());
        assert!(result.is_ok());

        assert!(fs::metadata(temp_dir.path().join("test_key.pem")).is_ok());
        assert!(fs::metadata(temp_dir.path().join("test_key.pub.pem")).is_ok());

        // Check private key is 32 bytes
        let private_data = fs::read(temp_dir.path().join("test_key.pem")).unwrap();
        assert_eq!(private_data.len(), 32);

        // Check public key format
        let public_data = fs::read_to_string(temp_dir.path().join("test_key.pub.pem")).unwrap();
        assert!(public_data.contains("-----BEGIN PUBLIC KEY-----"));
        assert!(public_data.contains("-----END PUBLIC KEY-----"));
    }

    #[test]
    fn test_generate_keypair_uniqueness() {
        let temp_dir = tempfile::tempdir().unwrap();

        let matches1 = mock_keypair_matches("key1");
        generate_keypair(&matches1, temp_dir.path()).unwrap();
        let matches2 = mock_keypair_matches("key2");
        generate_keypair(&matches2, temp_dir.path()).unwrap();

        let key1 = fs::read(temp_dir.path().join("key1.pem")).unwrap();
        let key2 = fs::read(temp_dir.path().join("key2.pem")).unwrap();
        assert_ne!(key1, key2);
    }

    #[test]
    fn test_generate_license_valid_output() {
        let temp_dir = tempfile::tempdir().unwrap();
        std::env::set_current_dir(&temp_dir).unwrap();

        // First generate a keypair
        let key_matches = mock_keypair_matches("license_key");
        generate_keypair(&key_matches, temp_dir.path()).unwrap();

        // Now generate license
        let license_matches = mock_license_matches(
            "test@example.com",
            "ABC123",
            "2026-12-31T23:59:59Z",
            "license_key.pem",
            Some("costpilot-v1"),
            "test_license.json",
        );
        let result = generate_license(&license_matches, temp_dir.path());
        assert!(result.is_ok());

        // Check file exists
        assert!(fs::metadata(temp_dir.path().join("test_license.json")).is_ok());

        // Parse JSON and validate structure
        let content = fs::read_to_string(temp_dir.path().join("test_license.json")).unwrap();
        let license: serde_json::Value = serde_json::from_str(&content).unwrap();

        assert_eq!(license["email"], "test@example.com");
        assert_eq!(license["license_key"], "ABC123");
        assert_eq!(license["expires"], "2026-12-31T23:59:59Z");
        assert_eq!(license["version"], "1.0");
        assert_eq!(license["issuer"], "costpilot-v1");
        assert!(license["issued_at"].is_string());
        assert!(license["signature"].is_string());

        // Validate signature (basic check: hex string of correct length)
        let sig = license["signature"].as_str().unwrap();
        assert_eq!(sig.len(), 128); // 64 bytes * 2 hex chars
        assert!(sig.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_generate_license_invalid_key() {
        let temp_dir = tempfile::tempdir().unwrap();

        // Create invalid key file (wrong length)
        fs::write(temp_dir.path().join("bad_key.pem"), b"short").unwrap();

        let matches = mock_license_matches(
            "test@example.com",
            "ABC123",
            "2026-12-31T23:59:59Z",
            "bad_key.pem",
            None,
            "test_license.json",
        );
        let result = generate_license(&matches, temp_dir.path());
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid key length"));
    }

    #[test]
    fn test_generate_license_missing_key_file() {
        let temp_dir = tempfile::tempdir().unwrap();

        let matches = mock_license_matches(
            "test@example.com",
            "ABC123",
            "2026-12-31T23:59:59Z",
            "nonexistent.pem",
            None,
            "test_license.json",
        );
        let result = generate_license(&matches, temp_dir.path());
        assert!(result.is_err());
    }
}
