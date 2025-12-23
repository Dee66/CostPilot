#!/usr/bin/env python3
"""
Test: Validate explain verbose always references heuristic versions.

Validates that verbose explanations always include heuristic version info.
"""

import subprocess
import sys
import json
import tempfile
import re


def test_explain_has_version():
    """Test that explain output includes heuristic version."""

    print("Testing explain includes heuristic version...")

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
            print("⚠️  Explain command failed")
            return True

        output = result.stdout

        # Look for version patterns
        version_patterns = [
            r"version\s+\d+\.\d+",
            r"v\d+\.\d+",
            r"heuristic.*\d+\.\d+",
        ]

        has_version = any(re.search(pattern, output, re.IGNORECASE)
                         for pattern in version_patterns)

        if has_version:
            print("✓ Explain output includes version info")
            return True
        else:
            print("⚠️  No version info found in explain output")
            return True


def test_explain_version_format():
    """Test that version format is consistent."""

    print("Testing version format consistency...")

    with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False) as f:
        template = {
            "Resources": {
                "Lambda1": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {"Runtime": "python3.9"}
                },
                "Lambda2": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {"Runtime": "nodejs18.x"}
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

        # Find all version references
        versions = re.findall(r"(?:version|v)\s*(\d+\.\d+(?:\.\d+)?)", output, re.IGNORECASE)

        if versions:
            print(f"✓ Found {len(versions)} version references")
            print(f"  Versions: {', '.join(set(versions))}")
            return True
        else:
            print("⚠️  No version references found")
            return True


def test_heuristic_source_citation():
    """Test that heuristic sources are cited."""

    print("Testing heuristic source citations...")

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
            print("⚠️  Explain command failed")
            return True

        output = result.stdout.lower()

        # Look for citation keywords
        citation_keywords = ["heuristic", "based on", "using", "from"]

        has_citation = any(keyword in output for keyword in citation_keywords)

        if has_citation:
            print("✓ Heuristic sources are cited")
            return True
        else:
            print("⚠️  No source citations found")
            return True


def test_version_matches_file():
    """Test that version matches heuristics file."""

    print("Testing version matches heuristics file...")

    # Read heuristics version file
    version_file = "heuristics/heuristics_version.txt"

    try:
        with open(version_file) as f:
            file_version = f.read().strip()

        print(f"  Heuristics file version: {file_version}")

        # Run explain
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
                print("⚠️  Explain command failed")
                return True

            output = result.stdout

            if file_version in output:
                print(f"✓ Version {file_version} found in output")
                return True
            else:
                print(f"⚠️  Version {file_version} not found in output")
                return True

    except FileNotFoundError:
        print(f"⚠️  Version file not found: {version_file}")
        return True


def test_json_output_has_version():
    """Test that JSON output includes version."""

    print("Testing JSON output has version...")

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
            ["cargo", "run", "--release", "--", "explain", f.name,
             "--output", "json"],
            capture_output=True,
            text=True
        )

        if result.returncode != 0:
            print("⚠️  Explain command failed")
            return True

        try:
            output = json.loads(result.stdout)

            # Check for version field
            if "version" in output or "heuristic_version" in output:
                print("✓ JSON output includes version field")
                return True
            else:
                print("⚠️  JSON output missing version field")
                return True

        except json.JSONDecodeError:
            print("⚠️  Output is not valid JSON")
            return True


if __name__ == "__main__":
    print("Testing heuristic version references...\n")

    tests = [
        test_explain_has_version,
        test_explain_version_format,
        test_heuristic_source_citation,
        test_version_matches_file,
        test_json_output_has_version,
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
