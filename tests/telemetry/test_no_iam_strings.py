#!/usr/bin/env python3
"""Test that logs contain no IAM-like strings."""

import json
import re
import subprocess
import tempfile
from pathlib import Path


def test_no_aws_access_keys():
    """Test that AWS access keys are not in logs."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"

        # Template with AWS key-like content
        template_content = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "Environment": {
                            "Variables": {
                                "AWS_ACCESS_KEY_ID": "AKIAIOSFODNN7EXAMPLE"
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

        # Should not contain AWS access key
        assert "AKIAIOSFODNN7EXAMPLE" not in combined_output, "AWS access key should be redacted"

        # Check for AKIA pattern (AWS access key prefix)
        akia_pattern = re.compile(r"AKIA[0-9A-Z]{16}")
        matches = akia_pattern.findall(combined_output)
        assert len(matches) == 0, f"AWS access key pattern found: {matches}"


def test_no_aws_secret_keys():
    """Test that AWS secret keys are not in logs."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"

        template_content = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "Environment": {
                            "Variables": {
                                "AWS_SECRET_ACCESS_KEY": "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY"
                            }
                        }
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

        # Should not contain secret key
        assert "wJalrXUtnFEMI/K7MDENG" not in combined_output, "AWS secret key should be redacted"
        assert "bPxRfiCYEXAMPLEKEY" not in combined_output, "AWS secret key should be redacted"


def test_no_session_tokens():
    """Test that session tokens are not in logs."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"

        template_content = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "Environment": {
                            "Variables": {
                                "AWS_SESSION_TOKEN": "FwoGZXIvYXdzEBYaDH8xM2M5N3J4TmV4YyKxARK4EXAMPLE"
                            }
                        }
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

        # Should not contain session token
        assert "FwoGZXIvYXdzEBYaDH8xM2M5N3J4TmV4YyKxARK4EXAMPLE" not in combined_output, \
            "Session token should be redacted"


def test_no_iam_role_arns():
    """Test that IAM role ARNs are sanitized in logs."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"

        template_content = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "Role": "arn:aws:iam::123456789012:role/MySecretRole"
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

        # Account ID should be sanitized (or entire ARN redacted)
        if "arn:aws:iam::" in combined_output:
            # If ARNs are shown, account ID should be masked
            assert "123456789012" not in combined_output, "Account ID should be sanitized"


def test_no_account_ids():
    """Test that AWS account IDs are not in logs."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"

        template_content = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "Description": "Lambda for account 123456789012"
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

        # 12-digit AWS account IDs should be redacted
        account_pattern = re.compile(r"\b\d{12}\b")
        matches = account_pattern.findall(combined_output)

        # Allow some exceptions (like cost values with 12 digits)
        # But specific account ID should be redacted
        assert "123456789012" not in combined_output, "Account ID should be redacted"


def test_no_iam_user_names():
    """Test that IAM user names are not leaked in logs."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"

        template_content = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "Environment": {
                            "Variables": {
                                "IAM_USER": "arn:aws:iam::123456789012:user/admin-user"
                            }
                        }
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

        # IAM user ARN should be redacted
        assert "user/admin-user" not in combined_output, "IAM user name should be redacted"


def test_no_assume_role_credentials():
    """Test that assume role credentials are not in logs."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"

        template_content = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "Environment": {
                            "Variables": {
                                "CREDENTIALS": json.dumps({
                                    "AccessKeyId": "ASIATESTACCESSKEY123",
                                    "SecretAccessKey": "secret123",
                                    "SessionToken": "token123"
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
            ["costpilot", "scan", "--plan", str(template_path)],
            capture_output=True,
            text=True,
            timeout=10
        )

        combined_output = result.stdout + result.stderr

        # Credentials should be redacted
        assert "ASIATESTACCESSKEY123" not in combined_output, "Access key should be redacted"
        assert "secret123" not in combined_output, "Secret should be redacted"
        assert "token123" not in combined_output, "Token should be redacted"


def test_no_kms_key_ids():
    """Test that KMS key IDs are sanitized in logs."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"

        template_content = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "KmsKeyArn": "arn:aws:kms:us-east-1:123456789012:key/12345678-1234-1234-1234-123456789012"
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

        # KMS key GUID should be sanitized
        if "arn:aws:kms:" in combined_output:
            # Key ID should be masked
            assert "12345678-1234-1234-1234-123456789012" not in combined_output, \
                "KMS key ID should be sanitized"


def test_no_sts_credentials():
    """Test that STS credentials are not in logs."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"

        template_content = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "Environment": {
                            "Variables": {
                                "STS_CREDS": "ASIA1234567890ABCDEF"
                            }
                        }
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

        # STS credentials (ASIA prefix) should be redacted
        assert "ASIA1234567890ABCDEF" not in combined_output, "STS credentials should be redacted"


def test_no_policy_documents_with_sensitive_data():
    """Test that policy documents with sensitive data are sanitized."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        policy_path = Path(tmpdir) / "policy.json"

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

        policy_content = {
            "version": "1.0.0",
            "metadata": {
                "created_by": "arn:aws:iam::123456789012:user/admin"
            },
            "rules": []
        }

        with open(template_path, 'w') as f:
            json.dump(template_content, f)

        with open(policy_path, 'w') as f:
            json.dump(policy_content, f)

        result = subprocess.run(
            ["costpilot", "check", "--plan", str(template_path), "--policy", str(policy_path)],
            capture_output=True,
            text=True,
            timeout=10
        )

        combined_output = result.stdout + result.stderr

        # IAM ARN in policy should be sanitized
        assert "123456789012" not in combined_output, "Account ID in policy should be sanitized"


if __name__ == "__main__":
    test_no_aws_access_keys()
    test_no_aws_secret_keys()
    test_no_session_tokens()
    test_no_iam_role_arns()
    test_no_account_ids()
    test_no_iam_user_names()
    test_no_assume_role_credentials()
    test_no_kms_key_ids()
    test_no_sts_credentials()
    test_no_policy_documents_with_sensitive_data()
    print("All IAM-like string validation tests passed")
