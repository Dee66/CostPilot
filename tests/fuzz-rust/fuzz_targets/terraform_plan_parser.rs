#![no_main]

use libfuzzer_sys::fuzz_target;
use costpilot::engines::detection::terraform::parse_terraform_plan;

fuzz_target!(|data: &[u8]| {
    // Convert fuzzer input to string
    if let Ok(input) = std::str::from_utf8(data) {
        // Fuzz the terraform plan parser
        let _ = parse_terraform_plan(input);
    }
});
