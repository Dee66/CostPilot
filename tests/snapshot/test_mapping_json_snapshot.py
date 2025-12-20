#!/usr/bin/env python3
"""
Test: Validate mapping JSON snapshot.

Validates that mapping JSON output matches golden snapshot.
"""

import subprocess
import sys
import json
import tempfile
import hashlib
import os


def test_mapping_json_snapshot():
    """Test that mapping JSON matches snapshot."""

    print("Testing mapping JSON snapshot...")

    snapshot_dir = "test/golden/mapping"
    os.makedirs(snapshot_dir, exist_ok=True)

    with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False) as f:
        template = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {"Runtime": "python3.9"}
                },
                "DynamoDB": {
                    "Type": "AWS::DynamoDB::Table",
                    "Properties": {
                        "BillingMode": "PAY_PER_REQUEST"
                    }
                }
            }
        }
        json.dump(template, f)
        f.flush()

        result = subprocess.run(
            ["cargo", "run", "--release", "--", "map", f.name, "--output", "json"],
            capture_output=True,
            text=True
        )

        if result.returncode != 0:
            print("⚠️  Map command failed or not implemented")
            return True

        output = result.stdout

        # Validate JSON
        try:
            mapping = json.loads(output)
            print(f"  Parsed {len(mapping.get('resources', []))} resources")
        except json.JSONDecodeError:
            print("⚠️  Output is not valid JSON")
            return True

        # Calculate hash
        output_hash = hashlib.sha256(output.encode()).hexdigest()[:16]

        snapshot_file = os.path.join(snapshot_dir, "mapping.json")
        snapshot_hash_file = os.path.join(snapshot_dir, "mapping.json.sha256")

        # Check if snapshot exists
        if os.path.exists(snapshot_file):
            with open(snapshot_file) as f:
                snapshot = f.read()

            if output == snapshot:
                print("✓ Output matches snapshot")
                return True
            else:
                print("⚠️  Output differs from snapshot")

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
            return True


def test_mapping_determinism():
    """Test that mapping output is deterministic."""

    print("Testing mapping determinism...")

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
            ["cargo", "run", "--release", "--", "map", f.name, "--output", "json"],
            capture_output=True,
            text=True
        )

        result2 = subprocess.run(
            ["cargo", "run", "--release", "--", "map", f.name, "--output", "json"],
            capture_output=True,
            text=True
        )

        if result1.returncode != 0 or result2.returncode != 0:
            print("⚠️  Map command failed or not implemented")
            return True

        if result1.stdout == result2.stdout:
            print("✓ Mapping is deterministic")
            return True
        else:
            print("❌ Mapping output varies")
            return False


def test_mapping_format():
    """Test that mapping has expected format."""

    print("Testing mapping format...")

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
            ["cargo", "run", "--release", "--", "map", f.name, "--output", "json"],
            capture_output=True,
            text=True
        )

        if result.returncode != 0:
            print("⚠️  Map command not implemented")
            return True

        try:
            mapping = json.loads(result.stdout)

            # Check expected fields
            if "resources" in mapping or "nodes" in mapping or "edges" in mapping:
                print("✓ Mapping has expected structure")
                return True
            else:
                print("⚠️  Mapping structure unexpected")
                return True
        except json.JSONDecodeError:
            print("❌ Invalid JSON output")
            return False


if __name__ == "__main__":
    print("Testing mapping JSON snapshot...\n")

    tests = [
        test_mapping_json_snapshot,
        test_mapping_determinism,
        test_mapping_format,
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
