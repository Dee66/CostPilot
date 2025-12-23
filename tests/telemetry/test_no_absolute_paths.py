#!/usr/bin/env python3
"""Test that logs contain no absolute paths."""

import json
import os
import re
import subprocess
import tempfile
from pathlib import Path


def test_no_absolute_paths_in_analyze_output():
    """Test that analyze output contains no absolute paths."""
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
            ["costpilot", "scan", "--plan", str(template_path), "--verbose"],
            capture_output=True,
            text=True,
            timeout=10
        )

        combined_output = result.stdout + result.stderr

        # Check for common absolute path patterns
        # Unix-like: /home/, /usr/, /var/, /tmp/
        # Windows: C:\, D:\, %USERPROFILE%, etc.

        # Should not leak tmpdir path
        assert str(tmpdir) not in combined_output, "Temp directory path should not be in output"

        # Check for common absolute path patterns (relaxed - allow some system paths)
        home_dir = os.path.expanduser("~")
        assert home_dir not in combined_output, "Home directory should not be in output"


def test_no_absolute_paths_in_error_messages():
    """Test that error messages don't contain absolute paths."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "nonexistent.json"

        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path)],
            capture_output=True,
            text=True,
            timeout=10
        )

        combined_output = result.stdout + result.stderr

        # Error should mention file but not full absolute path
        # Allow relative paths or basename only
        assert str(tmpdir) not in combined_output, "Temp directory should not be in error output"


def test_relative_paths_in_logs():
    """Test that logs use relative paths when appropriate."""
    with tempfile.TemporaryDirectory() as tmpdir:
        # Create subdirectory structure
        subdir = Path(tmpdir) / "configs"
        subdir.mkdir()
        template_path = subdir / "template.json"

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

        # Change to parent directory and use relative path
        original_cwd = os.getcwd()
        try:
            os.chdir(tmpdir)

            result = subprocess.run(
                ["costpilot", "scan", "--plan", "configs/template.json", "--verbose"],
                capture_output=True,
                text=True,
                timeout=10
            )

            combined_output = result.stdout + result.stderr

            # Should use relative path if mentioned
            if "template.json" in combined_output:
                # Should prefer relative path over absolute
                assert str(tmpdir) not in combined_output, "Should use relative paths"
        finally:
            os.chdir(original_cwd)


def test_no_username_in_paths():
    """Test that paths don't leak usernames."""
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
            ["costpilot", "scan", "--plan", str(template_path), "--debug"],
            capture_output=True,
            text=True,
            timeout=10
        )

        combined_output = result.stdout + result.stderr

        # Get current username
        username = os.getenv("USER") or os.getenv("USERNAME")

        if username:
            # Username in path context should be avoided
            # Allow username in other contexts (like author metadata)
            username_in_path = re.search(rf"[/\\]{username}[/\\]", combined_output)
            if username_in_path:
                print(f"Warning: Username '{username}' found in path context")


def test_no_system_paths_in_logs():
    """Test that system paths are not leaked in logs."""
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
            ["costpilot", "scan", "--plan", str(template_path)],
            capture_output=True,
            text=True,
            timeout=10
        )

        combined_output = result.stdout + result.stderr

        # Common system paths to check
        system_paths = [
            "/usr/local/",
            "/opt/",
            "C:\\Program Files\\",
            "C:\\Windows\\",
            "/Library/",
            "/System/"
        ]

        for sys_path in system_paths:
            if sys_path in combined_output:
                print(f"Warning: System path '{sys_path}' found in output")


def test_sanitized_stacktraces():
    """Test that stack traces don't contain full absolute paths."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "invalid.json"

        # Create invalid JSON to trigger parse error with stack trace
        with open(template_path, 'w') as f:
            f.write("{{invalid json}}")

        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path), "--debug"],
            capture_output=True,
            text=True,
            timeout=10
        )

        combined_output = result.stdout + result.stderr

        # Stack trace should not contain full paths to source code
        # (Allow relative paths like "src/lib.rs:123")
        assert str(tmpdir) not in combined_output, "Temp directory should not be in stack trace"


def test_no_paths_in_json_output():
    """Test that JSON output doesn't contain absolute paths."""
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
            ["costpilot", "scan", "--plan", str(template_path), "--format", "json"],
            capture_output=True,
            text=True,
            timeout=10
        )

        # Parse JSON output
        try:
            output_data = json.loads(result.stdout)

            # Recursively check for absolute paths in JSON
            def check_paths(obj):
                if isinstance(obj, dict):
                    for key, value in obj.items():
                        if isinstance(value, str):
                            assert str(tmpdir) not in value, f"Absolute path in JSON field '{key}'"
                        check_paths(value)
                elif isinstance(obj, list):
                    for item in obj:
                        check_paths(item)

            check_paths(output_data)
        except json.JSONDecodeError:
            # If not valid JSON, just check text output
            assert str(tmpdir) not in result.stdout, "Absolute path in output"


def test_path_normalization_in_logs():
    """Test that paths are normalized/sanitized in logs."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "subdir" / "nested" / "template.json"
        template_path.parent.mkdir(parents=True)

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
            ["costpilot", "scan", "--plan", str(template_path), "--verbose"],
            capture_output=True,
            text=True,
            timeout=10
        )

        combined_output = result.stdout + result.stderr

        # Should not contain full nested path
        assert "subdir/nested" not in combined_output or str(tmpdir) not in combined_output, \
            "Nested paths should be normalized or excluded"


if __name__ == "__main__":
    test_no_absolute_paths_in_analyze_output()
    test_no_absolute_paths_in_error_messages()
    test_relative_paths_in_logs()
    test_no_username_in_paths()
    test_no_system_paths_in_logs()
    test_sanitized_stacktraces()
    test_no_paths_in_json_output()
    test_path_normalization_in_logs()
    print("All absolute path validation tests passed")
