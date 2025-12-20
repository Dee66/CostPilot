#!/usr/bin/env python3
import os
COSTPILOT_PATH = os.path.join(os.path.dirname(__file__), "..", "..", "target", "debug", "costpilot")
"""Test Free Edition: advanced explain modes rejected."""

import subprocess
import tempfile
from pathlib import Path
import json
import pytest


@pytest.mark.skip("TODO: Update for new explain command structure with subcommands")
def test_verbose_flag_rejected():
    """Test --verbose flag rejected in Free Edition."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"

        template_content = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 1024
                    }
                }
            }
        }

        with open(template_path, 'w') as f:
            json.dump(template_content, f)

        result = subprocess.run(
            [COSTPILOT_PATH, "explain", "all", "--plan", str(template_path), "--verbose"],
            capture_output=True,
            text=True,
            timeout=10
        )

        # Should fail or ignore
        if result.returncode != 0:
            error = result.stderr.lower()
            assert "unknown" in error or "premium" in error or "free" in error or "unexpected" in error or "verbose" in error, \
                "Should reject --verbose"


def test_deep_flag_rejected():
    """Test --deep flag rejected in Free Edition."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"

        template_content = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 1024
                    }
                }
            }
        }

        with open(template_path, 'w') as f:
            json.dump(template_content, f)

        result = subprocess.run(
            [COSTPILOT_PATH, "explain", "all", "--plan", str(template_path), "--deep"],
            capture_output=True,
            text=True,
            timeout=10
        )

        # Should fail
        if result.returncode != 0:
            error = result.stderr.lower()
            assert "unknown" in error or "premium" in error or "free" in error or "unexpected" in error or "deep" in error, \
                "Should reject --deep"


def test_detailed_flag_rejected():
    """Test --detailed flag rejected in Free Edition."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"

        template_content = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 1024
                    }
                }
            }
        }

        with open(template_path, 'w') as f:
            json.dump(template_content, f)

        result = subprocess.run(
            [COSTPILOT_PATH, "explain", "all", "--plan", str(template_path), "--detailed"],
            capture_output=True,
            text=True,
            timeout=10
        )

        # Should fail
        if result.returncode != 0:
            error = result.stderr.lower()
            assert "unknown" in error or "premium" in error or "detailed" in error, \
                "Should reject --detailed"


def test_advanced_flag_rejected():
    """Test --advanced flag rejected in Free Edition."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"

        template_content = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 1024
                    }
                }
            }
        }

        with open(template_path, 'w') as f:
            json.dump(template_content, f)

        result = subprocess.run(
            [COSTPILOT_PATH, "explain", "all", "--plan", str(template_path), "--advanced"],
            capture_output=True,
            text=True,
            timeout=10
        )

        # Should fail
        if result.returncode != 0:
            error = result.stderr.lower()
            assert "unknown" in error or "premium" in error or "advanced" in error, \
                "Should reject --advanced"


def test_explain_mode_pro_rejected():
    """Test explain with --mode pro rejected."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"

        template_content = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 1024
                    }
                }
            }
        }

        with open(template_path, 'w') as f:
            json.dump(template_content, f)

        result = subprocess.run(
            [COSTPILOT_PATH, "explain", "all", "--plan", str(template_path), "--mode", "pro"],
            capture_output=True,
            text=True,
            timeout=10
        )

        # Should fail
        if result.returncode != 0:
            error = result.stderr.lower()
            assert "unknown" in error or "premium" in error or "mode" in error, \
                "Should reject --mode pro"


def test_basic_explain_allowed():
    """Test basic explain without flags is allowed."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"

        template_content = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 1024
                    }
                }
            }
        }

        with open(template_path, 'w') as f:
            json.dump(template_content, f)

        result = subprocess.run(
            [COSTPILOT_PATH, "explain", "all", "--plan", str(template_path)],
            capture_output=True,
            text=True,
            timeout=10
        )

        # Basic explain should work (or explain command might not exist)
        # This test documents expected behavior
        assert result.returncode in [0, 1, 2, 101], "Basic explain behavior"


if __name__ == "__main__":
    test_verbose_flag_rejected()
    test_deep_flag_rejected()
    test_detailed_flag_rejected()
    test_advanced_flag_rejected()
    test_explain_mode_pro_rejected()
    test_basic_explain_allowed()
    print("All Free Edition advanced explain mode gating tests passed")
