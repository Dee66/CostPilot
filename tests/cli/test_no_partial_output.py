#!/usr/bin/env python3
"""
Test: No partial output on failure paths.

Validates that failing commands produce no partial output on stdout.
"""

import subprocess
import sys
import os


def test_no_partial_output_on_failure():
    """Test that failing commands don't produce partial output."""
    # Test scan with nonexistent file
    cmd = ["./target/release/costpilot", "scan", "nonexistent.json"]
    result = subprocess.run(cmd, capture_output=True, text=True, cwd=os.path.dirname(__file__) + "/../..")

    # Should exit with non-zero
    if result.returncode == 0:
        print("❌ Expected failure, but command succeeded")
        return False

    # Stdout should be empty (only banner and error on stderr)
    if result.stdout.strip():
        print(f"❌ Unexpected stdout on failure: {repr(result.stdout)}")
        return False

    print("✅ No partial output on failure paths")
    return True


if __name__ == "__main__":
    if test_no_partial_output_on_failure():
        sys.exit(0)
    else:
        sys.exit(1)
