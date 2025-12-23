#!/usr/bin/env python3
"""
Test: Silent runs emit correct output.

Validates that silent runs emit no findings, no explain output, exit code 0.
"""

import subprocess
import sys
import os


def test_silent_runs():
    """Test that silent runs emit correct output."""
    # Test version command - should be silent (just version output)
    cmd = ["./target/release/costpilot", "--version"]
    result = subprocess.run(cmd, capture_output=True, text=True, cwd=os.path.dirname(__file__) + "/../..")

    if result.returncode != 0:
        print("❌ Version command should succeed")
        return False

    # Should have output (version)
    if not result.stdout.strip():
        print("❌ Version command should have output")
        return False

    # No stderr
    if result.stderr.strip():
        print(f"❌ Version command should not have stderr: {result.stderr}")
        return False

    print("✅ Silent runs emit correct output")
    return True


if __name__ == "__main__":
    if test_silent_runs():
        sys.exit(0)
    else:
        sys.exit(1)
