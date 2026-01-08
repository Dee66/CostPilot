//! Utility to print test public key bytes for hardcoding in crypto.rs

#[cfg(test)]
mod tests {
    use ed25519_dalek::SigningKey;

    #[test]
    fn print_test_public_key_bytes() {
        let seed = [42u8; 32];
        let signing_key = SigningKey::from_bytes(&seed);
        let verifying_key = signing_key.verifying_key();
        let bytes = verifying_key.to_bytes();

        print!("\nTEST_LICENSE_PUBLIC_KEY: &[");
        for (i, byte) in bytes.iter().enumerate() {
            if i > 0 {
                print!(", ");
            }
            if i % 8 == 0 && i > 0 {
                print!("\n    ");
            }
            print!("0x{:02x}", byte);
        }
        println!("\n];\n");
    }
}
