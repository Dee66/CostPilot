#[cfg(test)]
mod tests {
    use base64::Engine;
    use ed25519_dalek::Verifier;
    use std::fs;
    use std::process::Command;
    use tempfile::tempdir;

    #[test]
    fn test_cli_generate_keypair() {
        let temp_dir = tempdir().unwrap();
        let private_key = temp_dir.path().join("test_key.pem");
        let public_key = temp_dir.path().join("test_key.pub.pem");

        let output =
            Command::new("/home/dee/workspace/AI/GuardSuite/CostPilot/target/debug/license-issuer")
                .args(["generate-key", "test_key"])
                .current_dir(&temp_dir)
                .output()
                .expect("Failed to run command");

        assert!(output.status.success());
        assert!(private_key.exists());
        assert!(public_key.exists());

        // Check private key is 32 bytes
        let private_data = fs::read(&private_key).unwrap();
        assert_eq!(private_data.len(), 32);

        // Check public key format
        let public_data = fs::read_to_string(&public_key).unwrap();
        assert!(public_data.contains("-----BEGIN PUBLIC KEY-----"));
        assert!(public_data.contains("-----END PUBLIC KEY-----"));
    }

    #[test]
    fn test_cli_generate_license() {
        let temp_dir = tempdir().unwrap();

        let key_output =
            Command::new("/home/dee/workspace/AI/GuardSuite/CostPilot/target/debug/license-issuer")
                .args(["generate-key", "license_key"])
                .current_dir(&temp_dir)
                .output()
                .expect("Failed to generate keypair");
        assert!(key_output.status.success());

        // Now generate license
        let license_output =
            Command::new("/home/dee/workspace/AI/GuardSuite/CostPilot/target/debug/license-issuer")
                .args([
                    "generate-license",
                    "--email",
                    "test@example.com",
                    "--license-key",
                    "ABC123",
                    "--expires",
                    "2026-12-31T23:59:59Z",
                    "--private-key",
                    "license_key.pem",
                    "--output",
                    "test_license.json",
                ])
                .current_dir(&temp_dir)
                .output()
                .expect("Failed to generate license");

        assert!(license_output.status.success());

        let license_file = temp_dir.path().join("test_license.json");
        assert!(license_file.exists());

        // Parse and validate JSON
        let content = fs::read_to_string(&license_file).unwrap();
        let license: serde_json::Value = serde_json::from_str(&content).unwrap();

        assert_eq!(license["email"], "test@example.com");
        assert_eq!(license["license_key"], "ABC123");
        assert_eq!(license["expires"], "2026-12-31T23:59:59Z");
        assert_eq!(license["version"], "1.0");
        assert!(license["issued_at"].is_string());
        assert!(license["signature"].is_string());
    }

    #[test]
    fn test_cli_generate_license_invalid_key() {
        let temp_dir = tempdir().unwrap();

        // Create invalid key
        fs::write(temp_dir.path().join("bad_key.pem"), b"short").unwrap();

        let output =
            Command::new("/home/dee/workspace/AI/GuardSuite/CostPilot/target/debug/license-issuer")
                .args([
                    "generate-license",
                    "--email",
                    "test@example.com",
                    "--license-key",
                    "ABC123",
                    "--expires",
                    "2026-12-31T23:59:59Z",
                    "--private-key",
                    "bad_key.pem",
                    "--output",
                    "test_license.json",
                ])
                .current_dir(&temp_dir)
                .output()
                .expect("Failed to run command");

        assert!(!output.status.success());
        assert!(String::from_utf8_lossy(&output.stderr).contains("Invalid key length"));
    }

    #[test]
    fn test_end_to_end_integration_with_validation() {
        let temp_dir = tempdir().unwrap();

        // Generate keypair
        let key_output =
            Command::new("/home/dee/workspace/AI/GuardSuite/CostPilot/target/debug/license-issuer")
                .args(["generate-key", "integration_key"])
                .current_dir(&temp_dir)
                .output()
                .expect("Failed to generate keypair");
        assert!(key_output.status.success());

        // Generate license
        let license_output =
            Command::new("/home/dee/workspace/AI/GuardSuite/CostPilot/target/debug/license-issuer")
                .args([
                    "generate-license",
                    "--email",
                    "integration@test.com",
                    "--license-key",
                    "XYZ789",
                    "--expires",
                    "2026-12-31T23:59:59Z",
                    "--private-key",
                    "integration_key.pem",
                    "--output",
                    "integration_license.json",
                    "--issuer",
                    "costpilot-v1",
                ])
                .current_dir(&temp_dir)
                .output()
                .expect("Failed to generate license");
        assert!(license_output.status.success());

        // Now, load and validate the license using the licensing code
        // This simulates what the main binary does
        let license_path = temp_dir.path().join("integration_license.json");
        let license_content = fs::read_to_string(&license_path).unwrap();
        let license: serde_json::Value = serde_json::from_str(&license_content).unwrap();

        // Extract fields
        let email = license["email"].as_str().unwrap();
        let license_key = license["license_key"].as_str().unwrap();
        let expires = license["expires"].as_str().unwrap();
        let issuer = license["issuer"].as_str().unwrap();
        let signature_hex = license["signature"].as_str().unwrap();

        // Reconstruct canonical message
        let canonical_message = format!("{}|{}|{}|{}", email, license_key, expires, issuer);

        // Load public key
        let public_key_path = temp_dir.path().join("integration_key.pub.pem");
        let public_key_content = fs::read_to_string(&public_key_path).unwrap();
        // Extract the base64 part
        let base64_start = public_key_content
            .find("-----BEGIN PUBLIC KEY-----\n")
            .unwrap()
            + 27;
        let base64_end = public_key_content
            .find("\n-----END PUBLIC KEY-----")
            .unwrap();
        let public_key_b64 = &public_key_content[base64_start..base64_end];
        let public_key_bytes = base64::engine::general_purpose::STANDARD
            .decode(public_key_b64)
            .unwrap();
        let verifying_key =
            ed25519_dalek::VerifyingKey::from_bytes(&public_key_bytes.try_into().unwrap()).unwrap();

        // Decode signature
        let signature_bytes = hex::decode(signature_hex).unwrap();
        let signature = ed25519_dalek::Signature::from_bytes(&signature_bytes.try_into().unwrap());

        // Verify signature
        assert!(verifying_key
            .verify(canonical_message.as_bytes(), &signature)
            .is_ok());

        // Check expiry (basic check, not past)
        let expires_dt = chrono::DateTime::parse_from_rfc3339(expires).unwrap();
        let now = chrono::Utc::now();
        assert!(expires_dt > now);

        // Check issuer
        assert_eq!(issuer, "costpilot-v1");
    }
}
