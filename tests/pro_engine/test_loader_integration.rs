use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use chrono::{Duration, Utc};
use costpilot::pro_engine::crypto::hkdf_derive;
use costpilot::pro_engine::loader::{LoaderError, ProEngineLoader};
use ed25519_dalek::{Signer, SigningKey};
use rand::rngs::OsRng;
use serde_json::json;
use std::fs;
use std::io::Write;
use tempfile::tempdir;

#[test]
fn test_load_pro_engine_success() {
    let temp_dir = tempdir().unwrap();
    let keys_dir = temp_dir.path();

    // Generate test keypair
    let mut csprng = OsRng;
    let signing_key = SigningKey::generate(&mut csprng);
    let verifying_key = ed25519_dalek::VerifyingKey::from(&signing_key);

    // Create license
    let license_key = "test-license-key-12345";
    let expires = (Utc::now() + Duration::days(30)).to_rfc3339();
    let license_json = json!({
        "license_key": license_key,
        "email": "test@example.com",
        "expires": expires,
        "signature": "placeholder",
        "machine_binding": null
    });

    let license_path = keys_dir.join("license.json");
    fs::write(&license_path, serde_json::to_string(&license_json).unwrap()).unwrap();

    // Create sample WASM bytes
    let wasm_data = b"fake wasm module data for testing";

    // Sign the WASM
    let signature = signing_key.sign(wasm_data);

    // Encrypt WASM
    let aes_key = hkdf_derive(license_key, b"costpilot-pro-engine", b"aes-gcm-key");
    let nonce_bytes = [2u8; 12];
    let cipher = Aes256Gcm::new(&aes_key.into());
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher.encrypt(nonce, wasm_data.as_ref()).unwrap();

    // Write encrypted bundle (nonce + ciphertext)
    let bundle_path = keys_dir.join("pro_engine.wasm.enc");
    let mut bundle_file = fs::File::create(&bundle_path).unwrap();
    bundle_file.write_all(&nonce_bytes).unwrap();
    bundle_file.write_all(&ciphertext).unwrap();

    // Write signature
    let sig_path = keys_dir.join("pro_engine.wasm.sig");
    fs::write(&sig_path, signature.to_bytes()).unwrap();

    // Write public key PEM
    let pubkey_bytes = verifying_key.to_bytes();
    let der_bytes = [
        &[0x30, 0x2a, 0x30, 0x05, 0x06, 0x03, 0x2b, 0x65, 0x70, 0x03, 0x21, 0x00][..],
        &pubkey_bytes[..],
    ]
    .concat();

    let pem_content = format!(
        "-----BEGIN PUBLIC KEY-----\n{}\n-----END PUBLIC KEY-----\n",
        base64::encode(&der_bytes)
    );

    let pubkey_path = keys_dir.join("pubkey.pem");
    fs::write(&pubkey_path, pem_content).unwrap();

    // Load pro engine
    let loader = ProEngineLoader::new(keys_dir.to_path_buf());
    let result = loader.load_pro_engine(&license_path, &bundle_path, &pubkey_path);

    assert!(result.is_ok());
    let handle = result.unwrap();
    assert_eq!(handle.wasm_bytes, wasm_data);
}

#[test]
fn test_load_pro_engine_verify_error() {
    let temp_dir = tempdir().unwrap();
    let keys_dir = temp_dir.path();

    // Generate test keypair
    let mut csprng = OsRng;
    let signing_key = SigningKey::generate(&mut csprng);
    let verifying_key = ed25519_dalek::VerifyingKey::from(&signing_key);

    // Create license
    let license_key = "test-license-key-12345";
    let expires = (Utc::now() + Duration::days(30)).to_rfc3339();
    let license_json = json!({
        "license_key": license_key,
        "email": "test@example.com",
        "expires": expires,
        "signature": "placeholder",
        "machine_binding": null
    });

    let license_path = keys_dir.join("license.json");
    fs::write(&license_path, serde_json::to_string(&license_json).unwrap()).unwrap();

    // Create sample WASM bytes
    let wasm_data = b"fake wasm module data for testing";

    // Sign the WASM
    let signature = signing_key.sign(wasm_data);

    // Encrypt WASM
    let aes_key = hkdf_derive(license_key, b"costpilot-pro-engine", b"aes-gcm-key");
    let nonce_bytes = [2u8; 12];
    let cipher = Aes256Gcm::new(&aes_key.into());
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher.encrypt(nonce, wasm_data.as_ref()).unwrap();

    // Write encrypted bundle
    let bundle_path = keys_dir.join("pro_engine.wasm.enc");
    let mut bundle_file = fs::File::create(&bundle_path).unwrap();
    bundle_file.write_all(&nonce_bytes).unwrap();
    bundle_file.write_all(&ciphertext).unwrap();

    // Write CORRUPTED signature
    let mut bad_sig = signature.to_bytes();
    bad_sig[0] ^= 0xFF;
    let sig_path = keys_dir.join("pro_engine.wasm.sig");
    fs::write(&sig_path, bad_sig).unwrap();

    // Write public key PEM
    let pubkey_bytes = verifying_key.to_bytes();
    let der_bytes = [
        &[0x30, 0x2a, 0x30, 0x05, 0x06, 0x03, 0x2b, 0x65, 0x70, 0x03, 0x21, 0x00][..],
        &pubkey_bytes[..],
    ]
    .concat();

    let pem_content = format!(
        "-----BEGIN PUBLIC KEY-----\n{}\n-----END PUBLIC KEY-----\n",
        base64::encode(&der_bytes)
    );

    let pubkey_path = keys_dir.join("pubkey.pem");
    fs::write(&pubkey_path, pem_content).unwrap();

    // Load pro engine should fail
    let loader = ProEngineLoader::new(keys_dir.to_path_buf());
    let result = loader.load_pro_engine(&license_path, &bundle_path, &pubkey_path);

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), LoaderError::VerifyError));
}

#[test]
fn test_load_pro_engine_expired_license() {
    let temp_dir = tempdir().unwrap();
    let keys_dir = temp_dir.path();

    // Create expired license
    let license_key = "test-license-key-12345";
    let expires = (Utc::now() - Duration::days(1)).to_rfc3339();
    let license_json = json!({
        "license_key": license_key,
        "email": "test@example.com",
        "expires": expires,
        "signature": "placeholder",
        "machine_binding": null
    });

    let license_path = keys_dir.join("license.json");
    fs::write(&license_path, serde_json::to_string(&license_json).unwrap()).unwrap();

    let bundle_path = keys_dir.join("pro_engine.wasm.enc");
    let pubkey_path = keys_dir.join("pubkey.pem");

    // Create dummy files
    fs::write(&bundle_path, b"dummy").unwrap();
    fs::write(&pubkey_path, "dummy").unwrap();

    let loader = ProEngineLoader::new(keys_dir.to_path_buf());
    let result = loader.load_pro_engine(&license_path, &bundle_path, &pubkey_path);

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), LoaderError::Expired));
}
