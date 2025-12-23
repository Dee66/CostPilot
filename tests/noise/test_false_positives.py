#!/usr/bin/env python3
"""Test noise and false positive handling."""

import json
import subprocess
import tempfile
from pathlib import Path


def test_empty_file_no_findings():
    """Empty template file should produce no findings."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "empty.json"
        policy_path = Path(tmpdir) / "policy.json"

        # Empty file
        with open(template_path, 'w') as f:
            f.write("")

        policy_content = {
            "version": "1.0.0",
            "rules": []
        }

        with open(policy_path, 'w') as f:
            json.dump(policy_content, f)

        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path), "--policy", str(policy_path)],
            capture_output=True,
            text=True
        )

        # Should produce no findings (or structured error)
        output = result.stdout + result.stderr
        if "finding" in output.lower():
            assert "0 finding" in output.lower() or "no finding" in output.lower(), \
                "Empty file should produce no findings"


def test_invalid_json_structured_error():
    """Invalid JSON should produce structured INVALID_PLAN error."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "invalid.json"
        policy_path = Path(tmpdir) / "policy.json"

        # Invalid JSON
        with open(template_path, 'w') as f:
            f.write('{"Resources": {invalid json}')

        policy_content = {
            "version": "1.0.0",
            "rules": []
        }

        with open(policy_path, 'w') as f:
            json.dump(policy_content, f)

        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path), "--policy", str(policy_path), "--format", "json"],
            capture_output=True,
            text=True
        )

        # Should produce INVALID_PLAN error code
        assert result.returncode != 0, "Invalid JSON should fail"

        output = result.stdout + result.stderr
        if result.stdout:
            try:
                error_data = json.loads(result.stdout)
                assert "error" in error_data or "INVALID_PLAN" in str(error_data), \
                    "Should produce structured error"
            except json.JSONDecodeError:
                # Structured error in stderr
                assert "invalid" in output.lower() or "parse" in output.lower(), \
                    "Should mention parsing error"


def test_out_of_order_modules_deterministic():
    """Out-of-order resource modules should produce deterministic output."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template1_path = Path(tmpdir) / "template1.json"
        template2_path = Path(tmpdir) / "template2.json"
        policy_path = Path(tmpdir) / "policy.json"

        # Same resources, different order
        template1 = {
            "Resources": {
                "Lambda": {"Type": "AWS::Lambda::Function", "Properties": {"MemorySize": 1024}},
                "DynamoDB": {"Type": "AWS::DynamoDB::Table", "Properties": {"BillingMode": "PAY_PER_REQUEST"}},
                "S3": {"Type": "AWS::S3::Bucket", "Properties": {}}
            }
        }

        template2 = {
            "Resources": {
                "S3": {"Type": "AWS::S3::Bucket", "Properties": {}},
                "Lambda": {"Type": "AWS::Lambda::Function", "Properties": {"MemorySize": 1024}},
                "DynamoDB": {"Type": "AWS::DynamoDB::Table", "Properties": {"BillingMode": "PAY_PER_REQUEST"}}
            }
        }

        policy_content = {
            "version": "1.0.0",
            "rules": []
        }

        with open(template1_path, 'w') as f:
            json.dump(template1, f)

        with open(template2_path, 'w') as f:
            json.dump(template2, f)

        with open(policy_path, 'w') as f:
            json.dump(policy_content, f)

        # Analyze both
        result1 = subprocess.run(
            ["costpilot", "scan", "--plan", str(template1_path), "--policy", str(policy_path)],
            capture_output=True,
            text=True
        )

        result2 = subprocess.run(
            ["costpilot", "scan", "--plan", str(template2_path), "--policy", str(policy_path)],
            capture_output=True,
            text=True
        )

        # Outputs should be identical (deterministic ordering)
        assert result1.stdout == result2.stdout, "Out-of-order modules should produce deterministic output"


def test_providers_only_diff_no_findings():
    """Provider-only changes should produce no cost findings."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        policy_path = Path(tmpdir) / "policy.json"

        # Template with only provider config (no resources)
        template_content = {
            "terraform": {
                "required_providers": {
                    "aws": {
                        "source": "hashicorp/aws",
                        "version": "~> 5.0"
                    }
                }
            },
            "provider": {
                "aws": {
                    "region": "us-east-1"
                }
            }
        }

        policy_content = {
            "version": "1.0.0",
            "rules": []
        }

        with open(template_path, 'w') as f:
            json.dump(template_content, f)

        with open(policy_path, 'w') as f:
            json.dump(policy_content, f)

        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path), "--policy", str(policy_path)],
            capture_output=True,
            text=True
        )

        # Should produce no findings
        output = result.stdout + result.stderr
        if "finding" in output.lower():
            assert "0 finding" in output.lower() or "no finding" in output.lower(), \
                "Provider-only changes should produce no findings"


