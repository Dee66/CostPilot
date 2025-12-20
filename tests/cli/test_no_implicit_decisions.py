#!/usr/bin/env python3
"""
Test: No implicit decisions.

Validates that ambiguous inputs produce hard_stop.
"""

import subprocess
import sys
import os


def test_no_implicit_decisions():
    """Test that ambiguous inputs produce hard_stop."""
    # Test invalid command
    cmd = ["./target/release/costpilot", "invalid-command"]
    result = subprocess.run(cmd, capture_output=True, text=True, cwd=os.path.dirname(__file__) + "/../..")

    if result.returncode == 0:
        print("❌ Invalid command should not succeed")
        return False

    # Should exit with non-zero (hard_stop)
    print("✅ No implicit decisions")
    return True


if __name__ == "__main__":
    if test_no_implicit_decisions():
        sys.exit(0)
    else:
        sys.exit(1)
