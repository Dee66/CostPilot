#!/usr/bin/env python3
"""Test that path traversal is blocked."""

import json
import os
import subprocess
import tempfile
from pathlib import Path


def test_path_traversal_in_template_path():
    """Test that path traversal in template path is blocked."""
    with tempfile.TemporaryDirectory() as tmpdir:
        # Create a template in the temp directory
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

        # Try to access via path traversal
        traversal_paths = [
            f"{tmpdir}/../../../etc/passwd",
            f"{tmpdir}/../../..",
            "../" * 10 + "etc/passwd",
        ]

        for traversal in traversal_paths:
            result = subprocess.run(
                ["costpilot", "scan", "--plan", traversal],
                capture_output=True,
                text=True,
                timeout=10
            )

            combined_output = result.stdout + result.stderr

            # Should not access sensitive files
            assert "root:x:" not in combined_output, f"Path traversal succeeded: {traversal}"


def test_path_traversal_in_output_path():
    """Test that path traversal in output path is blocked."""
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

        # Try to write to sensitive locations via traversal
        dangerous_outputs = [
            "../../../etc/costpilot_pwned",
            "../" * 10 + "tmp/pwned",
            "/etc/costpilot_pwned",
        ]

        for output in dangerous_outputs:
            result = subprocess.run(
                ["costpilot", "baseline", "generate", "--plan", str(template_path), "--output", output],
                capture_output=True,
                text=True,
                timeout=10
            )

            # Check that file was not created in sensitive location
            pwned_paths = [
                Path("/etc/costpilot_pwned"),
                Path("/tmp/pwned"),
            ]

            for pwned in pwned_paths:
                if pwned.exists():
                    pwned.unlink()  # Clean up
                    assert False, f"Path traversal succeeded: {output} -> {pwned}"


def test_path_traversal_in_policy_path():
    """Test that path traversal in policy path is blocked."""
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

        # Attempt traversal in policy path
        traversal = "../../../etc/passwd"

        result = subprocess.run(
            ["costpilot", "check", "--plan", str(template_path), "--policy", traversal],
            capture_output=True,
            text=True,
            timeout=10
        )

        combined_output = result.stdout + result.stderr

        # Should not read sensitive files
        assert "root:x:" not in combined_output, "Path traversal in policy path"


def test_absolute_path_escape():
    """Test that absolute paths can't escape sandbox."""
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

        # Try absolute paths to sensitive locations
        sensitive_paths = [
            "/etc/passwd",
            "/etc/shadow",
            "/root/.ssh/id_rsa",
        ]

        for sensitive in sensitive_paths:
            result = subprocess.run(
                ["costpilot", "scan", "--plan", sensitive],
                capture_output=True,
                text=True,
                timeout=10
            )

            combined_output = result.stdout + result.stderr

            # Should reject or not access
            assert "root:x:" not in combined_output, f"Accessed sensitive file: {sensitive}"
            assert "BEGIN RSA PRIVATE KEY" not in combined_output, f"Accessed private key: {sensitive}"


def test_symlink_traversal():
    """Test that symlinks can't be used for path traversal."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        symlink_path = Path(tmpdir) / "traversal_link"

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

        # Create symlink to sensitive file
        try:
            os.symlink("/etc/passwd", symlink_path)
        except OSError:
            print("Symlink creation failed, skipping test")
            return

        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(symlink_path)],
            capture_output=True,
            text=True,
            timeout=10
        )

        combined_output = result.stdout + result.stderr

        # Should not follow symlink to sensitive file
        assert "root:x:" not in combined_output, "Symlink traversal succeeded"


def test_windows_path_traversal():
    """Test Windows-specific path traversal patterns."""
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

        # Windows path traversal patterns
        windows_traversals = [
            "..\\..\\..\\Windows\\System32\\config\\SAM",
            "C:\\Windows\\System32\\config\\SAM",
            "\\\\?\\C:\\Windows\\System32\\config\\SAM",
        ]

        for traversal in windows_traversals:
            result = subprocess.run(
                ["costpilot", "scan", "--plan", traversal],
                capture_output=True,
                text=True,
                timeout=10
            )

            # Should reject Windows traversal
            combined_output = result.stdout + result.stderr

            # Check that sensitive Windows files weren't accessed
            # (Test may pass on non-Windows systems)
            if os.name == 'nt':
                # On Windows, verify protection
                assert result.returncode != 0 or "SAM" not in combined_output


def test_url_encoded_traversal():
    """Test that URL-encoded path traversal is blocked."""
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

        # URL-encoded traversal
        encoded_traversals = [
            "..%2F..%2F..%2Fetc%2Fpasswd",
            "%2e%2e%2f%2e%2e%2f%2e%2e%2fetc%2fpasswd",
        ]

        for traversal in encoded_traversals:
            result = subprocess.run(
                ["costpilot", "scan", "--plan", traversal],
                capture_output=True,
                text=True,
                timeout=10
            )

            combined_output = result.stdout + result.stderr

            # Should not decode and traverse
            assert "root:x:" not in combined_output, f"URL-encoded traversal succeeded: {traversal}"


def test_double_encoding_traversal():
    """Test that double-encoded path traversal is blocked."""
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

        # Double-encoded traversal
        double_encoded = "%252e%252e%252f%252e%252e%252f%252e%252e%252fetc%252fpasswd"

        result = subprocess.run(
            ["costpilot", "scan", "--plan", double_encoded],
            capture_output=True,
            text=True,
            timeout=10
        )

        combined_output = result.stdout + result.stderr

        # Should not decode multiple times and traverse
        assert "root:x:" not in combined_output, "Double-encoded traversal succeeded"


def test_null_byte_path_truncation():
    """Test that null bytes don't truncate paths for traversal."""
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

        # Null byte truncation attack
        # e.g., "../../etc/passwd\x00.json" might bypass extension check
        traversal_with_null = f"{tmpdir}/../../../etc/passwd\x00.json"

        result = subprocess.run(
            ["costpilot", "scan", "--plan", traversal_with_null],
            capture_output=True,
            text=True,
            timeout=10
        )

        combined_output = result.stdout + result.stderr

        # Should not truncate at null byte
        assert "root:x:" not in combined_output, "Null byte truncation succeeded"


def test_unicode_normalization_traversal():
    """Test that Unicode normalization doesn't enable traversal."""
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

        # Unicode lookalikes for path traversal
        # (Some systems might normalize these)
        unicode_traversals = [
            "．．/．．/etc/passwd",  # Fullwidth periods
            "﹒﹒/﹒﹒/etc/passwd",  # Small periods
        ]

        for traversal in unicode_traversals:
            result = subprocess.run(
                ["costpilot", "scan", "--plan", traversal],
                capture_output=True,
                text=True,
                timeout=10
            )

            combined_output = result.stdout + result.stderr

            # Should not normalize and traverse
            assert "root:x:" not in combined_output, f"Unicode traversal succeeded: {traversal}"


if __name__ == "__main__":
    test_path_traversal_in_template_path()
    test_path_traversal_in_output_path()
    test_path_traversal_in_policy_path()
    test_absolute_path_escape()
    test_symlink_traversal()
    test_windows_path_traversal()
    test_url_encoded_traversal()
    test_double_encoding_traversal()
    test_null_byte_path_truncation()
    test_unicode_normalization_traversal()
    print("All path traversal blocking tests passed")
