use costpilot::pro_engine::loader::parse_bundle;

#[test]
fn test_parse_bundle_standalone() {
    let encrypted_bundle_bytes = vec![
        0, 0, 0, 15, // Metadata length (15 bytes)
        b'{', b'"', b'k', b'e', b'y', b'"', b':', b'"', b'v', b'a', b'l', b'u', b'e', b'"', b'}', // Metadata (valid JSON object)
        1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, // Nonce (12 bytes)
        20, 21, 22, 23, 24, 25, // Ciphertext (6 bytes)
        30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 91, 92, 93, 94, 95, // Signature (64 bytes)
    ];

    // omitted test debug output

    let result = parse_bundle(&encrypted_bundle_bytes);
    assert!(result.is_ok(), "parse_bundle should succeed with valid input");
}
