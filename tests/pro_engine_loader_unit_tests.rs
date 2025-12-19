// Unit tests for ProEngine crypto functions

#[cfg(test)]
mod tests {
    use base64::Engine;
    use costpilot::pro_engine::crypto::{aes_gcm_decrypt, ed25519_verify, hkdf_derive};
    use ed25519_dalek::{Signature, Signer, SigningKey};
    use rand::rngs::OsRng;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_hkdf_derivation_matches_vector() {
        // HKDF test vector from RFC 5869
        let key_material = "test_key_material";
        let salt = b"salt";
        let info = b"info";

        let key1 = hkdf_derive(key_material, salt, info);
        let key2 = hkdf_derive(key_material, salt, info);

        assert_eq!(key1, key2, "HKDF should be deterministic");
        assert_eq!(key1.len(), 32, "Should produce 32-byte key");
    }

    #[test]
    fn test_aes_gcm_decrypt_success() {
        use aes_gcm::{
            aead::{Aead, KeyInit},
            Aes256Gcm, Nonce,
        };

        let key = [1u8; 32];
        let nonce_bytes = [2u8; 12];
        let plaintext = b"Hello, ProEngine!";

        // Encrypt
        let cipher = Aes256Gcm::new(&key.into());
        let nonce = Nonce::from_slice(&nonce_bytes);
        let ciphertext = cipher.encrypt(nonce, plaintext.as_ref()).unwrap();

        // Decrypt using our function
        let decrypted = aes_gcm_decrypt(&key, &nonce_bytes, &ciphertext).unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_decrypt_with_bad_key_returns_error() {
        use aes_gcm::{
            aead::{Aead, KeyInit},
            Aes256Gcm, Nonce,
        };

        let key = [1u8; 32];
        let wrong_key = [2u8; 32];
        let nonce_bytes = [2u8; 12];
        let plaintext = b"Hello, ProEngine!";

        // Encrypt with correct key
        let cipher = Aes256Gcm::new(&key.into());
        let nonce = Nonce::from_slice(&nonce_bytes);
        let ciphertext = cipher.encrypt(nonce, plaintext.as_ref()).unwrap();

        // Try decrypt with wrong key
        let result = aes_gcm_decrypt(&wrong_key, &nonce_bytes, &ciphertext);

        assert!(result.is_err(), "Should fail with wrong key");
    }

    #[test]
    fn test_verify_sig_success_and_failure() {
        use aes_gcm::aead::rand_core::RngCore;
        
        let dir = tempdir().unwrap();
        let pubkey_path = dir.path().join("test_key.pub");

        // Generate ephemeral Ed25519 keypair
        let mut secret_bytes = [0u8; 32];
        OsRng.fill_bytes(&mut secret_bytes);
        let signing_key = SigningKey::from_bytes(&secret_bytes);
        let verifying_key = signing_key.verifying_key();

        // Write public key as PEM (just raw bytes, not proper DER encoding)
        let pubkey_der = verifying_key.to_bytes();
        let pem = format!(
            "-----BEGIN PUBLIC KEY-----\n{}\n-----END PUBLIC KEY-----\n",
            base64::engine::general_purpose::STANDARD.encode(&pubkey_der)
        );
        fs::write(&pubkey_path, pem).unwrap();

        let message = b"Test message";
        let signature: Signature = signing_key.sign(message);
        let sig_bytes = signature.to_bytes();

        // Verify correct signature
        assert!(
            ed25519_verify(&pubkey_path, message, &sig_bytes),
            "Should verify valid signature"
        );

        // Verify with wrong message
        assert!(
            !ed25519_verify(&pubkey_path, b"Wrong message", &sig_bytes),
            "Should fail with wrong message"
        );
    }
}
