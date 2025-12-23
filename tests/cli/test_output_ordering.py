#!/usr/bin/env python3
"""
Test: Output ordering stable across runs.

Validates that CLI output ordering is stable across multiple runs.
"""

import subprocess
import sys
import os


def run_command(args):
    """Run command and return stdout."""
    cmd = ["./target/release/costpilot"] + args
    result = subprocess.run(cmd, capture_output=True, text=True, cwd=os.path.dirname(__file__) + "/../..")
    if result.returncode != 0:
        return None
    return result.stdout


def test_output_ordering_stable():
    """Test that output ordering is stable across runs."""
    # Use explain command as it's deterministic
    args = ["explain", "aws_instance", "--instance-type", "t2.micro"]

    outputs = []
    for i in range(3):
        output = run_command(args)
        if output is None:
            print(f"❌ Command failed on run {i+1}")
            return False
        outputs.append(output)

    # Check all outputs are identical
    if not all(o == outputs[0] for o in outputs):
        print("❌ Output ordering not stable across runs")
        for i, o in enumerate(outputs):
            print(f"Run {i+1}: {repr(o)}")
        return False

    print("✅ Output ordering stable across runs")
    return True


if __name__ == "__main__":
    if test_output_ordering_stable():
        sys.exit(0)
    else:
        sys.exit(1)
