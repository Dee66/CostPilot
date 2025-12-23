#!/usr/bin/env python3
"""
Test: Exit codes map deterministically.

Validates that CLI exit codes match expected values for different scenarios.
"""

import subprocess
import sys
import os


def run_command(args, expect_success=True):
    """Run command and return exit code."""
    cmd = ["./target/release/costpilot"] + args
    result = subprocess.run(cmd, capture_output=True, text=True, cwd=os.path.dirname(__file__) + "/../..")
    return result.returncode


def test_exit_code_0():
    """Test exit code 0 for successful runs."""
    # Version command should succeed
    code = run_command(["--version"])
    return code == 0


def test_exit_code_4_invalid_input():
    """Test exit code 4 for invalid input."""
    # Invalid flag
    code = run_command(["--invalid-flag"])
    return code == 4


def test_exit_codes():
    """Test all exit codes."""
    print("Testing exit codes...")

    tests = [
        ("Exit code 0 (success)", test_exit_code_0),
        ("Exit code 4 (invalid input)", test_exit_code_4_invalid_input),
        # TODO: Add tests for 2, 3, 5 when scenarios are available
    ]

    all_passed = True
    for name, test_func in tests:
        print(f"  Testing {name}...")
        if test_func():
            print(f"  ✅ {name} passed")
        else:
            print(f"  ❌ {name} failed")
            all_passed = False

    if all_passed:
        print("✅ All tested exit codes are correct")
    else:
        print("❌ Some exit codes are incorrect")

    return all_passed


if __name__ == "__main__":
    if test_exit_codes():
        sys.exit(0)
    else:
        sys.exit(1)
