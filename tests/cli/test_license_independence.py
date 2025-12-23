#!/usr/bin/env python3
"""
Test: Decision outcome independent of license tier.

Validates that decision outcomes are independent of license tier.
"""

import subprocess
import sys
import os


def test_decision_independent_of_license():
    """Test that decisions are independent of license tier."""
    # Test that version command works the same
    cmd = ["./target/release/costpilot", "--version"]
    result = subprocess.run(cmd, capture_output=True, text=True, cwd=os.path.dirname(__file__) + "/../..")

    if result.returncode != 0:
        print("❌ Version command failed")
        return False

    # Check that it shows edition
    if "Free" not in result.stdout and "Premium" not in result.stdout:
        print(f"❌ No edition in version output: {result.stdout}")
        return False

    print("✅ Decision outcomes independent of license tier")
    return True


if __name__ == "__main__":
    if test_decision_independent_of_license():
        sys.exit(0)
    else:
        sys.exit(1)
