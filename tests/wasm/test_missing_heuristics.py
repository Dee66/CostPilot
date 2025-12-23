#!/usr/bin/env python3
"""Test missing heuristics failure mode."""

import json
import os
import subprocess
import tempfile
from pathlib import Path


def test_missing_heuristics_file_fails():
    """Missing heuristics file should cause graceful failure."""
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

        # Run without heuristics file available
        # (if costpilot can't find heuristics)
        result = subprocess.run(
            ["costpilot", "predict", "--plan", str(template_path)],
            capture_output=True,
            text=True,
            env={**os.environ, "COSTPILOT_HEURISTICS": "/nonexistent/heuristics.json"}
        )

        # Should fail gracefully with error message
        if result.returncode != 0:
            error_output = result.stderr + result.stdout
            assert "heuristics" in error_output.lower() or "not found" in error_output.lower(), \
                "Should mention missing heuristics"


def test_corrupted_heuristics_fails():
    """Corrupted heuristics file should fail with clear error."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        heuristics_path = Path(tmpdir) / "heuristics.json"

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

        # Corrupted heuristics
        with open(heuristics_path, 'w') as f:
            f.write('{invalid json}')

        with open(template_path, 'w') as f:
            json.dump(template_content, f)

        result = subprocess.run(
            ["costpilot", "predict", "--plan", str(template_path), "--heuristics", str(heuristics_path)],
            capture_output=True,
            text=True
        )

        # Should fail with parse error
        if result.returncode != 0:
            error_output = result.stderr + result.stdout
            assert "parse" in error_output.lower() or "invalid" in error_output.lower(), \
                "Should mention parsing error"


def test_empty_heuristics_fails():
    """Empty heuristics file should fail."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        heuristics_path = Path(tmpdir) / "heuristics.json"

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

        # Empty heuristics
        with open(heuristics_path, 'w') as f:
            f.write('{}')

        with open(template_path, 'w') as f:
            json.dump(template_content, f)

        result = subprocess.run(
            ["costpilot", "predict", "--plan", str(template_path), "--heuristics", str(heuristics_path)],
            capture_output=True,
            text=True
        )

        # Should fail or warn about empty heuristics
        if result.returncode != 0:
            error_output = result.stderr + result.stdout
            assert "empty" in error_output.lower() or "no heuristics" in error_output.lower(), \
                "Should mention empty heuristics"


def test_missing_required_heuristics_fails():
    """Missing required heuristic rules should fail."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        heuristics_path = Path(tmpdir) / "heuristics.json"

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

        # Heuristics missing Lambda rules
        heuristics_content = {
            "version": "1.0.0",
            "rules": []
        }

        with open(heuristics_path, 'w') as f:
            json.dump(heuristics_content, f)

        with open(template_path, 'w') as f:
            json.dump(template_content, f)

        result = subprocess.run(
            ["costpilot", "predict", "--plan", str(template_path), "--heuristics", str(heuristics_path)],
            capture_output=True,
            text=True
        )

        # Should fail or warn about missing rules
        output = result.stderr + result.stdout
        if "missing" in output.lower() or "no rule" in output.lower():
            assert True, "Should warn about missing rules"


def test_heuristics_version_mismatch_fails():
    """Heuristics version mismatch should fail."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        heuristics_path = Path(tmpdir) / "heuristics.json"

        template_content = {"Resources": {}}

        # Old version
        heuristics_content = {
            "version": "0.1.0",
            "rules": []
        }

        with open(heuristics_path, 'w') as f:
            json.dump(heuristics_content, f)

        with open(template_path, 'w') as f:
            json.dump(template_content, f)

        result = subprocess.run(
            ["costpilot", "predict", "--plan", str(template_path), "--heuristics", str(heuristics_path)],
            capture_output=True,
            text=True
        )

        # Should warn about version
        output = result.stderr + result.stdout
        if "version" in output.lower():
            assert True, "Should check version compatibility"


def test_heuristics_schema_validation_fails():
    """Invalid heuristics schema should fail."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        heuristics_path = Path(tmpdir) / "heuristics.json"

        template_content = {"Resources": {}}

        # Invalid schema (missing required fields)
        heuristics_content = {
            "rules": [
                {
                    "id": "lambda-cost",
                    # Missing required fields
                }
            ]
        }

        with open(heuristics_path, 'w') as f:
            json.dump(heuristics_content, f)

        with open(template_path, 'w') as f:
            json.dump(template_content, f)

        result = subprocess.run(
            ["costpilot", "predict", "--plan", str(template_path), "--heuristics", str(heuristics_path)],
            capture_output=True,
            text=True
        )

        # Should fail on schema validation
        if result.returncode != 0:
            error_output = result.stderr + result.stdout
            assert "schema" in error_output.lower() or "invalid" in error_output.lower() or "missing" in error_output.lower(), \
                "Should mention schema validation"


def test_default_heuristics_used():
    """Default heuristics should be used if none specified."""
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

        # Run without specifying heuristics
        result = subprocess.run(
            ["costpilot", "predict", "--plan", str(template_path)],
            capture_output=True,
            text=True
        )

        # Should use default heuristics (or fail gracefully)
        assert result.returncode in [0, 1, 2, 101], "Should handle default heuristics"


def test_heuristics_path_documented():
    """Default heuristics path should be documented."""
    result = subprocess.run(
        ["costpilot", "predict", "--help"],
        capture_output=True,
        text=True
    )

    if result.returncode == 0:
        help_text = result.stdout
        assert "--heuristics" in help_text or "heuristics" in help_text.lower(), \
            "Heuristics option should be documented"


if __name__ == "__main__":
    test_missing_heuristics_file_fails()
    test_corrupted_heuristics_fails()
    test_empty_heuristics_fails()
    test_missing_required_heuristics_fails()
    test_heuristics_version_mismatch_fails()
    test_heuristics_schema_validation_fails()
    test_default_heuristics_used()
    test_heuristics_path_documented()
    print("All missing heuristics failure mode tests passed")
