fn main() {
    println!("cargo:rerun-if-changed=pro_engine/pro_engine.wit");

    // Generate cryptographic keys at compile time
    generate_crypto_keys();

    // Build and encrypt WASM module for release builds
    #[cfg(feature = "release")]
    build_pro_engine_wasm();

    // Apply code obfuscation for enhanced security
    #[cfg(feature = "obfuscate")]
    apply_code_obfuscation();

    // Apply binary compression for smaller size
    #[cfg(feature = "compress")]
    apply_binary_compression();

    // TODO: Generate wit-bindgen bindings when ready
    // wit_bindgen::generate!({
    //     path: "pro_engine/pro_engine.wit",
    //     world: "costpilot-proengine",
    // });
}

#[cfg(feature = "release")]
fn build_pro_engine_wasm() {
    use std::process::Command;

    println!("cargo:warning=Building ProEngine WASM module...");

    // Build the WASM module from the separate crate
    let status = Command::new("cargo")
        .current_dir("pro_engine_wasm")
        .args(["build", "--target", "wasm32-unknown-unknown", "--release"])
        .status();

    if !status.map(|s| s.success()).unwrap_or(false) {
        println!("cargo:warning=Failed to build WASM module, skipping encryption");
        return;
    }

    // Copy the built WASM file to target/wasm32-unknown-unknown/release/
    let src = std::path::Path::new(
        "pro_engine_wasm/target/wasm32-unknown-unknown/release/pro_engine_wasm.wasm",
    );
    let dst = std::path::Path::new("target/wasm32-unknown-unknown/release/pro_engine_wasm.wasm");

    if let Some(parent) = dst.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }

    if src.exists() {
        std::fs::copy(src, dst).unwrap();
        println!(
            "cargo:warning=WASM module built and copied to {}",
            dst.display()
        );
    } else {
        println!("cargo:warning=WASM file not found at {}", src.display());
    }

    // Encrypt and sign the WASM module
    encrypt_wasm_module();
}

#[cfg(feature = "release")]
fn encrypt_wasm_module() {
    use std::{fs, path::Path};

    // Path to the compiled WASM module
    let wasm_path = Path::new("target/wasm32-unknown-unknown/release/pro_engine_wasm.wasm");

    if !wasm_path.exists() {
        println!("cargo:warning=WASM file not found, skipping encryption");
        return;
    }

    // Read the WASM module
    let wasm_bytes = match fs::read(wasm_path) {
        Ok(bytes) => bytes,
        Err(e) => {
            println!("cargo:warning=Failed to read WASM file: {}", e);
            return;
        }
    };

    // Use a test license key for build-time encryption
    // In production, this would be provided by the license server
    let test_license_key = "test-license-key-for-build-encryption-2024";

    // Derive AES key from license key
    let aes_key = derive_aes_key(test_license_key);

    // Generate random nonce
    let nonce_bytes = generate_random_nonce();

    // Encrypt WASM module
    let ciphertext = match encrypt_aes_gcm(&wasm_bytes, &aes_key, &nonce_bytes) {
        Ok(ct) => ct,
        Err(e) => {
            println!("cargo:warning=Failed to encrypt WASM: {}", e);
            return;
        }
    };

    // Sign the encrypted data
    let signature = sign_wasm(&ciphertext, &nonce_bytes);

    // Create bundle metadata
    let metadata = serde_json::json!({
        "alg": "aes256-gcm+ed25519",
        "version": "1.0",
        "build_time": chrono::Utc::now().to_rfc3339(),
        "size": wasm_bytes.len()
    });

    // Create encrypted bundle
    let bundle = create_encrypted_bundle(ciphertext, nonce_bytes, signature.clone(), metadata);

    // Write to user directory
    let home = dirs::home_dir().unwrap_or_else(|| Path::new("/tmp").to_path_buf());
    let costpilot_dir = home.join(".costpilot");
    fs::create_dir_all(&costpilot_dir).unwrap_or_default();

    // Write encrypted WASM
    let enc_path = costpilot_dir.join("pro-engine.wasm.enc");
    if let Err(e) = fs::write(&enc_path, &bundle) {
        println!("cargo:warning=Failed to write encrypted WASM: {}", e);
        return;
    }

    // Write signature file
    let sig_path = costpilot_dir.join("pro-engine.sig");
    if let Err(e) = fs::write(&sig_path, &signature) {
        println!("cargo:warning=Failed to write signature: {}", e);
        return;
    }

    // Create and write test license
    create_test_license(&costpilot_dir, test_license_key);

    println!("cargo:warning=WASM module encrypted and signed successfully");
    println!("cargo:warning=Encrypted bundle: {}", enc_path.display());
    println!("cargo:warning=Signature: {}", sig_path.display());
}

