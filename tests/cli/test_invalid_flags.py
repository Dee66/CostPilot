#!/usr/bin/env python3
"""
Test: Invalid flags → hard stop with message.

Validates that invalid flags cause hard stop with error_class=invalid_input.
"""

import subprocess
import sys
import os


def test_invalid_flags_error_message():
    """Test that invalid flags produce the correct error message."""
    cmd = ["./target/release/costpilot", "--invalid-flag"]
    result = subprocess.run(cmd, capture_output=True, text=True, cwd=os.path.dirname(__file__) + "/../..")

    # Should exit with 4
    if result.returncode != 4:
        print(f"❌ Expected exit code 4, got {result.returncode}")
        return False

    # Should contain error_class=invalid_input
    if "error_class=invalid_input" not in result.stderr:
        print(f"❌ Expected 'error_class=invalid_input' in stderr, got: {result.stderr}")
        return False

    print("✅ Invalid flags produce correct error message")
    return True


if __name__ == "__main__":
    if test_invalid_flags_error_message():
        sys.exit(0)
    else:
        sys.exit(1)
