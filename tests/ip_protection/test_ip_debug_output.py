#!/usr/bin/env python3
"""Test IP Protection: No debug mode prints internal heuristics."""

import subprocess
import tempfile
from pathlib import Path
import json


def test_debug_no_heuristics_dump():
    """Test debug mode doesn't dump heuristics."""
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
            ["costpilot", "scan", "--plan", str(template_path), "--debug"],
            capture_output=True,
            text=True,
            timeout=10
        )

        # Debug mode might fail in Free or work with limited output
        if result.returncode == 0 or "--debug" in result.stderr:
            output = (result.stdout + result.stderr).lower()

            # Should not dump internal heuristics
            forbidden = [
                "heuristics_entry:",
                "internal_rule:",
                "premium_heuristics:",
                "encrypted_data:",
                "bundle_content:",
                "heuristics_key:"
            ]

            for item in forbidden:
                assert item not in output, f"Debug should not print: {item}"


def test_verbose_no_internal_details():
    """Test verbose mode doesn't expose internals."""
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
            ["costpilot", "scan", "--plan", str(template_path), "--verbose"],
            capture_output=True,
            text=True,
            timeout=10
        )

        if result.returncode == 0:
            output = (result.stdout + result.stderr).lower()

            # Should not expose internal implementation
            forbidden = [
                "heuristics_version:",
                "internal_api_key:",
                "bundle_decrypt",
                "premium_engine_ptr",
                "license_validation_key"
            ]

            for item in forbidden:
                assert item not in output, f"Verbose should not print: {item}"


def test_log_level_trace_sanitized():
    """Test trace logging doesn't leak secrets."""
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
            ["costpilot", "scan", "--plan", str(template_path), "--log-level", "trace"],
            capture_output=True,
            text=True,
            timeout=10
        )

        # Trace might not be available in Free
        if result.returncode == 0:
            output = (result.stdout + result.stderr)

            # Should not leak secrets
            forbidden = [
                "SECRET_KEY:",
                "API_KEY:",
                "ENCRYPTION_KEY:",
                "PRIVATE_KEY:",
                "LICENSE_TOKEN:"
            ]

            for item in forbidden:
                assert item not in output, f"Trace should not leak: {item}"


def test_debug_output_deterministic():
    """Test debug output is deterministic."""
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

        outputs = []
        for _ in range(3):
            result = subprocess.run(
                ["costpilot", "scan", "--plan", str(template_path), "--debug"],
                capture_output=True,
                text=True,
                timeout=10
            )

            if result.returncode == 0:
                outputs.append(result.stdout + result.stderr)

        # Debug output should be consistent
        if len(outputs) >= 2:
            # Remove timestamps if present
            sanitized = []
            for output in outputs:
                # Remove common timestamp patterns
                import re
                cleaned = re.sub(r'\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}', 'TIMESTAMP', output)
                cleaned = re.sub(r'\d+\.\d+s', 'DURATION', cleaned)
                sanitized.append(cleaned)

            # Should be identical after sanitization
            assert sanitized[0] == sanitized[1], "Debug output should be deterministic"


def test_error_messages_no_internal_paths():
    """Test error messages don't reveal internal paths."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "nonexistent.json"

        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path)],
            capture_output=True,
            text=True,
            timeout=10
        )

        # Should fail (file doesn't exist)
        assert result.returncode != 0

        output = result.stdout + result.stderr

        # Should not reveal internal paths
        forbidden = [
            "/opt/costpilot/internal",
            "/usr/local/share/costpilot/premium",
            "~/.costpilot/pro",
            "/build/src/premium",
            "C:\\CostPilot\\Internal"
        ]

        for path in forbidden:
            assert path not in output, f"Error should not reveal path: {path}"


if __name__ == "__main__":
    test_debug_no_heuristics_dump()
    test_verbose_no_internal_details()
    test_log_level_trace_sanitized()
    test_debug_output_deterministic()
    test_error_messages_no_internal_paths()
    print("All IP Protection: debug output tests passed")
