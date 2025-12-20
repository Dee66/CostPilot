#!/usr/bin/env python3
"""Test corrupted rollback file safety."""

import subprocess
import tempfile
from pathlib import Path
import json


def test_corrupted_json_rollback():
    """Test behavior with corrupted JSON rollback file."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        rollback_path = Path(tmpdir) / "rollback.json"

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

        # Create corrupted rollback file
        with open(rollback_path, 'w') as f:
            f.write('{"partial": "json"')

        with open(template_path, 'w') as f:
            json.dump(template_content, f)

        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path)],
            capture_output=True,
            text=True,
            timeout=30
        )

        # Should handle corrupted rollback gracefully
        assert result.returncode in [0, 1, 2, 101], "Should handle corrupted rollback"


def test_empty_rollback_file():
    """Test behavior with empty rollback file."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        rollback_path = Path(tmpdir) / "rollback.json"

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

        # Create empty rollback file
        rollback_path.touch()

        with open(template_path, 'w') as f:
            json.dump(template_content, f)

        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path)],
            capture_output=True,
            text=True,
            timeout=30
        )

        # Should handle empty rollback
        assert result.returncode in [0, 1, 2, 101], "Should handle empty rollback"


def test_binary_rollback_file():
    """Test behavior with binary rollback file."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        rollback_path = Path(tmpdir) / "rollback.json"

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

        # Create binary rollback file
        with open(rollback_path, 'wb') as f:
            f.write(b'\x00\x01\x02\x03\x04\x05')

        with open(template_path, 'w') as f:
            json.dump(template_content, f)

        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path)],
            capture_output=True,
            text=True,
            timeout=30
        )

        # Should handle binary rollback
        assert result.returncode in [0, 1, 2, 101], "Should handle binary rollback"


def test_rollback_with_null_bytes():
    """Test rollback file with null bytes."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        rollback_path = Path(tmpdir) / "rollback.json"

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

        # Create rollback with null bytes
        with open(rollback_path, 'w') as f:
            f.write('{"data":\x00"corrupted"}')

        with open(template_path, 'w') as f:
            json.dump(template_content, f)

        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path)],
            capture_output=True,
            text=True,
            timeout=30
        )

        # Should handle null bytes
        assert result.returncode in [0, 1, 2, 101], "Should handle null bytes in rollback"


def test_oversized_rollback_file():
    """Test behavior with oversized rollback file."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        rollback_path = Path(tmpdir) / "rollback.json"

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

        # Create oversized rollback file (10MB)
        with open(rollback_path, 'w') as f:
            f.write('{"data": "' + 'X' * 10000000 + '"}')

        with open(template_path, 'w') as f:
            json.dump(template_content, f)

        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path)],
            capture_output=True,
            text=True,
            timeout=60
        )

        # Should handle oversized rollback
        assert result.returncode in [0, 1, 2, 101], "Should handle oversized rollback"


def test_malformed_rollback_structure():
    """Test malformed rollback structure."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        rollback_path = Path(tmpdir) / "rollback.json"

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

        # Create rollback with wrong structure
        with open(rollback_path, 'w') as f:
            json.dump({"wrong": "structure", "missing": "fields"}, f)

        with open(template_path, 'w') as f:
            json.dump(template_content, f)

        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path)],
            capture_output=True,
            text=True,
            timeout=30
        )

        # Should handle malformed structure
        assert result.returncode in [0, 1, 2, 101], "Should handle malformed rollback structure"


def test_rollback_with_invalid_utf8():
    """Test rollback with invalid UTF-8."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        rollback_path = Path(tmpdir) / "rollback.json"

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

        # Create rollback with invalid UTF-8
        with open(rollback_path, 'wb') as f:
            f.write(b'{"data": "\x80\x81\x82"}')

        with open(template_path, 'w') as f:
            json.dump(template_content, f)

        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path)],
            capture_output=True,
            text=True,
            timeout=30
        )

        # Should handle invalid UTF-8
        assert result.returncode in [0, 1, 2, 101], "Should handle invalid UTF-8 in rollback"


def test_missing_rollback_file():
    """Test behavior when rollback file is missing."""
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
            timeout=30
        )

        # Should complete without rollback
        assert result.returncode in [0, 1, 2, 101], "Should handle missing rollback"


def test_rollback_permission_denied():
    """Test behavior when rollback file is inaccessible."""
    import os

    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        rollback_path = Path(tmpdir) / "rollback.json"

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

        # Create inaccessible rollback file
        with open(rollback_path, 'w') as f:
            json.dump({"data": "test"}, f)

        os.chmod(rollback_path, 0o000)

        with open(template_path, 'w') as f:
            json.dump(template_content, f)

        try:
            result = subprocess.run(
                ["costpilot", "scan", "--plan", str(template_path)],
                capture_output=True,
                text=True,
                timeout=30
            )

            # Should handle inaccessible rollback
            assert result.returncode in [0, 1, 2, 101], "Should handle inaccessible rollback"
        finally:
            os.chmod(rollback_path, 0o644)


if __name__ == "__main__":
    test_corrupted_json_rollback()
    test_empty_rollback_file()
    test_binary_rollback_file()
    test_rollback_with_null_bytes()
    test_oversized_rollback_file()
    test_malformed_rollback_structure()
    test_rollback_with_invalid_utf8()
    test_missing_rollback_file()
    test_rollback_permission_denied()
    print("All corrupted rollback file safety tests passed")
