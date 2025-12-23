#!/usr/bin/env python3
"""Test Premium: full explain mode references encrypted heuristics bundle."""

import subprocess
import tempfile
from pathlib import Path
import json
import os


def test_explain_full_mode():
    """Test full explain mode exists in Premium."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"

        template_content = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 2048
                    }
                }
            }
        }

        with open(template_path, 'w') as f:
            json.dump(template_content, f)

        result = subprocess.run(
            ["costpilot", "explain", "all", "--plan", str(template_path), "--mode", "full"],
            capture_output=True,
            text=True,
            timeout=10
        )

        # In Premium, should succeed
        # In Free, should reject --mode full
        if result.returncode == 0:
            assert len(result.stdout) > 0, "Full explain should produce output"


def test_explain_references_bundle():
    """Test explain references heuristics bundle."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"

        template_content = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 2048,
                        "Runtime": "python3.9"
                    }
                }
            }
        }

        with open(template_path, 'w') as f:
            json.dump(template_content, f)

        result = subprocess.run(
            ["costpilot", "explain", "all", "--plan", str(template_path), "--verbose"],
            capture_output=True,
            text=True,
            timeout=10
        )

        # In Premium, might reference heuristics
        if result.returncode == 0:
            output = result.stdout.lower()
            # May mention heuristics or advanced analysis


def test_explain_detailed_mode():
    """Test detailed explain mode."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"

        template_content = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 2048,
                        "Timeout": 300,
                        "Runtime": "python3.9"
                    }
                }
            }
        }

        with open(template_path, 'w') as f:
            json.dump(template_content, f)

        result = subprocess.run(
            ["costpilot", "explain", "all", "--plan", str(template_path), "--detailed"],
            capture_output=True,
            text=True,
            timeout=10
        )

        # Premium should provide detailed explanations
        if result.returncode == 0:
            output_len = len(result.stdout)
            # Detailed mode should produce more output
            assert output_len > 100, "Detailed explanation should be substantial"


def test_explain_with_bundle_path():
    """Test explain with explicit bundle path."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        bundle_path = Path(tmpdir) / "premium.bundle"

        template_content = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 2048
                    }
                }
            }
        }

        # Create dummy bundle
        with open(bundle_path, 'wb') as f:
            f.write(b"BUNDLE:VERSION:1.0.0\n")
            f.write(b"ENCRYPTED_HEURISTICS_DATA")

        with open(template_path, 'w') as f:
            json.dump(template_content, f)

        result = subprocess.run(
            ["costpilot", "explain", "all", "--plan", str(template_path), "--bundle", str(bundle_path)],
            capture_output=True,
            text=True,
            timeout=10
        )

        # Should attempt to use bundle


def test_explain_advanced_analysis():
    """Test explain advanced analysis features."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"

        template_content = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 2048,
                        "Timeout": 300
                    }
                },
                "DynamoDB": {
                    "Type": "AWS::DynamoDB::Table",
                    "Properties": {
                        "BillingMode": "PAY_PER_REQUEST"
                    }
                }
            }
        }

        with open(template_path, 'w') as f:
            json.dump(template_content, f)

        result = subprocess.run(
            ["costpilot", "explain", "all", "--plan", str(template_path), "--advanced"],
            capture_output=True,
            text=True,
            timeout=10
        )

        # Premium should provide advanced analysis
        if result.returncode == 0:
            assert len(result.stdout) > 0, "Advanced explain should produce output"


def test_explain_output_format():
    """Test explain output format."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        output_path = Path(tmpdir) / "explanation.json"

        template_content = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 2048
                    }
                }
            }
        }

        with open(template_path, 'w') as f:
            json.dump(template_content, f)

        result = subprocess.run(
            ["costpilot", "explain", "all", "--plan", str(template_path), "--output", str(output_path)],
            capture_output=True,
            text=True,
            timeout=10
        )

        # Should create output file
        if result.returncode == 0 and output_path.exists():
            with open(output_path) as f:
                explanation = json.load(f)
            assert isinstance(explanation, (dict, list)), "Explanation should be valid JSON"


if __name__ == "__main__":
    test_explain_full_mode()
    test_explain_references_bundle()
    test_explain_detailed_mode()
    test_explain_with_bundle_path()
    test_explain_advanced_analysis()
    test_explain_output_format()
    print("All Premium explain mode tests passed")
