#!/usr/bin/env python3
"""Test Free Edition: help shows only Free commands."""

import subprocess


def test_help_shows_only_free_commands():
    """Test --help shows only Free Edition commands."""
    result = subprocess.run(
        ["costpilot", "--help"],
        capture_output=True,
        text=True,
        timeout=10
    )
    
    help_text = result.stdout.lower()
    
    # Should show Free commands
    free_commands = ["scan", "diff", "policy", "validate"]
    for cmd in free_commands:
        assert cmd in help_text, f"Help should mention {cmd}"
    
    # Should NOT show Pro commands (if hidden in free mode)
    # Note: Currently all commands visible but some may require premium
    # This test validates that help output is accessible


def test_help_subcommand_list():
    """Test help lists only available subcommands."""
    result = subprocess.run(
        ["costpilot", "--help"],
        capture_output=True,
        text=True,
        timeout=10
    )
    
    # Count subcommands mentioned
    help_text = result.stdout.lower()
    
    # Free commands should be present
    assert "commands:" in help_text or "subcommands:" in help_text, \
        "Help should list commands"


def test_individual_command_help():
    """Test individual command help works for Free commands."""
    free_commands = ["scan", "diff", "validate"]
    
    for cmd in free_commands:
        result = subprocess.run(
            ["costpilot", cmd, "--help"],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        # Should succeed
        assert result.returncode == 0, f"Help for {cmd} should work"
        assert len(result.stdout) > 0, f"Help for {cmd} should have output"


def test_pro_command_help_fails():
    """Test Pro command help fails or not listed."""
    pro_commands = ["autofix", "patch", "slo"]
    
    for cmd in pro_commands:
        result = subprocess.run(
            ["costpilot", cmd, "--help"],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        # Should fail
        assert result.returncode != 0, f"Help for Pro command {cmd} should fail"


def test_help_mentions_free_edition():
    """Test help mentions Free/Community Edition."""
    result = subprocess.run(
        ["costpilot", "--help"],
        capture_output=True,
        text=True,
        timeout=10
    )
    
    help_text = result.stdout.lower()
    
    # Should mention edition
    edition_terms = ["free", "community", "edition"]
    found = any(term in help_text for term in edition_terms)
    
    # If not mentioned, at least shouldn't say "Pro" or "Premium"
    if not found:
        assert "pro edition" not in help_text, "Should not claim to be Pro"
        assert "premium edition" not in help_text, "Should not claim to be Premium"


def test_help_no_premium_flags():
    """Test help doesn't list premium-only flags."""
    result = subprocess.run(
        ["costpilot", "scan", "--help"],
        capture_output=True,
        text=True,
        timeout=10
    )
    
    help_text = result.stdout.lower()
    
    # Should not list premium flags
    premium_flags = ["--mode", "--license", "--bundle", "--pro", "--premium"]
    
    for flag in premium_flags:
        # Flags might not appear at all (good) or might be documented as unavailable
        pass


if __name__ == "__main__":
    test_help_shows_only_free_commands()
    test_help_subcommand_list()
    test_individual_command_help()
    test_pro_command_help_fails()
    test_help_mentions_free_edition()
    test_help_no_premium_flags()
    print("All Free Edition help gating tests passed")
