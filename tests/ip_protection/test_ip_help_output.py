#!/usr/bin/env python3
"""Test IP Protection: No premium feature names in Free help."""

import subprocess


def test_free_help_no_premium_commands():
    """Test Free help doesn't list premium commands."""
    result = subprocess.run(
        ["costpilot", "--help"],
        capture_output=True,
        text=True,
        timeout=10
    )
    
    assert result.returncode == 0, "Help should succeed"
    output = result.stdout.lower()
    
    # Should not mention premium commands
    forbidden_commands = [
        "autofix",
        "patch",
        "slo",
        "drift",
        "anomaly",
        "economic-attack"
    ]
    
    for cmd in forbidden_commands:
        assert cmd not in output, f"Free help should not mention: {cmd}"


def test_free_help_no_premium_flags():
    """Test Free help doesn't list premium flags."""
    result = subprocess.run(
        ["costpilot", "scan", "--help"],
        capture_output=True,
        text=True,
        timeout=10
    )
    
    if result.returncode == 0:
        output = result.stdout.lower()
        
        # Should not mention premium flags
        forbidden_flags = [
            "--bundle",
            "--license",
            "--premium",
            "--pro",
            "--enterprise",
            "--advanced",
            "--mode pro",
            "--mode premium"
        ]
        
        for flag in forbidden_flags:
            assert flag not in output, f"Free help should not mention: {flag}"


def test_free_subcommand_list():
    """Test Free subcommand list excludes premium."""
    result = subprocess.run(
        ["costpilot", "--help"],
        capture_output=True,
        text=True,
        timeout=10
    )
    
    assert result.returncode == 0, "Help should succeed"
    output = result.stdout.lower()
    
    # Should only show Free commands
    assert "analyze" in output, "Should show analyze"
    assert "predict" in output or "check" in output, "Should show basic commands"
    
    # Should not show premium
    assert "autofix" not in output, "Should not show autofix"
    assert "patch" not in output, "Should not show patch"


def test_free_help_no_premium_examples():
    """Test Free help doesn't include premium examples."""
    result = subprocess.run(
        ["costpilot", "--help"],
        capture_output=True,
        text=True,
        timeout=10
    )
    
    if result.returncode == 0:
        output = result.stdout.lower()
        
        # Should not contain premium examples
        forbidden_examples = [
            "autofix --apply",
            "patch --output",
            "slo check",
            "drift detect",
            "--bundle premium.bundle",
            "--license license.key"
        ]
        
        for example in forbidden_examples:
            assert example not in output, f"Free help should not include example: {example}"


def test_free_help_no_premium_documentation_links():
    """Test Free help doesn't link to premium docs."""
    result = subprocess.run(
        ["costpilot", "--help"],
        capture_output=True,
        text=True,
        timeout=10
    )
    
    if result.returncode == 0:
        output = result.stdout.lower()
        
        # Should not link to premium documentation
        forbidden_links = [
            "docs/premium",
            "docs/pro",
            "docs/enterprise",
            "/premium/",
            "/pro/",
            "premium-guide"
        ]
        
        for link in forbidden_links:
            assert link not in output, f"Free help should not link to: {link}"


def test_free_command_list_consistent():
    """Test Free command list is consistent across invocations."""
    results = []
    for _ in range(3):
        result = subprocess.run(
            ["costpilot", "--help"],
            capture_output=True,
            text=True,
            timeout=10
        )
        assert result.returncode == 0
        results.append(result.stdout)
    
    # All outputs should be identical
    assert results[0] == results[1] == results[2], "Help output should be deterministic"


def test_free_help_mentions_edition():
    """Test Free help identifies as Community/Free edition."""
    result = subprocess.run(
        ["costpilot", "--help"],
        capture_output=True,
        text=True,
        timeout=10
    )
    
    if result.returncode == 0:
        output = result.stdout.lower()
        
        # Should identify as Free/Community
        # (may not always be present, but if edition mentioned, should be Free)
        if "edition" in output:
            assert "community" in output or "free" in output or "oss" in output, \
                "Should identify as Community/Free/OSS edition"


if __name__ == "__main__":
    test_free_help_no_premium_commands()
    test_free_help_no_premium_flags()
    test_free_subcommand_list()
    test_free_help_no_premium_examples()
    test_free_help_no_premium_documentation_links()
    test_free_command_list_consistent()
    test_free_help_mentions_edition()
    print("All IP Protection: help output tests passed")
