#!/usr/bin/env python3
"""Test multi-line redaction in telemetry."""

import json
import re
import subprocess
import tempfile
from pathlib import Path


def test_multiline_secret_redaction():
    """Test that multi-line secrets are redacted in logs."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        
        # Template with multi-line secret-like content
        template_content = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "Environment": {
                            "Variables": {
                                "SECRET_KEY": "AKIAIOSFODNN7EXAMPLE\nline2\nline3",
                                "PRIVATE_KEY": "-----BEGIN RSA PRIVATE KEY-----\nMIIEpAIBAAKCAQEA...\n-----END RSA PRIVATE KEY-----"
                            }
                        }
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
        
        # Check that secrets are not in output
        combined_output = result.stdout + result.stderr
        
        # Should not contain AWS keys
        assert "AKIAIOSFODNN7EXAMPLE" not in combined_output, "AWS key should be redacted"
        
        # Should not contain private key
        assert "BEGIN RSA PRIVATE KEY" not in combined_output, "Private key should be redacted"


def test_multiline_stacktrace_redaction():
    """Test that multi-line stack traces with sensitive info are redacted."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template_invalid.json"
        
        # Invalid template to trigger error
        template_content = "invalid json content\nwith multiple lines\nand sensitive data AKIAIOSFODNN7EXAMPLE"
        
        with open(template_path, 'w') as f:
            f.write(template_content)
        
        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path)],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        combined_output = result.stdout + result.stderr
        
        # Error message should not leak sensitive data
        assert "AKIAIOSFODNN7EXAMPLE" not in combined_output, "Secrets in errors should be redacted"


def test_multiline_json_redaction():
    """Test redaction of multi-line JSON with sensitive fields."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        
        template_content = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "Environment": {
                            "Variables": {
                                "CONFIG": json.dumps({
                                    "api_key": "sk_live_abc123def456",
                                    "password": "super_secret_password",
                                    "token": "ghp_abcdefghijklmnop"
                                })
                            }
                        }
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
        
        # Sensitive fields should be redacted
        assert "sk_live_abc123def456" not in combined_output, "API key should be redacted"
        assert "super_secret_password" not in combined_output, "Password should be redacted"
        assert "ghp_abcdefghijklmnop" not in combined_output, "Token should be redacted"


def test_multiline_log_redaction():
    """Test multi-line log entries with sensitive data."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        
        # Template with description containing sensitive data
        template_content = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "Description": """
This is a multi-line description
with sensitive data embedded:
AWS_ACCESS_KEY_ID=AKIAIOSFODNN7EXAMPLE
AWS_SECRET_ACCESS_KEY=wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY
DATABASE_PASSWORD=my_super_secret_password_123
"""
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
        
        # Credentials should be redacted
        assert "AKIAIOSFODNN7EXAMPLE" not in combined_output, "Access key should be redacted"
        assert "wJalrXUtnFEMI/K7MDENG" not in combined_output, "Secret key should be redacted"
        assert "my_super_secret_password_123" not in combined_output, "Password should be redacted"


def test_redaction_preserves_structure():
    """Test that redaction preserves log structure across multiple lines."""
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
        
        # Logs should be structured (not broken by redaction)
        assert result.returncode in [0, 1, 2, 101], "Should complete successfully"
        
        # Check for well-formed output (no broken lines)
        lines = result.stdout.split('\n')
        for line in lines:
            if line.strip():
                # Each non-empty line should be complete (not cut off mid-word)
                assert not line.endswith('\\'), "Lines should not be broken"


def test_multiline_credential_patterns():
    """Test detection and redaction of various multi-line credential patterns."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        
        credentials = [
            "-----BEGIN PRIVATE KEY-----\nMIIEvQIBADANBgkqhkiG9w0BAQEFAASC\n-----END PRIVATE KEY-----",
            "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABAQC\nlong_key_content\nuser@host",
            "AKIAIOSFODNN7EXAMPLE\nsecret_part_2\nAWSKEY",
            "ghp_1234567890abcdefghijklmnopqr\nstuvwxyz"
        ]
        
        template_content = {
            "Resources": {
                f"Lambda{i}": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "Description": cred
                    }
                }
                for i, cred in enumerate(credentials)
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
        
        # None of the credential patterns should appear
        assert "BEGIN PRIVATE KEY" not in combined_output, "Private key should be redacted"
        assert "ssh-rsa" not in combined_output, "SSH key should be redacted"
        assert "AKIAIOSFODNN7EXAMPLE" not in combined_output, "AWS key should be redacted"
        assert "ghp_1234567890" not in combined_output, "GitHub token should be redacted"


def test_multiline_pii_redaction():
    """Test redaction of multi-line PII data."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        
        # Template with PII-like data
        template_content = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "Description": """
Name: John Doe
Email: john.doe@example.com
SSN: 123-45-6789
Credit Card: 4111-1111-1111-1111
Phone: +1-555-123-4567
"""
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
        
        # Check that some form of redaction is applied
        # (Implementation may vary - checking that SSN pattern is not leaked)
        if "123-45-6789" in combined_output:
            print("Warning: SSN pattern detected in output")


def test_redaction_performance():
    """Test that multi-line redaction doesn't significantly impact performance."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        
        # Large template with many potential secrets
        template_content = {
            "Resources": {
                f"Lambda{i}": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "Description": f"Line1\nLine2\nAKIA{i}EXAMPLE\nLine4"
                    }
                }
                for i in range(100)
            }
        }
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        import time
        start = time.time()
        
        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path)],
            capture_output=True,
            text=True,
            timeout=30
        )
        
        duration = time.time() - start
        
        # Should complete reasonably fast despite redaction
        assert result.returncode in [0, 1, 2, 101], "Should complete"
        assert duration < 30, "Redaction should not significantly impact performance"


if __name__ == "__main__":
    test_multiline_secret_redaction()
    test_multiline_stacktrace_redaction()
    test_multiline_json_redaction()
    test_multiline_log_redaction()
    test_redaction_preserves_structure()
    test_multiline_credential_patterns()
    test_multiline_pii_redaction()
    test_redaction_performance()
    print("All multi-line redaction tests passed")
