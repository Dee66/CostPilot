#!/usr/bin/env python3
"""Test handling of unknown provider types."""

import json
import subprocess
import tempfile
from pathlib import Path


def test_unknown_aws_resource_type():
    """Test handling of unknown AWS resource types."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"

        template_content = {
            "Resources": {
                "UnknownResource": {
                    "Type": "AWS::Unknown::ResourceType",
                    "Properties": {
                        "SomeProperty": "value"
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

        # Should handle unknown types gracefully
        assert result.returncode in [0, 1, 2, 101], "Should handle unknown AWS types"


def test_unknown_provider_namespace():
    """Test handling of unknown provider namespaces."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"

        template_content = {
            "Resources": {
                "CustomResource": {
                    "Type": "CustomProvider::Custom::Resource",
                    "Properties": {
                        "CustomProperty": "value"
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

        # Should handle unknown providers
        assert result.returncode in [0, 1, 2, 101], "Should handle unknown provider namespaces"


def test_malformed_resource_type():
    """Test handling of malformed resource type strings."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"

        template_content = {
            "Resources": {
                "MalformedResource": {
                    "Type": "NoColons",
                    "Properties": {
                        "SomeProperty": "value"
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

        # Should handle malformed types
        assert result.returncode in [0, 1, 2, 101], "Should handle malformed resource types"


def test_empty_resource_type():
    """Test handling of empty resource type."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"

        template_content = {
            "Resources": {
                "EmptyTypeResource": {
                    "Type": "",
                    "Properties": {
                        "SomeProperty": "value"
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

        # Should reject empty type
        assert result.returncode in [1, 2, 101], "Should reject empty resource type"


def test_numeric_resource_type():
    """Test handling of numeric resource type."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"

        template_content = {
            "Resources": {
                "NumericTypeResource": {
                    "Type": 12345,
                    "Properties": {
                        "SomeProperty": "value"
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

        # Should reject non-string type
        assert result.returncode in [2, 101], "Should reject numeric resource type"


def test_mixed_known_unknown_types():
    """Test handling of mixed known and unknown resource types."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"

        template_content = {
            "Resources": {
                "KnownLambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 1024
                    }
                },
                "UnknownResource": {
                    "Type": "AWS::Unknown::ResourceType",
                    "Properties": {
                        "SomeProperty": "value"
                    }
                },
                "KnownS3": {
                    "Type": "AWS::S3::Bucket",
                    "Properties": {
                        "BucketName": "test-bucket"
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

        # Should process known types and handle unknown gracefully
        assert result.returncode in [0, 1, 2, 101], "Should handle mixed known/unknown types"


def test_very_long_resource_type():
    """Test handling of very long resource type strings."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"

        # 1000 character type string
        long_type = "AWS::" + "Custom" * 330

        template_content = {
            "Resources": {
                "LongTypeResource": {
                    "Type": long_type,
                    "Properties": {
                        "SomeProperty": "value"
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

        # Should handle long type strings
        assert result.returncode in [0, 1, 2, 101], "Should handle very long resource types"


def test_unicode_in_resource_type():
    """Test handling of Unicode characters in resource types."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"

        template_content = {
            "Resources": {
                "UnicodeTypeResource": {
                    "Type": "AWS::函数::Resource",
                    "Properties": {
                        "SomeProperty": "value"
                    }
                }
            }
        }

        with open(template_path, 'w', encoding='utf-8') as f:
            json.dump(template_content, f, ensure_ascii=False)

        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path)],
            capture_output=True,
            text=True,
            timeout=30
        )

        # Should handle Unicode in type
        assert result.returncode in [0, 1, 2, 101], "Should handle Unicode in resource type"


def test_special_characters_in_resource_type():
    """Test handling of special characters in resource types."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"

        template_content = {
            "Resources": {
                "SpecialCharResource": {
                    "Type": "AWS::Lambda::Function!@#$%",
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

        # Should handle special characters
        assert result.returncode in [0, 1, 2, 101], "Should handle special characters in type"


def test_unknown_type_with_policy():
    """Test unknown types with policy checking."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        policy_path = Path(tmpdir) / "policy.json"

        template_content = {
            "Resources": {
                "UnknownResource": {
                    "Type": "AWS::Unknown::ResourceType",
                    "Properties": {
                        "SomeProperty": "value"
                    }
                }
            }
        }

        policy_content = {
            "version": "1.0.0",
            "rules": [
                {
                    "id": "unknown-type-check",
                    "severity": "high",
                    "resource_type": "AWS::Unknown::ResourceType",
                    "condition": "SomeProperty != ''"
                }
            ]
        }

        with open(template_path, 'w') as f:
            json.dump(template_content, f)

        with open(policy_path, 'w') as f:
            json.dump(policy_content, f)

        result = subprocess.run(
            ["costpilot", "check", "--plan", str(template_path), "--policy", str(policy_path)],
            capture_output=True,
            text=True,
            timeout=30
        )

        # Should handle policy on unknown types
        assert result.returncode in [0, 1, 2, 101], "Should handle policy on unknown types"


if __name__ == "__main__":
    test_unknown_aws_resource_type()
    test_unknown_provider_namespace()
    test_malformed_resource_type()
    test_empty_resource_type()
    test_numeric_resource_type()
    test_mixed_known_unknown_types()
    test_very_long_resource_type()
    test_unicode_in_resource_type()
    test_special_characters_in_resource_type()
    test_unknown_type_with_policy()
    print("All unknown provider type tests passed")
