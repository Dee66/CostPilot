#![no_main]

use libfuzzer_sys::fuzz_target;
use costpilot::engines::policy::parser::dsl::DslParser;

fuzz_target!(|data: &[u8]| {
    // Convert fuzzer input to string
    if let Ok(input) = std::str::from_utf8(data) {
        // Fuzz YAML policy parsing
        let _ = DslParser::parse_yaml(input);

        // Fuzz JSON policy parsing
        let _ = DslParser::parse_json(input);
    }
});
