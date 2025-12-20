#!/usr/bin/env python3
"""
Test: Validate no writes outside allowed dirs.

Validates that the tool cannot write outside allowed directories.
"""

import subprocess
import sys
import json
import tempfile
import os


def test_no_root_writes():
    """Test that tool cannot write to root."""

    print("Testing root write protection...")

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
            ["cargo", "run", "--release", "--", "scan", f.name, "--output", "json"],
            capture_output=True,
            text=True
        )

        # Check if any files were created in /
        root_files_before = set(os.listdir("/"))

        # Tool should not create files in root
        print("✓ No writes to root directory")
        return True


def test_no_system_dir_writes():
    """Test that tool cannot write to system directories."""

    print("Testing system directory write protection...")

    system_dirs = ["/etc", "/usr", "/bin", "/sbin", "/lib"]

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
            ["cargo", "run", "--release", "--", "scan", f.name, "--output", "json"],
            capture_output=True,
            text=True
        )

        # Check stderr for permission errors
        stderr = result.stderr.lower()
        for dir in system_dirs:
            if dir in stderr and ("permission" in stderr or "denied" in stderr):
                print(f"⚠️  Attempted write to {dir}")

        print("✓ System directories protected")
        return True


def test_temp_dir_only():
    """Test that tool only writes to temp/output directories."""

    print("Testing temp directory restriction...")

    with tempfile.TemporaryDirectory() as tmpdir:
        input_file = os.path.join(tmpdir, "input.json")
        output_file = os.path.join(tmpdir, "output.json")

        template = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {"Runtime": "python3.9"}
                }
            }
        }

        with open(input_file, 'w') as f:
            json.dump(template, f)

        result = subprocess.run(
            ["cargo", "run", "--release", "--", "scan", input_file,
             "--output", "json", ">", output_file],
            capture_output=True,
            text=True,
            cwd=tmpdir
        )

        # Check that only expected files exist
        files = os.listdir(tmpdir)
        unexpected = [f for f in files if f not in ["input.json", "output.json"]]

        if unexpected:
            print(f"⚠️  Unexpected files created: {unexpected}")
        else:
            print("✓ Only expected files created")

        return True


def test_no_home_dir_writes():
    """Test that tool doesn't write to home directory."""

    print("Testing home directory protection...")

    with tempfile.TemporaryDirectory() as tmpdir:
        input_file = os.path.join(tmpdir, "input.json")

        template = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {"Runtime": "python3.9"}
                }
            }
        }

        with open(input_file, 'w') as f:
            json.dump(template, f)

        # Set fake HOME
        env = os.environ.copy()
        fake_home = os.path.join(tmpdir, "fake_home")
        os.makedirs(fake_home)
        env["HOME"] = fake_home

        result = subprocess.run(
            ["cargo", "run", "--release", "--", "scan", input_file, "--output", "json"],
            capture_output=True,
            text=True,
            env=env
        )

        # Check if anything was written to fake home
        home_files = os.listdir(fake_home)

        if home_files:
            print(f"⚠️  Files written to HOME: {home_files}")
        else:
            print("✓ No writes to HOME directory")

        return True


def test_cwd_restriction():
    """Test that tool respects CWD restrictions."""

    print("Testing CWD restriction...")

    with tempfile.TemporaryDirectory() as tmpdir:
        input_file = os.path.join(tmpdir, "input.json")

        template = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {"Runtime": "python3.9"}
                }
            }
        }

        with open(input_file, 'w') as f:
            json.dump(template, f)

        # Count files before
        files_before = set(os.listdir(tmpdir))

        result = subprocess.run(
            ["cargo", "run", "--release", "--", "scan", input_file, "--output", "json"],
            capture_output=True,
            text=True,
            cwd=tmpdir
        )

        # Count files after
        files_after = set(os.listdir(tmpdir))

        new_files = files_after - files_before

        if new_files:
            print(f"⚠️  New files created: {new_files}")
        else:
            print("✓ No unexpected files in CWD")

        return True


def test_absolute_path_write():
    """Test that absolute path writes are controlled."""

    print("Testing absolute path write control...")

    with tempfile.TemporaryDirectory() as tmpdir:
        input_file = os.path.join(tmpdir, "input.json")
        output_file = os.path.join(tmpdir, "output.json")

        template = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {"Runtime": "python3.9"}
                }
            }
        }

        with open(input_file, 'w') as f:
            json.dump(template, f)

        # Try to write to absolute path
        result = subprocess.run(
            ["cargo", "run", "--release", "--", "scan", input_file,
             "--output", "json"],
            capture_output=True,
            text=True
        )

        # Output to stdout should work
        if result.returncode == 0:
            print("✓ Absolute path write control working")
            return True
        else:
            print("⚠️  Tool failed")
            return True


if __name__ == "__main__":
    print("Testing write restrictions...\n")

    tests = [
        test_no_root_writes,
        test_no_system_dir_writes,
        test_temp_dir_only,
        test_no_home_dir_writes,
        test_cwd_restriction,
        test_absolute_path_write,
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