def test_mixed_crlf_lf_normalization():
    """Mixed CRLF/LF line endings should be normalized."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_crlf = Path(tmpdir) / "template_crlf.json"
        template_lf = Path(tmpdir) / "template_lf.json"
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
            "rules": []
        }

        # Write with CRLF
        json_str = json.dumps(template_content, indent=2)
        with open(template_crlf, 'w', newline='\r\n') as f:
            f.write(json_str)

        # Write with LF
        with open(template_lf, 'w', newline='\n') as f:
            f.write(json_str)

        with open(policy_path, 'w') as f:
            json.dump(policy_content, f)

        # Analyze both
        result_crlf = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_crlf), "--policy", str(policy_path)],
            capture_output=True,
            text=True
        )

        result_lf = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_lf), "--policy", str(policy_path)],
            capture_output=True,
            text=True
        )

        # Outputs should be identical (normalized)
        assert result_crlf.stdout == result_lf.stdout, "Line endings should be normalized"


def test_whitespace_only_changes_ignored():
    """Whitespace-only changes should not affect findings."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template1_path = Path(tmpdir) / "template1.json"
        template2_path = Path(tmpdir) / "template2.json"
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
            "rules": []
        }

        # Write with different indentation
        with open(template1_path, 'w') as f:
            json.dump(template_content, f, indent=2)

        with open(template2_path, 'w') as f:
            json.dump(template_content, f, indent=4)

        with open(policy_path, 'w') as f:
            json.dump(policy_content, f)

        # Analyze both
        result1 = subprocess.run(
            ["costpilot", "scan", "--plan", str(template1_path), "--policy", str(policy_path)],
            capture_output=True,
            text=True
        )

        result2 = subprocess.run(
            ["costpilot", "scan", "--plan", str(template2_path), "--policy", str(policy_path)],
            capture_output=True,
            text=True
        )

        # Outputs should be identical
        assert result1.stdout == result2.stdout, "Whitespace changes should not affect findings"


def test_comments_preserved_in_yaml():
    """YAML comments should not affect analysis."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template1_path = Path(tmpdir) / "template1.yaml"
        template2_path = Path(tmpdir) / "template2.yaml"
        policy_path = Path(tmpdir) / "policy.json"

        # With comments
        yaml1 = """# Production template
Resources:
  Lambda:
    Type: AWS::Lambda::Function
    Properties:
      MemorySize: 1024  # Default memory
"""

        # Without comments
        yaml2 = """Resources:
  Lambda:
    Type: AWS::Lambda::Function
    Properties:
      MemorySize: 1024
"""

        policy_content = {
            "version": "1.0.0",
            "rules": []
        }

        with open(template1_path, 'w') as f:
            f.write(yaml1)

        with open(template2_path, 'w') as f:
            f.write(yaml2)

        with open(policy_path, 'w') as f:
            json.dump(policy_content, f)

        # Analyze both
        result1 = subprocess.run(
            ["costpilot", "scan", "--plan", str(template1_path), "--policy", str(policy_path)],
            capture_output=True,
            text=True
        )

        result2 = subprocess.run(
            ["costpilot", "scan", "--plan", str(template2_path), "--policy", str(policy_path)],
            capture_output=True,
            text=True
        )

        # Outputs should be identical
        assert result1.stdout == result2.stdout, "Comments should not affect analysis"


if __name__ == "__main__":
    test_empty_file_no_findings()
    test_invalid_json_structured_error()
    test_out_of_order_modules_deterministic()
    test_providers_only_diff_no_findings()
    test_mixed_crlf_lf_normalization()
    test_whitespace_only_changes_ignored()
    test_comments_preserved_in_yaml()
    print("All noise and false positive tests passed")
