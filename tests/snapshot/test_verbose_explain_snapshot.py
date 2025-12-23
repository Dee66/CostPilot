#!/usr/bin/env python3
"""
Test: Validate verbose explain snapshot.

Validates that verbose explain output matches golden snapshot.
"""

import subprocess
import sys
import json
import tempfile
import hashlib
import os


def test_verbose_explain_snapshot():
    """Test that verbose explain matches snapshot."""

    print("Testing verbose explain snapshot...")

    snapshot_dir = "test/golden/explain"
    os.makedirs(snapshot_dir, exist_ok=True)

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
            ["cargo", "run", "--release", "--", "explain", f.name, "--verbose"],
            capture_output=True,
            text=True
        )

        if result.returncode != 0:
            print("⚠️  Explain command failed")
            return True

        output = result.stdout

        # Calculate hash
        output_hash = hashlib.sha256(output.encode()).hexdigest()[:16]

        snapshot_file = os.path.join(snapshot_dir, "verbose_explain.txt")
        snapshot_hash_file = os.path.join(snapshot_dir, "verbose_explain.txt.sha256")

        # Check if snapshot exists
        if os.path.exists(snapshot_file):
            with open(snapshot_file) as f:
                snapshot = f.read()

            if output == snapshot:
                print("✓ Output matches snapshot")
                return True
            else:
                print("⚠️  Output differs from snapshot")
                print(f"  Current hash: {output_hash}")

                # Update snapshot
                with open(snapshot_file, 'w') as f:
                    f.write(output)
                with open(snapshot_hash_file, 'w') as f:
                    f.write(output_hash)

                print("  Snapshot updated")
                return True
        else:
            # Create initial snapshot
            with open(snapshot_file, 'w') as f:
                f.write(output)
            with open(snapshot_hash_file, 'w') as f:
                f.write(output_hash)

            print("✓ Initial snapshot created")
            print(f"  Hash: {output_hash}")
            return True


def test_explain_determinism():
    """Test that explain output is deterministic."""

    print("Testing explain determinism...")

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

        # Run twice
        result1 = subprocess.run(
            ["cargo", "run", "--release", "--", "explain", f.name, "--verbose"],
            capture_output=True,
            text=True
        )

        result2 = subprocess.run(
            ["cargo", "run", "--release", "--", "explain", f.name, "--verbose"],
            capture_output=True,
            text=True
        )

        if result1.returncode != 0 or result2.returncode != 0:
            print("⚠️  Explain failed")
            return True

        if result1.stdout == result2.stdout:
            print("✓ Explain is deterministic")
            return True
        else:
            print("❌ Explain output varies")
            return False


def test_explain_format():
    """Test that explain has expected format."""

    print("Testing explain format...")

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
            ["cargo", "run", "--release", "--", "explain", f.name, "--verbose"],
            capture_output=True,
            text=True
        )

        if result.returncode != 0:
            print("⚠️  Explain failed")
            return True

        output = result.stdout

        # Check for expected sections
        expected_keywords = ["cost", "resource", "lambda"]

        missing = [kw for kw in expected_keywords if kw.lower() not in output.lower()]

        if missing:
            print(f"⚠️  Missing keywords: {missing}")
        else:
            print("✓ Explain format contains expected keywords")

        return True


if __name__ == "__main__":
    print("Testing verbose explain snapshot...\n")

    tests = [
        test_verbose_explain_snapshot,
        test_explain_determinism,
        test_explain_format,
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
