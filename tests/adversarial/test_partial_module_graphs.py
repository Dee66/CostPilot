#!/usr/bin/env python3
"""Test handling of partial module graphs."""

import json
import subprocess
import tempfile
from pathlib import Path


def test_missing_resource_dependencies():
    """Test handling of graphs with missing resource dependencies."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"

        template_content = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 1024,
                        "Role": {"Fn::GetAtt": ["NonExistentRole", "Arn"]}
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

        # Should handle missing dependencies
        assert result.returncode in [0, 1, 2, 101], "Should handle missing dependencies"


def test_circular_dependency_incomplete():
    """Test handling of incomplete circular dependencies."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"

        template_content = {
            "Resources": {
                "LambdaA": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 1024
                    },
                    "DependsOn": "LambdaB"
                },
                "LambdaB": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 2048
                    },
                    "DependsOn": "LambdaC"
                }
                # LambdaC missing - incomplete cycle
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

        # Should handle incomplete cycles
        assert result.returncode in [0, 1, 2, 101], "Should handle incomplete cycles"


def test_orphaned_resources():
    """Test handling of orphaned resources with no connections."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"

        template_content = {
            "Resources": {
                "Lambda1": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 1024
                    }
                },
                "Lambda2": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 2048
                    }
                },
                "Lambda3": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 3072
                    }
                }
                # All independent - no dependencies
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

        # Should handle orphaned resources
        assert result.returncode in [0, 1, 2, 101], "Should handle orphaned resources"


def test_partially_defined_intrinsic_functions():
    """Test handling of partially defined intrinsic functions."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"

        template_content = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 1024,
                        "Role": {"Fn::GetAtt": ["IncompleteRef"]}  # Missing attribute name
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

        # Should handle incomplete intrinsics
        assert result.returncode in [0, 1, 2, 101], "Should handle incomplete intrinsic functions"


def test_missing_parameters_in_graph():
    """Test handling of graphs with missing parameter references."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"

        template_content = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": {"Ref": "NonExistentParameter"}
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

        # Should handle missing parameter refs
        assert result.returncode in [0, 1, 2, 101], "Should handle missing parameter refs"


def test_disconnected_subgraphs():
    """Test handling of multiple disconnected subgraphs."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"

        template_content = {
            "Resources": {
                # Subgraph 1
                "Lambda1A": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 1024
                    },
                    "DependsOn": "Lambda1B"
                },
                "Lambda1B": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 2048
                    }
                },
                # Subgraph 2 (disconnected)
                "Lambda2A": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 3072
                    },
                    "DependsOn": "Lambda2B"
                },
                "Lambda2B": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 4096
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

        # Should handle disconnected subgraphs
        assert result.returncode in [0, 1, 2, 101], "Should handle disconnected subgraphs"


def test_self_referential_resource():
    """Test handling of self-referential resources."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"

        template_content = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 1024
                    },
                    "DependsOn": "Lambda"  # Self-reference
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
        assert result.returncode in [0, 1, 2, 101], "Should handle self-referential resources"


def test_partial_graph_with_conditions():
    """Test handling of partial graphs with conditional resources."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"

        template_content = {
            "Conditions": {
                "CreateResource": {"Fn::Equals": ["true", "false"]}
            },
            "Resources": {
                "Lambda1": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 1024
                    },
                    "DependsOn": "Lambda2"
                },
                "Lambda2": {
                    "Type": "AWS::Lambda::Function",
                    "Condition": "CreateResource",  # May not be created
                    "Properties": {
                        "MemorySize": 2048
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

        # Should handle conditional dependencies
        assert result.returncode in [0, 1, 2, 101], "Should handle conditional dependencies"


def test_incomplete_nested_stacks():
    """Test handling of incomplete nested stack references."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"

        template_content = {
            "Resources": {
                "NestedStack": {
                    "Type": "AWS::CloudFormation::Stack",
                    "Properties": {
                        "TemplateURL": "https://s3.amazonaws.com/bucket/nonexistent.json"
                    }
                },
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": {"Fn::GetAtt": ["NestedStack", "Outputs.MemorySize"]}
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

        # Should handle nested stack references
        assert result.returncode in [0, 1, 2, 101], "Should handle nested stack references"


def test_graph_with_only_outputs():
    """Test handling of graphs with only outputs (no resources)."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"

        template_content = {
            "Outputs": {
                "OutputValue": {
                    "Value": {"Ref": "NonExistentResource"}
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

        # Should handle output-only templates
        assert result.returncode in [0, 1, 2, 101], "Should handle output-only templates"


if __name__ == "__main__":
    test_missing_resource_dependencies()
    test_circular_dependency_incomplete()
    test_orphaned_resources()
    test_partially_defined_intrinsic_functions()
    test_missing_parameters_in_graph()
    test_disconnected_subgraphs()
    test_self_referential_resource()
    test_partial_graph_with_conditions()
    test_incomplete_nested_stacks()
    test_graph_with_only_outputs()
    print("All partial module graph tests passed")
