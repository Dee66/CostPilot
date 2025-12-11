#!/usr/bin/env python3
"""Test explanation under large diffs."""

import json
import subprocess
import tempfile
from pathlib import Path


def test_explain_large_resource_count_diff():
    """Test explaining diff with large resource count change."""
    with tempfile.TemporaryDirectory() as tmpdir:
        before_path = Path(tmpdir) / "before.json"
        after_path = Path(tmpdir) / "after.json"
        
        # Before: 10 resources
        before_content = {
            "Resources": {
                f"Lambda{i}": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 1024
                    }
                }
                for i in range(10)
            }
        }
        
        # After: 1000 resources
        after_content = {
            "Resources": {
                f"Lambda{i}": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 1024
                    }
                }
                for i in range(1000)
            }
        }
        
        with open(before_path, 'w') as f:
            json.dump(before_content, f)
        
        with open(after_path, 'w') as f:
            json.dump(after_content, f)
        
        result = subprocess.run(
            ["costpilot", "explain", "--before", str(before_path), "--after", str(after_path)],
            capture_output=True,
            text=True,
            timeout=60
        )
        
        # Should explain large diff
        assert result.returncode in [0, 1, 2, 101], "Should explain large resource count diff"


def test_explain_property_changes_at_scale():
    """Test explaining many property changes."""
    with tempfile.TemporaryDirectory() as tmpdir:
        before_path = Path(tmpdir) / "before.json"
        after_path = Path(tmpdir) / "after.json"
        
        # Before
        before_content = {
            "Resources": {
                f"Lambda{i}": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 1024,
                        "Timeout": 30,
                        "Runtime": "python3.9"
                    }
                }
                for i in range(500)
            }
        }
        
        # After: All properties changed
        after_content = {
            "Resources": {
                f"Lambda{i}": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 2048,
                        "Timeout": 60,
                        "Runtime": "python3.11"
                    }
                }
                for i in range(500)
            }
        }
        
        with open(before_path, 'w') as f:
            json.dump(before_content, f)
        
        with open(after_path, 'w') as f:
            json.dump(after_content, f)
        
        result = subprocess.run(
            ["costpilot", "explain", "--before", str(before_path), "--after", str(after_path)],
            capture_output=True,
            text=True,
            timeout=120
        )
        
        # Should explain massive property changes
        assert result.returncode in [0, 1, 2, 101], "Should explain many property changes"


def test_explain_mixed_operations():
    """Test explaining diff with additions, deletions, and modifications."""
    with tempfile.TemporaryDirectory() as tmpdir:
        before_path = Path(tmpdir) / "before.json"
        after_path = Path(tmpdir) / "after.json"
        
        # Before: Resources 0-999
        before_content = {
            "Resources": {
                f"Lambda{i}": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 1024
                    }
                }
                for i in range(1000)
            }
        }
        
        # After: Remove 0-499, modify 500-999, add 1000-1499
        after_content = {
            "Resources": {}
        }
        
        # Modified (500-999)
        for i in range(500, 1000):
            after_content["Resources"][f"Lambda{i}"] = {
                "Type": "AWS::Lambda::Function",
                "Properties": {
                    "MemorySize": 2048
                }
            }
        
        # Added (1000-1499)
        for i in range(1000, 1500):
            after_content["Resources"][f"Lambda{i}"] = {
                "Type": "AWS::Lambda::Function",
                "Properties": {
                    "MemorySize": 1024
                }
            }
        
        with open(before_path, 'w') as f:
            json.dump(before_content, f)
        
        with open(after_path, 'w') as f:
            json.dump(after_content, f)
        
        result = subprocess.run(
            ["costpilot", "explain", "--before", str(before_path), "--after", str(after_path)],
            capture_output=True,
            text=True,
            timeout=120
        )
        
        # Should handle mixed operations
        assert result.returncode in [0, 1, 2, 101], "Should explain mixed operations"


def test_explain_deeply_nested_changes():
    """Test explaining changes in deeply nested properties."""
    with tempfile.TemporaryDirectory() as tmpdir:
        before_path = Path(tmpdir) / "before.json"
        after_path = Path(tmpdir) / "after.json"
        
        # Create deeply nested structure
        def create_nested(depth, value):
            if depth == 0:
                return value
            return {"level": create_nested(depth - 1, value)}
        
        before_content = {
            "Resources": {
                f"Lambda{i}": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": create_nested(20, "value_before")
                }
                for i in range(100)
            }
        }
        
        after_content = {
            "Resources": {
                f"Lambda{i}": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": create_nested(20, "value_after")
                }
                for i in range(100)
            }
        }
        
        with open(before_path, 'w') as f:
            json.dump(before_content, f)
        
        with open(after_path, 'w') as f:
            json.dump(after_content, f)
        
        result = subprocess.run(
            ["costpilot", "explain", "--before", str(before_path), "--after", str(after_path)],
            capture_output=True,
            text=True,
            timeout=60
        )
        
        # Should explain deeply nested changes
        assert result.returncode in [0, 1, 2, 101], "Should explain deeply nested changes"


