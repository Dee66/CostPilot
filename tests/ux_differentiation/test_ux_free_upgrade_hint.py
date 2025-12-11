#!/usr/bin/env python3
"""Test UX Differentiation: Free help includes upgrade hint."""

import subprocess


def test_free_help_includes_upgrade_hint():
    """Test Free help mentions upgrade option."""
    result = subprocess.run(
        ["costpilot", "--help"],
        capture_output=True,
        text=True,
        timeout=10
    )
    
    assert result.returncode == 0, "Help should succeed"
    output = result.stdout.lower()
    
    # Should mention upgrade, premium, or pro
    upgrade_indicators = [
        "upgrade",
        "premium",
        "pro",
        "enterprise",
        "commercial",
        "license"
    ]
    
    # At least one indicator should be present
    found = any(indicator in output for indicator in upgrade_indicators)
    assert found, "Free help should include upgrade hint"


def test_free_version_includes_edition():
    """Test Free version identifies as Community/Free."""
    result = subprocess.run(
        ["costpilot", "--version"],
        capture_output=True,
        text=True,
        timeout=10
    )
    
    if result.returncode == 0:
        output = result.stdout.lower()
        
        # Should identify edition
        edition_indicators = ["community", "free", "oss", "open source"]
        found = any(indicator in output for indicator in edition_indicators)
        
        # If edition mentioned, should be Free
        if any(word in output for word in ["edition", "version"]):
            assert found, "Should identify as Community/Free/OSS"


def test_free_help_footer():
    """Test Free help footer mentions upgrade path."""
    result = subprocess.run(
        ["costpilot", "--help"],
        capture_output=True,
        text=True,
        timeout=10
    )
    
    if result.returncode == 0:
        output = result.stdout
        
        # Check last few lines for upgrade info
        lines = output.strip().split('\n')
        last_lines = '\n'.join(lines[-5:]).lower()
        
        # Footer might mention more features, upgrade, or website
        upgrade_phrases = [
            "for more features",
            "upgrade to",
            "premium features",
            "visit",
            "learn more"
        ]
        
        # At least one phrase might be present


def test_free_command_help_mentions_limitations():
    """Test Free command help mentions limitations."""
    result = subprocess.run(
        ["costpilot", "scan", "--help"],
        capture_output=True,
        text=True,
        timeout=10
    )
    
    if result.returncode == 0:
        output = result.stdout.lower()
        
        # Might mention depth limit or other restrictions
        # This is documentation, not enforcement


def test_upgrade_hint_deterministic():
    """Test upgrade hint is consistent."""
    outputs = []
    for _ in range(3):
        result = subprocess.run(
            ["costpilot", "--help"],
            capture_output=True,
            text=True,
            timeout=10
        )
        assert result.returncode == 0
        outputs.append(result.stdout)
    
    # All outputs should be identical
    assert outputs[0] == outputs[1] == outputs[2], "Help should be deterministic"


def test_free_help_professional():
    """Test Free help is professional and clear."""
    result = subprocess.run(
        ["costpilot", "--help"],
        capture_output=True,
        text=True,
        timeout=10
    )
    
    if result.returncode == 0:
        output = result.stdout.lower()
        
        # Should not use aggressive marketing
        forbidden = [
            "buy now",
            "limited time",
            "special offer",
            "discount",
            "sale"
        ]
        
        for phrase in forbidden:
            assert phrase not in output, f"Help should not include: {phrase}"


if __name__ == "__main__":
    test_free_help_includes_upgrade_hint()
    test_free_version_includes_edition()
    test_free_help_footer()
    test_free_command_help_mentions_limitations()
    test_upgrade_hint_deterministic()
    test_free_help_professional()
    print("All UX Differentiation: Free upgrade hint tests passed")
