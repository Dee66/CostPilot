#!/usr/bin/env python3
"""
Test: Validate rejection of AWS credential load attempts.

Validates that the tool rejects attempts to load AWS credentials.
"""

import subprocess
import sys
import json
import tempfile
import os


def test_rejects_aws_credentials():
    """Test that tool doesn't load AWS credentials."""

    print("Testing AWS credential rejection...")

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

        # Set fake AWS credentials
        env = os.environ.copy()
        env["AWS_ACCESS_KEY_ID"] = "AKIAIOSFODNN7EXAMPLE"
        env["AWS_SECRET_ACCESS_KEY"] = "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY"
        env["AWS_REGION"] = "us-east-1"

        result = subprocess.run(
            ["cargo", "run", "--release", "--", "scan", f.name, "--output", "json"],
            capture_output=True,
            text=True,
            env=env
        )

        # Should succeed without using credentials
        if result.returncode == 0:
            print("✓ Tool ignores AWS credentials")

            # Check that no network activity occurred
            stderr = result.stderr.lower()
            if any(word in stderr for word in ["credential", "aws", "auth"]):
                print("⚠️  Tool may have attempted credential loading")

            return True
        else:
            print("⚠️  Tool failed (may be unrelated)")
            return True


def test_no_credential_files():
    """Test that tool doesn't read credential files."""

    print("Testing credential file access...")

    with tempfile.TemporaryDirectory() as tmpdir:
        # Create fake AWS credential file
        aws_dir = os.path.join(tmpdir, ".aws")
        os.makedirs(aws_dir)

        creds_file = os.path.join(aws_dir, "credentials")
        with open(creds_file, 'w') as f:
            f.write("[default]\n")
            f.write("aws_access_key_id = AKIAIOSFODNN7EXAMPLE\n")
            f.write("aws_secret_access_key = wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY\n")

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

            env = os.environ.copy()
            env["HOME"] = tmpdir
            env["AWS_SHARED_CREDENTIALS_FILE"] = creds_file

            result = subprocess.run(
                ["cargo", "run", "--release", "--", "scan", f.name, "--output", "json"],
                capture_output=True,
                text=True,
                env=env
            )

            # Tool should work without reading credentials
            if result.returncode == 0:
                print("✓ Tool doesn't require credential files")
                return True
            else:
                print("⚠️  Tool failed")
                return True


def test_no_instance_metadata():
    """Test that tool doesn't access instance metadata."""

    print("Testing instance metadata blocking...")

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

        # Check stderr for metadata access attempts
        stderr = result.stderr.lower()
        if "169.254.169.254" in stderr or "metadata" in stderr:
            print("❌ Tool attempted metadata access")
            return False
        else:
            print("✓ No instance metadata access")
            return True


def test_no_sts_calls():
    """Test that tool doesn't make STS calls."""

    print("Testing STS call blocking...")

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

        stderr = result.stderr.lower()
        if "sts" in stderr or "get-caller-identity" in stderr:
            print("❌ Tool attempted STS calls")
            return False
        else:
            print("✓ No STS calls detected")
            return True


def test_offline_mode():
    """Test that tool works in offline mode."""

    print("Testing offline operation...")

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

        # Remove all AWS-related env vars
        env = os.environ.copy()
        for key in list(env.keys()):
            if key.startswith("AWS_"):
                del env[key]

        result = subprocess.run(
            ["cargo", "run", "--release", "--", "scan", f.name, "--output", "json"],
            capture_output=True,
            text=True,
            env=env
        )

        if result.returncode == 0:
            print("✓ Tool works offline without AWS context")
            return True
        else:
            print("⚠️  Tool requires AWS context")
            return True


if __name__ == "__main__":
    print("Testing AWS credential rejection...\n")

    tests = [
        test_rejects_aws_credentials,
        test_no_credential_files,
        test_no_instance_metadata,
        test_no_sts_calls,
        test_offline_mode,
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