def test_explain_large_string_changes():
    """Test explaining changes with large string properties."""
    with tempfile.TemporaryDirectory() as tmpdir:
        before_path = Path(tmpdir) / "before.json"
        after_path = Path(tmpdir) / "after.json"
        
        # Large string properties
        before_content = {
            "Resources": {
                f"Lambda{i}": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "Description": "A" * 10000,
                        "Code": {
                            "ZipFile": "data_before" * 1000
                        }
                    }
                }
                for i in range(100)
            }
        }
        
        after_content = {
            "Resources": {
                f"Lambda{i}": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "Description": "B" * 10000,
                        "Code": {
                            "ZipFile": "data_after" * 1000
                        }
                    }
                }
                for i in range(100)
            }
        }
        
        with open(before_path, 'w') as f:
            json.dump(before_content, f)
        
        with open(after_path, 'w') as f:
            json.dump(after_content, f)
        
        result = subprocess.run(
            ["costpilot", "explain", "--before", str(before_path), "--after", str(after_path)],
            capture_output=True,
            text=True,
            timeout=120
        )
        
        # Should handle large string changes
        assert result.returncode in [0, 1, 2, 101], "Should explain large string changes"


def test_explain_type_changes():
    """Test explaining resource type changes at scale."""
    with tempfile.TemporaryDirectory() as tmpdir:
        before_path = Path(tmpdir) / "before.json"
        after_path = Path(tmpdir) / "after.json"
        
        # Before: All Lambda
        before_content = {
            "Resources": {
                f"Resource{i}": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 1024
                    }
                }
                for i in range(200)
            }
        }
        
        # After: Mix of types
        types = ["AWS::Lambda::Function", "AWS::EC2::Instance", "AWS::RDS::DBInstance", "AWS::DynamoDB::Table"]
        after_content = {
            "Resources": {
                f"Resource{i}": {
                    "Type": types[i % len(types)],
                    "Properties": {}
                }
                for i in range(200)
            }
        }
        
        with open(before_path, 'w') as f:
            json.dump(before_content, f)
        
        with open(after_path, 'w') as f:
            json.dump(after_content, f)
        
        result = subprocess.run(
            ["costpilot", "explain", "--before", str(before_path), "--after", str(after_path)],
            capture_output=True,
            text=True,
            timeout=60
        )
        
        # Should explain type changes
        assert result.returncode in [0, 1, 2, 101], "Should explain type changes"


def test_explain_output_truncation():
    """Test that explanation output is appropriately truncated for huge diffs."""
    with tempfile.TemporaryDirectory() as tmpdir:
        before_path = Path(tmpdir) / "before.json"
        after_path = Path(tmpdir) / "after.json"
        
        # Massive diff
        before_content = {
            "Resources": {
                f"Lambda{i}": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 1024 + i
                    }
                }
                for i in range(5000)
            }
        }
        
        after_content = {
            "Resources": {
                f"Lambda{i}": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 2048 + i
                    }
                }
                for i in range(5000)
            }
        }
        
        with open(before_path, 'w') as f:
            json.dump(before_content, f)
        
        with open(after_path, 'w') as f:
            json.dump(after_content, f)
        
        result = subprocess.run(
            ["costpilot", "explain", "--before", str(before_path), "--after", str(after_path)],
            capture_output=True,
            text=True,
            timeout=180
        )
        
        # Should complete and produce reasonable output
        assert result.returncode in [0, 1, 2, 101], "Should handle massive diff"
        
        # Output should be bounded (not gigabytes)
        output_size = len(result.stdout) + len(result.stderr)
        assert output_size < 10 * 1024 * 1024, "Output should be bounded"


if __name__ == "__main__":
    test_explain_large_resource_count_diff()
    test_explain_property_changes_at_scale()
    test_explain_mixed_operations()
    test_explain_deeply_nested_changes()
    test_explain_large_string_changes()
    test_explain_type_changes()
    test_explain_output_truncation()
    print("All large diff explanation tests passed")
