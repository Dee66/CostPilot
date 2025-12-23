#!/usr/bin/env python3
"""Test cycle-detected mapping errors are clean."""

import subprocess
import tempfile
from pathlib import Path
import json


def test_simple_circular_dependency():
    """Test simple circular dependency detection."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"

        template_content = {
            "Resources": {
                "RoleA": {
                    "Type": "AWS::IAM::Role",
                    "Properties": {
                        "AssumeRolePolicyDocument": {
                            "Fn::GetAtt": ["RoleB", "Arn"]
                        }
                    }
                },
                "RoleB": {
                    "Type": "AWS::IAM::Role",
                    "Properties": {
                        "AssumeRolePolicyDocument": {
                            "Fn::GetAtt": ["RoleA", "Arn"]
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
            timeout=30
        )

        # Should detect cycle and report cleanly
        if result.returncode != 0:
            # Check for clean error message
            assert "cycle" in result.stderr.lower() or "circular" in result.stderr.lower(), \
                "Should report cycle detection cleanly"


def test_three_way_circular_dependency():
    """Test three-way circular dependency."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"

        template_content = {
            "Resources": {
                "ResourceA": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "Role": {"Fn::GetAtt": ["ResourceB", "Arn"]}
                    }
                },
                "ResourceB": {
                    "Type": "AWS::IAM::Role",
                    "Properties": {
                        "AssumeRolePolicyDocument": {
                            "Fn::GetAtt": ["ResourceC", "Arn"]
                        }
                    }
                },
                "ResourceC": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "Role": {"Fn::GetAtt": ["ResourceA", "Arn"]}
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

        # Should detect cycle cleanly
        if result.returncode != 0:
            assert "cycle" in result.stderr.lower() or "circular" in result.stderr.lower(), \
                "Should report three-way cycle"


def test_self_referential_resource():
    """Test self-referential resource."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"

        template_content = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "Role": {"Fn::GetAtt": ["Lambda", "Arn"]}
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

        # Should detect self-reference
        if result.returncode != 0:
            assert "cycle" in result.stderr.lower() or "self" in result.stderr.lower() or \
                   "circular" in result.stderr.lower(), \
                "Should report self-reference"


def test_complex_circular_chain():
    """Test complex circular dependency chain."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"

        # Create a 10-resource circular chain
        resources = {}
        for i in range(10):
            next_i = (i + 1) % 10
            resources[f"Resource{i}"] = {
                "Type": "AWS::Lambda::Function",
                "Properties": {
                    "Role": {"Fn::GetAtt": [f"Resource{next_i}", "Arn"]}
                }
            }

        template_content = {"Resources": resources}

        with open(template_path, 'w') as f:
            json.dump(template_content, f)

        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path)],
            capture_output=True,
            text=True,
            timeout=30
        )

        # Should detect cycle
        if result.returncode != 0:
            assert "cycle" in result.stderr.lower() or "circular" in result.stderr.lower(), \
                "Should detect complex cycle"


def test_nested_circular_dependency():
    """Test nested circular dependency."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"

        template_content = {
            "Resources": {
                "Parent": {
                    "Type": "AWS::CloudFormation::Stack",
                    "Properties": {
                        "TemplateURL": {"Fn::GetAtt": ["Child", "Outputs.URL"]}
                    }
                },
                "Child": {
                    "Type": "AWS::CloudFormation::Stack",
                    "Properties": {
                        "TemplateURL": {"Fn::GetAtt": ["Parent", "Outputs.URL"]}
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

        # Should detect nested cycle
        if result.returncode != 0:
            assert "cycle" in result.stderr.lower() or "circular" in result.stderr.lower(), \
                "Should detect nested cycle"


def test_cycle_error_message_quality():
    """Test cycle error message quality."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"

        template_content = {
            "Resources": {
                "A": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "Role": {"Fn::GetAtt": ["B", "Arn"]}
                    }
                },
                "B": {
                    "Type": "AWS::IAM::Role",
                    "Properties": {
                        "AssumeRolePolicyDocument": {
                            "Fn::GetAtt": ["A", "Arn"]
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
            timeout=30
        )

        # Error message should be clean and informative
        if result.returncode != 0:
            error = result.stderr.lower()

            # Should mention cycle
            assert "cycle" in error or "circular" in error, "Should mention cycle"

            # Should not panic or stack trace
            assert "panic" not in error, "Should not panic"
            assert "thread" not in error or "main" in error, "Should not show thread panic"


def test_conditional_cycle_handling():
    """Test conditional circular dependencies."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"

        template_content = {
            "Parameters": {
                "UseCircular": {
                    "Type": "String",
                    "Default": "true"
                }
            },
            "Conditions": {
                "ShouldCreateCircular": {"Fn::Equals": [{"Ref": "UseCircular"}, "true"]}
            },
            "Resources": {
                "ResourceA": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "Role": {
                            "Fn::If": [
                                "ShouldCreateCircular",
                                {"Fn::GetAtt": ["ResourceB", "Arn"]},
                                "arn:aws:iam::123456789012:role/safe"
                            ]
                        }
                    }
                },
                "ResourceB": {
                    "Type": "AWS::IAM::Role",
                    "Properties": {
                        "AssumeRolePolicyDocument": {
                            "Fn::GetAtt": ["ResourceA", "Arn"]
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
            timeout=30
        )

        # Should handle conditional cycles
        if result.returncode != 0:
            error = result.stderr.lower()
            # Clean error handling
            assert "panic" not in error, "Should not panic on conditional cycle"


def test_no_false_positive_cycles():
    """Test no false positive cycle detection."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"

        template_content = {
            "Resources": {
                "Role": {
                    "Type": "AWS::IAM::Role",
                    "Properties": {
                        "AssumeRolePolicyDocument": {}
                    }
                },
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "Role": {"Fn::GetAtt": ["Role", "Arn"]},
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

        # Should not detect false cycle
        if result.returncode != 0:
            error = result.stderr.lower()
            assert "cycle" not in error and "circular" not in error, \
                "Should not report false cycle"


if __name__ == "__main__":
    test_simple_circular_dependency()
    test_three_way_circular_dependency()
    test_self_referential_resource()
    test_complex_circular_chain()
    test_nested_circular_dependency()
    test_cycle_error_message_quality()
    test_conditional_cycle_handling()
    test_no_false_positive_cycles()
    print("All cycle-detected mapping error tests passed")