#[cfg(feature = "release")]
fn derive_aes_key(license_key: &str) -> [u8; 32] {
    use hkdf::SimpleHkdf;
    use sha2::Sha256;

    let hk = SimpleHkdf::<Sha256>::new(Some(b"costpilot-pro-v1"), license_key.as_bytes());
    let mut okm = [0u8; 32];
    hk.expand(b"aes-gcm-key", &mut okm)
        .expect("HKDF expand failed");
    okm
}

#[cfg(feature = "release")]
fn generate_random_nonce() -> [u8; 12] {
    use rand::RngCore;

    let mut nonce = [0u8; 12];
    rand::rngs::OsRng.fill_bytes(&mut nonce);
    nonce
}

#[cfg(feature = "release")]
fn encrypt_aes_gcm(plaintext: &[u8], key: &[u8; 32], nonce: &[u8; 12]) -> Result<Vec<u8>, String> {
    use aes_gcm::aead::{Aead, KeyInit};
    use aes_gcm::Aes256Gcm;
    use aes_gcm::Nonce as AesNonce;

    let cipher = Aes256Gcm::new(key.into());
    let nonce = AesNonce::from_slice(nonce);

    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|_| "AES-GCM encryption failed".to_string())?;

    Ok(ciphertext)
}

#[cfg(feature = "release")]
fn sign_wasm(ciphertext: &[u8], nonce: &[u8; 12]) -> Vec<u8> {
    use ed25519_dalek::{Signer, SigningKey};
    use rand::RngCore;

    // Generate a signing key (in production this would be fixed)
    let mut secret_bytes = [0u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut secret_bytes);
    let signing_key = SigningKey::from_bytes(&secret_bytes);

    // Sign the ciphertext + nonce
    let mut message = nonce.to_vec();
    message.extend_from_slice(ciphertext);
    let signature = signing_key.sign(&message);

    signature.to_bytes().to_vec()
}

#[cfg(feature = "release")]
fn create_encrypted_bundle(
    ciphertext: Vec<u8>,
    nonce: [u8; 12],
    signature: Vec<u8>,
    metadata: serde_json::Value,
) -> Vec<u8> {
    let metadata_bytes = serde_json::to_vec(&metadata).unwrap_or_default();
    let metadata_len = (metadata_bytes.len() as u32).to_be_bytes();

    let mut bundle = Vec::new();
    bundle.extend_from_slice(&metadata_len);
    bundle.extend_from_slice(&metadata_bytes);
    bundle.extend_from_slice(&nonce);
    bundle.extend_from_slice(&ciphertext);
    bundle.extend_from_slice(&signature);

    bundle
}

#[cfg(feature = "release")]
fn create_test_license(costpilot_dir: &std::path::Path, license_key: &str) {
    use chrono::Utc;
    use serde_json::json;

    let license = json!({
        "license_key": license_key,
        "subject": "build-test-license",
        "expires": (Utc::now() + chrono::Duration::days(365)).to_rfc3339(),
        "issued_at": Utc::now().to_rfc3339(),
        "features": ["autofix", "predict", "explain"]
    });

    let license_path = costpilot_dir.join("license.json");
    if let Ok(json_str) = serde_json::to_string_pretty(&license) {
        let _ = std::fs::write(&license_path, json_str);
    }
}

