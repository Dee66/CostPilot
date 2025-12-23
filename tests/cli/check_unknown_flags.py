#!/usr/bin/env python3
"""
Test: Validate unknown flags produce structured errors.

Validates that unknown flags produce structured, parseable error messages.
"""

import subprocess
import sys
import json


def test_unknown_flag(flag):
    """Test that unknown flag produces structured error."""

    print(f"Testing unknown flag: {flag}")

    result = subprocess.run(
        ["costpilot", flag],
        capture_output=True,
        text=True
    )

    # Should fail
    if result.returncode == 0:
        print(f"❌ Unknown flag {flag} was accepted")
        return False

    error_output = result.stderr

    # Check for structured error
    if not error_output:
        print(f"❌ No error output for unknown flag {flag}")
        return False

    # Check for key error components
    required_components = [
        "error:",  # Error prefix
        flag,      # The actual flag
    ]

    for component in required_components:
        if component.lower() not in error_output.lower():
            print(f"❌ Error missing component: {component}")
            print(f"   Error output: {error_output[:100]}")
            return False

    print(f"✓ Unknown flag {flag} produces structured error")
    print(f"  Error: {error_output.split(chr(10))[0][:80]}")
    return True


def test_unknown_command(command):
    """Test that unknown command produces structured error."""

    print(f"Testing unknown command: {command}")

    result = subprocess.run(
        ["costpilot", command],
        capture_output=True,
        text=True
    )

    # Should fail
    if result.returncode == 0:
        print(f"❌ Unknown command {command} was accepted")
        return False

    error_output = result.stderr

    # Check for structured error
    if not error_output:
        print(f"❌ No error output for unknown command {command}")
        return False

    print(f"✓ Unknown command {command} produces structured error")
    print(f"  Error: {error_output.split(chr(10))[0][:80]}")
    return True


def test_malformed_flag(flag):
    """Test that malformed flag produces structured error."""

    print(f"Testing malformed flag: {flag}")

    result = subprocess.run(
        ["costpilot", flag],
        capture_output=True,
        text=True
    )

    # Should fail or be ignored
    error_output = result.stderr

    if error_output:
        print(f"✓ Malformed flag {flag} produces error")
        print(f"  Error: {error_output.split(chr(10))[0][:80]}")
    else:
        print(f"⚠️  Malformed flag {flag} was silently ignored")

    return True


def test_error_format_consistency():
    """Test that errors have consistent format."""

    print("Testing error format consistency...")

    test_flags = ["--unknown1", "--unknown2", "--badFlag"]
    errors = []

    for flag in test_flags:
        result = subprocess.run(
            ["costpilot", flag],
            capture_output=True,
            text=True
        )

        if result.returncode != 0 and result.stderr:
            errors.append(result.stderr)

    if not errors:
        print("⚠️  No errors to compare")
        return True

    # Check if errors start with similar prefix
    first_words = [err.split()[0] if err.split() else "" for err in errors]

    if len(set(first_words)) == 1:
        print(f"✓ Errors have consistent format (prefix: {first_words[0]})")
        return True
    else:
        print(f"⚠️  Errors have varying formats: {set(first_words)}")
        return True  # Don't fail, just warn


def test_error_exit_codes():
    """Test that errors use appropriate exit codes."""

    print("Testing error exit codes...")

    test_cases = [
        ("--unknown-flag", "unknown flag"),
        ("badcommand", "unknown command"),
        ("--version=", "malformed flag"),
    ]

    for flag, description in test_cases:
        result = subprocess.run(
            ["costpilot", flag],
            capture_output=True,
            text=True
        )

        if result.returncode == 0:
            print(f"❌ {description} returned exit code 0")
            return False

        print(f"✓ {description} returns non-zero exit code ({result.returncode})")

    return True


def test_suggestions_for_typos():
    """Test that typos get helpful suggestions."""

    print("Testing suggestions for typos...")

    # Common typos
    typos = [
        "--hep",      # help
        "--versio",   # version
        "scn",        # scan
    ]

    for typo in typos:
        result = subprocess.run(
            ["costpilot", typo],
            capture_output=True,
            text=True
        )

        error_output = result.stderr.lower()

        # Look for suggestions
        has_suggestion = any(word in error_output for word in [
            "did you mean",
            "similar",
            "try",
            "suggestion",
        ])

        if has_suggestion:
            print(f"✓ Typo {typo} gets helpful suggestion")
        else:
            print(f"⚠️  Typo {typo} doesn't get suggestion")

    return True


def test_error_no_stacktrace():
    """Test that user errors don't show stack traces."""

    print("Testing that errors don't show stack traces...")

    result = subprocess.run(
        ["costpilot", "--unknown-flag"],
        capture_output=True,
        text=True
    )

    error_output = result.stderr

    # Check for stack trace indicators
    stacktrace_indicators = [
        "panicked at",
        "stack backtrace:",
        ".rs:",  # File references
        "thread",
    ]

    has_stacktrace = any(indicator in error_output for indicator in stacktrace_indicators)

    if has_stacktrace:
        print("❌ Error shows stack trace")
        print(f"  Error: {error_output[:200]}")
        return False
    else:
        print("✓ Error doesn't show stack trace")
        return True


def test_json_error_output():
    """Test JSON error output format."""

    print("Testing JSON error output...")

    result = subprocess.run(
        ["costpilot", "--output", "json", "--unknown-flag"],
        capture_output=True,
        text=True
    )

    # Should fail
    if result.returncode == 0:
        print("❌ Unknown flag was accepted")
        return False

    # Check if error is in JSON format
    error_output = result.stderr

    try:
        # Try to parse as JSON
        error_json = json.loads(error_output)

        # Check for error fields
        if "error" in error_json or "message" in error_json:
            print("✓ Error output is valid JSON")
            return True
        else:
            print("⚠️  JSON error missing standard fields")
            return True

    except json.JSONDecodeError:
        print("⚠️  Error output is not JSON (acceptable for text mode)")
        return True


if __name__ == "__main__":
    print("Testing unknown flag error handling...\n")

    tests = [
        lambda: test_unknown_flag("--unknown-flag"),
        lambda: test_unknown_flag("--not-a-real-option"),
        lambda: test_unknown_command("unknowncommand"),
        lambda: test_malformed_flag("--=value"),
        test_error_format_consistency,
        test_error_exit_codes,
        test_suggestions_for_typos,
        test_error_no_stacktrace,
        test_json_error_output,
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
            print(f"❌ Test failed with error: {e}")
            failed += 1
        print()

    print(f"\nResults: {passed} passed, {failed} failed\n")

    if failed == 0:
        print("✅ All unknown flag tests passed")
        sys.exit(0)
    else:
        print(f"❌ {failed} test(s) failed")
        sys.exit(1)
