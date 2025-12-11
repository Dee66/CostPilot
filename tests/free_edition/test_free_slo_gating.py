#!/usr/bin/env python3
"""Test Free Edition: slo command not present."""

import subprocess
import tempfile
from pathlib import Path
import json


def test_slo_command_not_present():
    """Test slo command not available in Free Edition."""
    result = subprocess.run(
        ["costpilot", "slo", "--help"],
        capture_output=True,
        text=True,
        timeout=10
    )
    
    # Should fail - slo not available in Free
    assert result.returncode != 0, "slo command should not exist in Free Edition"
    
    # Check error message
    error = result.stderr.lower()
    assert "not found" in error or "unknown" in error or "free" in error or "premium" in error, \
        "Should indicate command not available"


def test_slo_with_config_rejected():
    """Test slo with config is rejected."""
    with tempfile.TemporaryDirectory() as tmpdir:
        slo_path = Path(tmpdir) / "slo.json"
        
        slo_content = {
            "version": "1.0.0",
            "objectives": [
                {
                    "id": "cost-stability",
                    "target": 99.0,
                    "window": "30d"
                }
            ]
        }
        
        with open(slo_path, 'w') as f:
            json.dump(slo_content, f)
        
        result = subprocess.run(
            ["costpilot", "slo", "--config", str(slo_path)],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        # Should fail
        assert result.returncode != 0, "slo should be rejected"


def test_slo_not_in_help():
    """Test slo not listed in help."""
    result = subprocess.run(
        ["costpilot", "--help"],
        capture_output=True,
        text=True,
        timeout=10
    )
    
    # slo should not appear in help as a command
    help_text = result.stdout.lower()
    # Allow mentions in descriptions but not as subcommand
    if "slo" in help_text:
        assert "costpilot slo" not in help_text, "slo should not be a subcommand in help"


def test_slo_subcommand_rejected():
    """Test slo subcommand variations rejected."""
    commands = [
        ["costpilot", "slo"],
        ["costpilot", "slo", "check"],
        ["costpilot", "slo", "validate"],
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


def test_slo_with_baseline_rejected():
    """Test slo with baseline rejected."""
    with tempfile.TemporaryDirectory() as tmpdir:
        baseline_path = Path(tmpdir) / "baseline.json"
        
        baseline_content = {
            "resources": [
                {
                    "id": "Lambda",
                    "type": "AWS::Lambda::Function",
                    "cost": 10.0
                }
            ]
        }
        
        with open(baseline_path, 'w') as f:
            json.dump(baseline_content, f)
        
        result = subprocess.run(
            ["costpilot", "slo", "--baseline", str(baseline_path)],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        # Should fail
        assert result.returncode != 0, "slo --baseline should be rejected"


def test_slo_burn_rate_rejected():
    """Test slo burn-rate subcommand rejected."""
    result = subprocess.run(
        ["costpilot", "slo", "burn-rate"],
        capture_output=True,
        text=True,
        timeout=10
    )
    
    # Should fail
    assert result.returncode != 0, "slo burn-rate should be rejected"


if __name__ == "__main__":
    test_slo_command_not_present()
    test_slo_with_config_rejected()
    test_slo_not_in_help()
    test_slo_subcommand_rejected()
    test_slo_with_baseline_rejected()
    test_slo_burn_rate_rejected()
    print("All Free Edition slo gating tests passed")
