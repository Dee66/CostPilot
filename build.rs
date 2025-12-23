fn main() {
    println!("cargo:rerun-if-changed=pro_engine/pro_engine.wit");

    // Generate cryptographic keys at compile time
    generate_crypto_keys();

    // TODO: Generate wit-bindgen bindings when ready
    // wit_bindgen_rust::generate!({
    //     path: "pro_engine/pro_engine.wit",
    //     world: "costpilot-proengine",
    // });
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
