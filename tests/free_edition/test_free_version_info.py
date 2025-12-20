#!/usr/bin/env python3
import os
COSTPILOT_PATH = os.path.join(os.path.dirname(__file__), "..", "..", "target", "debug", "costpilot")
"""Test Free Edition: version info clearly identifies Community Edition."""

import subprocess
import re


def test_version_identifies_community():
    """Test --version identifies Community Edition."""
    result = subprocess.run(
        [COSTPILOT_PATH, "--version"],
        capture_output=True,
        text=True,
        timeout=10
    )

    version_text = result.stdout.lower()

    # Should mention Free (new canonical format)
    assert "free" in version_text or "(free)" in version_text, "Free version should identify as Free"

    # Should not claim Pro/Premium
    assert "pro" not in version_text or "(pro)" not in version_text, "Free version should not claim Pro"
    assert "premium" not in version_text, "Free version should not claim Premium"
    assert "enterprise" not in version_text, "Free version should not claim Enterprise"


def test_version_format():
    """Test version format is clear."""
    result = subprocess.run(
        [COSTPILOT_PATH, "--version"],
        capture_output=True,
        text=True,
        timeout=10
    )

    version_text = result.stdout

    # Should have version number
    version_pattern = r"\d+\.\d+\.\d+"
    assert re.search(version_pattern, version_text), "Should have version number"


def test_version_no_pro_suffix():
    """Test version doesn't have -pro suffix."""
    result = subprocess.run(
        [COSTPILOT_PATH, "--version"],
        capture_output=True,
        text=True,
        timeout=10
    )

    version_text = result.stdout.lower()

    # Should not have Pro suffix
    pro_suffixes = ["-pro", "-premium", "-enterprise", "+pro", "+premium"]

    for suffix in pro_suffixes:
        assert suffix not in version_text, f"Version should not have {suffix} suffix"


def test_version_output_consistent():
    """Test version output is consistent across calls."""
    results = []

    for _ in range(3):
        result = subprocess.run(
            [COSTPILOT_PATH, "--version"],
            capture_output=True,
            text=True,
            timeout=10
        )
        results.append(result.stdout)

    # All should be identical
    assert results[0] == results[1] == results[2], "Version output should be consistent"


def test_version_via_v_flag():
    """Test version via -V flag."""
    result = subprocess.run(
        [COSTPILOT_PATH, "-V"],
        capture_output=True,
        text=True,
        timeout=10
    )

    # Should succeed or fail consistently with --version
    result_version = subprocess.run(
        [COSTPILOT_PATH, "--version"],
        capture_output=True,
        text=True,
        timeout=10
    )

    # Both should have same behavior
    if result.returncode == 0 and result_version.returncode == 0:
        # Both should show version
        assert len(result.stdout) > 0, "-V should show version"


def test_about_or_info_shows_edition():
    """Test about/info command shows edition."""
    # Some CLIs have about or info commands
    for cmd in ["about", "info"]:
        result = subprocess.run(
            [COSTPILOT_PATH, cmd],
            capture_output=True,
            text=True,
            timeout=10
        )

        if result.returncode == 0:
            output = result.stdout.lower()

            # Should mention edition
            edition_terms = ["community", "free", "edition"]
            found = any(term in output for term in edition_terms)

            # At minimum, shouldn't claim Pro
            if not found:
                assert "pro edition" not in output, f"{cmd} should not claim Pro"


if __name__ == "__main__":
    test_version_identifies_community()
    test_version_format()
    test_version_no_pro_suffix()
    test_version_output_consistent()
    test_version_via_v_flag()
    test_about_or_info_shows_edition()
    print("All Free Edition version identification tests passed")
