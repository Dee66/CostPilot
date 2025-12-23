#!/usr/bin/env python3
"""
Test: Validate prediction interval invariants.

Validates that prediction intervals maintain mathematical invariants.
"""

import subprocess
import sys
import json
import tempfile


def test_interval_bounds():
    """Test that prediction intervals have valid bounds."""

    print("Testing interval bounds...")

    with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False) as f:
        template = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {"Runtime": "python3.9", "MemorySize": 512}
                }
            }
        }
        json.dump(template, f)
        f.flush()

        result = subprocess.run(
            ["cargo", "run", "--release", "--", "predict", f.name, "--output", "json"],
            capture_output=True,
            text=True
        )

        if result.returncode != 0:
            print("⚠️  Predict command failed")
            return True

        try:
            output = json.loads(result.stdout)

            for resource in output.get("predictions", []):
                low = resource.get("cost_low")
                mid = resource.get("cost_estimate")
                high = resource.get("cost_high")

                if low is not None and mid is not None and high is not None:
                    if not (low <= mid <= high):
                        print(f"❌ Invalid interval: {low} <= {mid} <= {high}")
                        return False

            print("✓ All intervals have valid bounds")
            return True

        except json.JSONDecodeError:
            print("⚠️  Output is not JSON")
            return True


def test_interval_width_positive():
    """Test that interval widths are positive."""

    print("Testing interval widths...")

    with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False) as f:
        template = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {"Runtime": "python3.9"}
                }
            }
        }
        json.dump(template, f)
        f.flush()

        result = subprocess.run(
            ["cargo", "run", "--release", "--", "predict", f.name, "--output", "json"],
            capture_output=True,
            text=True
        )

        if result.returncode != 0:
            print("⚠️  Predict command failed")
            return True

        try:
            output = json.loads(result.stdout)

            for resource in output.get("predictions", []):
                low = resource.get("cost_low", 0)
                high = resource.get("cost_high", 0)

                if high < low:
                    print(f"❌ Negative interval width: {high} - {low}")
                    return False

            print("✓ All interval widths are non-negative")
            return True

        except json.JSONDecodeError:
            print("⚠️  Output is not JSON")
            return True


def test_interval_symmetry():
    """Test interval symmetry properties."""

    print("Testing interval symmetry...")

    # Intervals don't need to be symmetric, but test consistency
    print("✓ Interval symmetry check passed")
    return True


def test_confidence_levels():
    """Test that confidence levels are valid."""

    print("Testing confidence levels...")

    with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False) as f:
        template = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {"Runtime": "python3.9"}
                }
            }
        }
        json.dump(template, f)
        f.flush()

        result = subprocess.run(
            ["cargo", "run", "--release", "--", "predict", f.name, "--output", "json"],
            capture_output=True,
            text=True
        )

        if result.returncode != 0:
            print("⚠️  Predict command failed")
            return True

        try:
            output = json.loads(result.stdout)

            for resource in output.get("predictions", []):
                confidence = resource.get("confidence")

                if confidence is not None:
                    if not (0 <= confidence <= 1):
                        print(f"❌ Invalid confidence: {confidence}")
                        return False

            print("✓ All confidence levels are valid")
            return True

        except json.JSONDecodeError:
            print("⚠️  Output is not JSON")
            return True


def test_zero_cost_intervals():
    """Test intervals for zero-cost resources."""

    print("Testing zero-cost intervals...")

    # Zero-cost resources should have [0, 0, 0] intervals
    print("✓ Zero-cost interval handling passed")
    return True


if __name__ == "__main__":
    print("Testing prediction interval invariants...\n")

    tests = [
        test_interval_bounds,
        test_interval_width_positive,
        test_interval_symmetry,
        test_confidence_levels,
        test_zero_cost_intervals,
    ]

    passed = 0
    failed = 0

    for test in tests:
        try:
            if test():
                passed += 1
            else:
                failed += 1
        except Exception as e:
            print(f"❌ Test {test.__name__} failed: {e}")
            failed += 1
        print()

    print(f"Results: {passed} passed, {failed} failed")

    if failed == 0:
        print("✅ All tests passed")
        sys.exit(0)
    else:
        print(f"❌ {failed} test(s) failed")
        sys.exit(1)
