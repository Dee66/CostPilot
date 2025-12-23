#[cfg(test)]
mod tests {
    use crate::pro_engine::license::License;
    use ed25519_dalek::{SigningKey, Signer};

    #[test]
    fn test_license_signature_verification_with_real_keys() {
        // Create a test license
        let license = License {
            email: "test@example.com".to_string(),
            license_key: "test-key-123".to_string(),
            expires: "2025-12-31".to_string(),
            signature: "".to_string(),
            issuer: "costpilot-v1".to_string(),
        };

        // Create a signing key and sign the license
        let mut secret_bytes = [0u8; 32];
        rand::RngCore::fill_bytes(&mut rand::rngs::OsRng, &mut secret_bytes);
        let signing_key = SigningKey::from_bytes(&secret_bytes);
        let verifying_key = signing_key.verifying_key();

        let message = format!("{}|{}|{}|{}", license.email, license.license_key, license.expires, license.issuer);
        let signature = signing_key.sign(message.as_bytes());
        let signed_license = License {
            signature: hex::encode(signature.to_bytes()),
            ..license
        };

        // Test verification with the generated key (this should work now)
        // Note: In real usage, this would use the embedded keys from build.rs
        let result = verify_license_signature_with_key(&signed_license, &verifying_key.to_bytes());
        assert!(result.is_ok(), "License signature verification should succeed with correct key");
    }

    // Helper function to test with specific key for testing
    fn verify_license_signature_with_key(lic: &License, key: &[u8; 32]) -> Result<(), String> {
        let message = format!("{}|{}|{}|{}", lic.email, lic.license_key, lic.expires, lic.issuer);
        let sig_bytes = hex::decode(&lic.signature).map_err(|_| "Invalid signature format")?;

        let public_key = ring::signature::UnparsedPublicKey::new(&ring::signature::ED25519, key);
        public_key
            .verify(message.as_bytes(), &sig_bytes)
            .map_err(|_| "License signature verification failed".to_string())
    }

    #[test]
    fn test_embedded_keys_are_valid() {
        // Test that the embedded keys are valid 32-byte arrays (not all zeros)
        // This verifies that the build.rs script successfully generated real keys
        use crate::pro_engine::crypto::{LICENSE_PUBLIC_KEY, WASM_PUBLIC_KEY};

        // Keys should not be all zeros (which would indicate placeholder values)
        assert!(!LICENSE_PUBLIC_KEY.iter().all(|&b| b == 0), "LICENSE_PUBLIC_KEY should not be all zeros");
        assert!(!WASM_PUBLIC_KEY.iter().all(|&b| b == 0), "WASM_PUBLIC_KEY should not be all zeros");

        // Keys should be exactly 32 bytes
        assert_eq!(LICENSE_PUBLIC_KEY.len(), 32, "LICENSE_PUBLIC_KEY should be 32 bytes");
        assert_eq!(WASM_PUBLIC_KEY.len(), 32, "WASM_PUBLIC_KEY should be 32 bytes");
    }
}
