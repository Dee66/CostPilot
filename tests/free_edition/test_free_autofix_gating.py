#!/usr/bin/env python3
"""Test Free Edition: autofix command not present."""

import subprocess
import tempfile
from pathlib import Path
import json


def test_autofix_command_not_present():
    """Test autofix command not available in Free Edition."""
    result = subprocess.run(
        ["costpilot", "autofix", "--help"],
        capture_output=True,
        text=True,
        timeout=10
    )
    
    # Should fail - autofix not available in Free
    assert result.returncode != 0, "autofix command should not exist in Free Edition"
    
    # Check error message
    error = result.stderr.lower()
    assert "not found" in error or "unknown" in error or "free" in error or "premium" in error, \
        "Should indicate command not available"


def test_autofix_with_template_rejected():
    """Test autofix with template is rejected."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        
        template_content = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 10240
                    }
                }
            }
        }
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        result = subprocess.run(
            ["costpilot", "autofix", "--plan", str(template_path)],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        # Should fail
        assert result.returncode != 0, "autofix should be rejected"


def test_autofix_not_in_help():
    """Test autofix not listed in help."""
    result = subprocess.run(
        ["costpilot", "--help"],
        capture_output=True,
        text=True,
        timeout=10
    )
    
    # autofix should not appear in help
    assert "autofix" not in result.stdout.lower(), "autofix should not be in help"


def test_autofix_subcommand_rejected():
    """Test autofix subcommand variations rejected."""
    commands = [
        ["costpilot", "autofix"],
        ["costpilot", "auto-fix"],
        ["costpilot", "fix", "--auto"],
    ]
    
    for cmd in commands:
        result = subprocess.run(
            cmd,
            capture_output=True,
            text=True,
            timeout=10
        )
        
        # Should fail
        assert result.returncode != 0, f"Command {cmd} should be rejected"


def test_autofix_with_apply_rejected():
    """Test autofix with --apply flag rejected."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        
        template_content = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 10240
                    }
                }
            }
        }
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        result = subprocess.run(
            ["costpilot", "autofix", "--plan", str(template_path), "--apply"],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        # Should fail
        assert result.returncode != 0, "autofix --apply should be rejected"


if __name__ == "__main__":
    test_autofix_command_not_present()
    test_autofix_with_template_rejected()
    test_autofix_not_in_help()
    test_autofix_subcommand_rejected()
    test_autofix_with_apply_rejected()
    print("All Free Edition autofix gating tests passed")