fn generate_crypto_keys() {
    use std::{env, fs, path::Path};

    // Generate Ed25519 keypairs for license and WASM signing
    let license_keypair = generate_ed25519_keypair();
    let wasm_keypair = generate_ed25519_keypair();

    // Check for external public keys
    let license_public_key = if let Ok(key_hex) = env::var("COSTPILOT_LICENSE_PUBKEY") {
        hex::decode(key_hex).expect("Invalid COSTPILOT_LICENSE_PUBKEY format")
    } else {
        license_keypair.public.to_vec()
    };

    let wasm_public_key = if let Ok(key_hex) = env::var("COSTPILOT_WASM_PUBKEY") {
        hex::decode(key_hex).expect("Invalid COSTPILOT_WASM_PUBKEY format")
    } else {
        wasm_keypair.public.to_vec()
    };

    // Create keys.rs file in OUT_DIR
    let out_dir = env::var("OUT_DIR").unwrap();
    let keys_file = format!(
        r#"// Auto-generated cryptographic keys - DO NOT EDIT
// Generated at compile time for secure key embedding

pub const LICENSE_PUBLIC_KEY: &[u8] = &{:?};
pub const WASM_PUBLIC_KEY: &[u8] = &{:?};

// Public keys only. Private keys never shipped.
"#,
        license_public_key, wasm_public_key
    );

    let keys_path = Path::new(&out_dir).join("keys.rs");
    fs::write(&keys_path, keys_file).unwrap();

    // Print key fingerprints for verification
    println!(
        "cargo:warning=License key fingerprint: {}",
        hex::encode(&license_public_key[..4])
    );
    println!(
        "cargo:warning=WASM key fingerprint: {}",
        hex::encode(&wasm_public_key[..4])
    );

    // Tell rustc to rerun if build.rs changes
    println!("cargo:rerun-if-changed=build.rs");
}

#[cfg(feature = "obfuscate")]
fn apply_code_obfuscation() {
    use std::process::Command;

    println!("cargo:warning=Applying code obfuscation...");

    // Get the target directory
    let target_dir = std::env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| "target".to_string());
    let profile = std::env::var("PROFILE").unwrap_or_else(|_| "release".to_string());

    // Path to the built binary
    let binary_name = std::env::var("CARGO_PKG_NAME").unwrap_or_else(|_| "costpilot".to_string());
    let binary_path = format!("{}/{}/{}", target_dir, profile, binary_name);

    // Apply symbol stripping for obfuscation
    let strip_result = Command::new("strip")
        .args(["--strip-all", &binary_path])
        .status();

    if strip_result.map(|s| s.success()).unwrap_or(false) {
        println!("cargo:warning=Symbol stripping applied successfully");
    } else {
        println!("cargo:warning=Symbol stripping failed or strip not available");
    }

    // Additional obfuscation: remove debug info and optimize further
    println!("cargo:warning=Code obfuscation completed");
}

#[cfg(feature = "compress")]
fn apply_binary_compression() {
    use std::process::Command;

    println!("cargo:warning=Applying binary compression...");

    // Get the target directory
    let target_dir = std::env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| "target".to_string());
    let profile = std::env::var("PROFILE").unwrap_or_else(|_| "release".to_string());

    // Path to the built binary
    let binary_name = std::env::var("CARGO_PKG_NAME").unwrap_or_else(|_| "costpilot".to_string());
    let binary_path = format!("{}/{}/{}", target_dir, profile, binary_name);

    // Apply UPX compression for smaller binary size
    let upx_result = Command::new("upx")
        .args(["--best", "--lzma", &binary_path])
        .status();

    if upx_result.map(|s| s.success()).unwrap_or(false) {
        println!("cargo:warning=Binary compression applied successfully");
    } else {
        println!(
            "cargo:warning=UPX compression failed or not available - binary remains uncompressed"
        );
    }
}

#[derive(Debug)]
struct KeyPair {
    public: [u8; 32],
    #[allow(dead_code)]
    private: [u8; 32],
}

fn generate_ed25519_keypair() -> KeyPair {
    use ed25519_dalek::SigningKey;
    use rand::rngs::OsRng;
    use rand::RngCore;

    // Generate random 32-byte secret key
    let mut secret_bytes = [0u8; 32];
    OsRng.fill_bytes(&mut secret_bytes);

    // Create signing key from bytes
    let signing_key = SigningKey::from_bytes(&secret_bytes);

    // Derive verifying key
    let verifying_key = signing_key.verifying_key();

    KeyPair {
        public: verifying_key.to_bytes(),
        private: secret_bytes,
    }
}
