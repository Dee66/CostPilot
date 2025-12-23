#!/usr/bin/env python3
"""
Test: Every decision has an explanation artifact.

Validates that decisions include explanations.
"""

import subprocess
import sys
import os


def test_decisions_have_explanations():
    """Test that decisions include explanations."""
    # Test explain command produces explanation
    cmd = ["./target/release/costpilot", "explain", "aws_instance", "--instance-type", "t2.micro"]
    result = subprocess.run(cmd, capture_output=True, text=True, cwd=os.path.dirname(__file__) + "/../..")

    if result.returncode != 0:
        print("❌ Explain command failed")
        return False

    # Check that output contains explanation
    if "Explanation" not in result.stdout and "Predicted" not in result.stdout:
        print(f"❌ No explanation in output: {result.stdout}")
        return False

    print("✅ Decisions have explanation artifacts")
    return True


if __name__ == "__main__":
    if test_decisions_have_explanations():
        sys.exit(0)
    else:
        sys.exit(1)
