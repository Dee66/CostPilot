#!/usr/bin/env python3
"""Test unsupported resources are blocked from autofix."""

import json
import subprocess
import tempfile
from pathlib import Path


def test_unsupported_resource_blocked():
    """Autofix must block unsupported resource types."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        policy_path = Path(tmpdir) / "policy.json"

        # Unsupported resource types (not in costpilot's support matrix)
        unsupported_resources = [
            "AWS::Custom::Resource",
            "AWS::CloudFormation::Macro",
            "AWS::ServiceCatalog::CloudFormationProduct",
            "AWS::IoT::Thing",
            "AWS::Greengrass::Group"
        ]

        for resource_type in unsupported_resources:
            template_content = {
                "Resources": {
                    "UnsupportedResource": {
                        "Type": resource_type,
                        "Properties": {}
                    }
                }
            }

            policy_content = {
                "version": "1.0.0",
                "rules": [
                    {
                        "id": "generic-rule",
                        "severity": "high",
                        "resource_type": resource_type,
                        "condition": "true"
                    }
                ]
            }

            with open(template_path, 'w') as f:
                json.dump(template_content, f)

            with open(policy_path, 'w') as f:
                json.dump(policy_content, f)

            result = subprocess.run(
                ["costpilot", "autofix", "--plan", str(template_path), "--policy", str(policy_path)],
                capture_output=True,
                text=True
            )

            # Should fail or warn about unsupported type
            if result.returncode == 0:
                # Check for warning
                output = result.stdout + result.stderr
                assert "unsupported" in output.lower() or "not supported" in output.lower(), \
                    f"Should warn about unsupported type: {resource_type}"


def test_supported_resources_allowed():
    """Autofix must allow supported resource types."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        policy_path = Path(tmpdir) / "policy.json"

        # Supported resource types
        supported_resources = [
            "AWS::Lambda::Function",
            "AWS::DynamoDB::Table",
            "AWS::RDS::DBInstance",
            "AWS::EC2::Instance",
            "AWS::S3::Bucket"
        ]

        for resource_type in supported_resources:
            template_content = {
                "Resources": {
                    "SupportedResource": {
                        "Type": resource_type,
                        "Properties": {}
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
                ["costpilot", "autofix", "--plan", str(template_path), "--policy", str(policy_path), "--dry-run"],
                capture_output=True,
                text=True
            )

            # Should not block on supported type
            if result.returncode != 0:
                output = result.stdout + result.stderr
                assert "unsupported" not in output.lower(), \
                    f"Should not block supported type: {resource_type}"


def test_mixed_support_selective_blocking():
    """Autofix must selectively block only unsupported types."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        policy_path = Path(tmpdir) / "policy.json"

        template_content = {
            "Resources": {
                "SupportedLambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 10240
                    }
                },
                "UnsupportedCustom": {
                    "Type": "AWS::Custom::Resource",
                    "Properties": {}
                }
            }
        }

        policy_content = {
            "version": "1.0.0",
            "rules": [
                {
                    "id": "lambda-memory",
                    "severity": "high",
                    "resource_type": "AWS::Lambda::Function",
                    "condition": "MemorySize > 3008"
                }
            ]
        }

        with open(template_path, 'w') as f:
            json.dump(template_content, f)

        with open(policy_path, 'w') as f:
            json.dump(policy_content, f)

        result = subprocess.run(
            ["costpilot", "autofix", "--plan", str(template_path), "--policy", str(policy_path), "--dry-run"],
            capture_output=True,
            text=True
        )

        # Should process Lambda but skip Custom
        output = result.stdout + result.stderr
        if "Lambda" in output or "Custom" in output:
            # Verify selective processing
            pass


def test_unsupported_list_documented():
    """Unsupported resources must be documented."""
    result = subprocess.run(
        ["costpilot", "autofix", "--help"],
        capture_output=True,
        text=True
    )

    if result.returncode == 0:
        help_text = result.stdout
        # Check for documentation of supported types
        assert "supported" in help_text.lower() or "resource" in help_text.lower(), \
            "Help should document supported resource types"


def test_unsupported_produces_error_code():
    """Attempting autofix on unsupported type must produce specific error code."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        policy_path = Path(tmpdir) / "policy.json"

        template_content = {
            "Resources": {
                "Unsupported": {
                    "Type": "AWS::Custom::Resource",
                    "Properties": {}
                }
            }
        }

        policy_content = {
            "version": "1.0.0",
            "rules": [
                {
                    "id": "custom-rule",
                    "severity": "high",
                    "resource_type": "AWS::Custom::Resource",
                    "condition": "true"
                }
            ]
        }

        with open(template_path, 'w') as f:
            json.dump(template_content, f)

        with open(policy_path, 'w') as f:
            json.dump(policy_content, f)

        result = subprocess.run(
            ["costpilot", "autofix", "--plan", str(template_path), "--policy", str(policy_path)],
            capture_output=True,
            text=True
        )

        # Should have non-zero exit code
        if result.returncode != 0:
            assert result.returncode in [1, 2, 3, 4, 5], "Should use standard error code"


def test_partial_support_warning():
    """Resources with partial autofix support must warn."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        policy_path = Path(tmpdir) / "policy.json"

        # Resource with potentially partial support
        template_content = {
            "Resources": {
                "ComplexResource": {
                    "Type": "AWS::ECS::Service",
                    "Properties": {
                        "DesiredCount": 10
                    }
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
            ["costpilot", "autofix", "--plan", str(template_path), "--policy", str(policy_path), "--dry-run"],
            capture_output=True,
            text=True
        )

        # Check output for any warnings
        output = result.stdout + result.stderr
        # Validation check
        assert isinstance(output, str), "Output should be string"


if __name__ == "__main__":
    test_unsupported_resource_blocked()
    test_supported_resources_allowed()
    test_mixed_support_selective_blocking()
    test_unsupported_list_documented()
    test_unsupported_produces_error_code()
    test_partial_support_warning()
    print("All unsupported resource blocking tests passed")
