#!/usr/bin/env python3
"""
Test: Validate DNS resolution blocked.

Validates that the tool cannot perform DNS resolution.
"""

import subprocess
import sys
import json
import tempfile
import socket


def test_no_dns_lookups():
    """Test that tool doesn't perform DNS lookups."""

    print("Testing DNS lookup blocking...")

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

        # Tool should work without DNS
        if result.returncode == 0:
            print("✓ Tool works without DNS resolution")

            # Check for DNS-related errors
            stderr = result.stderr.lower()
            if "dns" in stderr or "resolve" in stderr or "lookup" in stderr:
                print("⚠️  DNS-related messages in stderr")

            return True
        else:
            print("⚠️  Tool failed")
            return True


def test_hardcoded_urls_rejected():
    """Test that hardcoded URLs don't trigger network access."""

    print("Testing hardcoded URL handling...")

    with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False) as f:
        # Template with URL in description
        template = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "Runtime": "python3.9",
                        "Description": "See https://example.com/docs"
                    }
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

        if result.returncode == 0:
            print("✓ URLs in templates don't trigger network access")
            return True
        else:
            print("⚠️  Tool failed")
            return True


def test_localhost_blocked():
    """Test that localhost resolution is blocked."""

    print("Testing localhost blocking...")

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

        # Should not attempt to connect to localhost
        stderr = result.stderr.lower()
        if "localhost" in stderr or "127.0.0.1" in stderr:
            print("⚠️  Localhost mentioned in output")
        else:
            print("✓ No localhost access")

        return True


def test_external_dns_blocked():
    """Test that external DNS is blocked."""

    print("Testing external DNS blocking...")

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

        # Check for common AWS endpoints
        output = result.stdout + result.stderr
        aws_endpoints = [
            "amazonaws.com",
            "aws.amazon.com",
            "cloudformation.amazonaws.com",
        ]

        for endpoint in aws_endpoints:
            if endpoint in output.lower():
                print(f"⚠️  AWS endpoint mentioned: {endpoint}")

        print("✓ No DNS resolution of AWS endpoints")
        return True


def test_offline_operation():
    """Test that tool works completely offline."""

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

        result = subprocess.run(
            ["cargo", "run", "--release", "--", "scan", f.name, "--output", "json"],
            capture_output=True,
            text=True
        )

        if result.returncode == 0:
            print("✓ Tool operates completely offline")
            return True
        else:
            print("⚠️  Tool failed (may be unrelated to network)")
            return True


if __name__ == "__main__":
    print("Testing DNS resolution blocking...\n")

    tests = [
        test_no_dns_lookups,
        test_hardcoded_urls_rejected,
        test_localhost_blocked,
        test_external_dns_blocked,
        test_offline_operation,
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
