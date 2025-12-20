#!/usr/bin/env python3
"""
Test Template Generator for CostPilot

Quickly generate test files from templates.

Usage:
    python3 scripts/generate_test.py unit prediction test_heuristics_loader
    python3 scripts/generate_test.py integration test_full_pipeline
    python3 scripts/generate_test.py e2e test_cli_scan
"""

import sys
import os
from pathlib import Path

UNIT_TEMPLATE = '''/// Unit tests for {module_name}
///
/// {description}

#[cfg(test)]
mod {test_name}_tests {{
    use super::*;

    #[test]
    fn test_{test_name}_basic() {{
        // TODO: Implement test
        assert!(true);
    }}

    #[test]
    fn test_{test_name}_error_handling() {{
        // TODO: Test error cases
    }}

    #[test]
    fn test_{test_name}_edge_cases() {{
        // TODO: Test edge cases
    }}
}}
'''

INTEGRATION_TEMPLATE = '''/// Integration tests for {module_name}
///
/// {description}

#[cfg(test)]
mod {test_name}_tests {{
    use super::*;

    #[test]
    fn test_{test_name}_success_path() {{
        // TODO: Test happy path
        assert!(true);
    }}

    #[test]
    fn test_{test_name}_with_real_data() {{
        // TODO: Test with realistic data
    }}

    #[test]
    fn test_{test_name}_error_recovery() {{
        // TODO: Test error handling
    }}
}}
'''

E2E_TEMPLATE = '''/// End-to-end tests for {module_name}
///
/// {description}

#[cfg(test)]
mod {test_name}_tests {{
    use assert_cmd::Command;
    use predicates::prelude::*;
    use tempfile::TempDir;

    #[test]
    fn test_{test_name}_cli_success() {{
        // TODO: Test CLI command
        // Command::cargo_bin("costpilot")
        //     .unwrap()
        //     .arg("command")
        //     .assert()
        //     .success();
    }}

    #[test]
    fn test_{test_name}_cli_error_handling() {{
        // TODO: Test error cases
    }}

    #[test]
    fn test_{test_name}_cli_output_format() {{
        // TODO: Test output format
    }}
}}
'''

PROPERTY_TEMPLATE = '''/// Property-based tests for {module_name}
///
/// {description}

#[cfg(test)]
mod {test_name}_properties {{
    use proptest::prelude::*;

    proptest! {{
        #[test]
        fn property_{test_name}(value in 0.0..1000.0) {{
            // TODO: Define property
            prop_assert!(value >= 0.0);
        }}
    }}
}}
'''

SNAPSHOT_TEMPLATE = '''/// Snapshot tests for {module_name}
///
/// {description}

#[cfg(test)]
mod {test_name}_snapshots {{
    use insta::assert_json_snapshot;

    #[test]
    fn test_{test_name}_snapshot() {{
        // TODO: Generate output
        let result = generate_output();
        assert_json_snapshot!(result);
    }}
}}
'''

def generate_test(test_type: str, module_name: str, test_name: str):
    """Generate a test file from template"""

    templates = {
        'unit': (UNIT_TEMPLATE, 'tests/unit'),
        'integration': (INTEGRATION_TEMPLATE, 'tests/integration'),
        'e2e': (E2E_TEMPLATE, 'tests/e2e'),
        'property': (PROPERTY_TEMPLATE, 'tests/property'),
        'snapshot': (SNAPSHOT_TEMPLATE, 'tests/snapshot'),
    }

    if test_type not in templates:
        print(f"❌ Unknown test type: {test_type}")
        print(f"   Available: {', '.join(templates.keys())}")
        sys.exit(1)

    template, base_dir = templates[test_type]

    # Generate description
    description = f"Tests for {module_name} module"

    # Fill template
    content = template.format(
        module_name=module_name,
        test_name=test_name,
        description=description
    )

    # Determine output path
    output_dir = Path(base_dir)
    output_dir.mkdir(parents=True, exist_ok=True)

    output_file = output_dir / f"{test_name}.rs"

    if output_file.exists():
        print(f"⚠️  File already exists: {output_file}")
        response = input("   Overwrite? (y/N): ")
        if response.lower() != 'y':
            print("   Cancelled")
            sys.exit(0)

    # Write file
    output_file.write_text(content)
    print(f"✅ Created: {output_file}")
    print(f"   Type: {test_type}")
    print(f"   Module: {module_name}")
    print(f"   Test: {test_name}")

def show_usage():
    """Show usage information"""
    print("""
CostPilot Test Generator

Usage:
    python3 scripts/generate_test.py <type> <module> <test_name>

Types:
    unit         - Unit test (tests/unit/)
    integration  - Integration test (tests/integration/)
    e2e          - End-to-end test (tests/e2e/)
    property     - Property-based test (tests/property/)
    snapshot     - Snapshot test (tests/snapshot/)

Examples:
    python3 scripts/generate_test.py unit prediction test_heuristics_loader
    python3 scripts/generate_test.py integration test_full_pipeline
    python3 scripts/generate_test.py e2e test_cli_scan
    python3 scripts/generate_test.py property test_prediction_invariants
    python3 scripts/generate_test.py snapshot test_explain_output
""")

def main():
    if len(sys.argv) != 4:
        show_usage()
        sys.exit(1)

    test_type = sys.argv[1]
    module_name = sys.argv[2]
    test_name = sys.argv[3]

    generate_test(test_type, module_name, test_name)

if __name__ == '__main__':
    main()
