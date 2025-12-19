#![no_main]

use libfuzzer_sys::fuzz_target;
use costpilot::artifact::parse_artifact;

fuzz_target!(|data: &[u8]| {
    // Convert fuzzer input to string
    if let Ok(input) = std::str::from_utf8(data) {
        // Fuzz artifact parsing
        let _ = parse_artifact(input, "fuzz");
    }
});
