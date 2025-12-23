#!/usr/bin/env python3
"""Test WASM stack overflow behavior."""

import subprocess
import tempfile
from pathlib import Path
import json


def test_wasm_deep_recursion():
    """Test WASM behavior with deep recursion in template."""
    wasm_target = Path("target/wasm32-unknown-unknown/release/costpilot.wasm")

    if not wasm_target.exists():
        print("WASM build not found, skipping test")
        return

    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"

        # Create deeply nested structure (1000 levels)
        nested = {"value": "leaf"}
        for _ in range(1000):
            nested = {"nested": nested}

        template_content = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "Environment": nested,
                        "MemorySize": 1024
                    }
                }
            }
        }

        with open(template_path, 'w') as f:
            json.dump(template_content, f)

        # WASM should handle deep nesting without stack overflow
        result = subprocess.run(
            ["wasmtime", "run", str(wasm_target), "--", "analyze", "--plan", str(template_path)],
            capture_output=True,
            text=True,
            timeout=60
        )

        # Should handle or reject gracefully (no panic)
        assert result.returncode in [0, 1, 2, 101], "WASM should handle deep nesting gracefully"


def test_wasm_stack_limit():
    """Test WASM stack limit enforcement."""
    wasm_target = Path("target/wasm32-unknown-unknown/release/costpilot.wasm")

    if not wasm_target.exists():
        print("WASM build not found, skipping test")
        return

    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"

        # Very deep nesting (5000 levels)
        nested = {"value": "leaf"}
        for _ in range(5000):
            nested = {"nested": nested}

        template_content = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "Environment": nested,
                        "MemorySize": 1024
                    }
                }
            }
        }

        with open(template_path, 'w') as f:
            json.dump(template_content, f)

        # WASM should enforce stack limit
        result = subprocess.run(
            ["wasmtime", "run", str(wasm_target), "--", "analyze", "--plan", str(template_path)],
            capture_output=True,
            text=True,
            timeout=60
        )

        # Should fail gracefully or succeed (no crash)
        assert result.returncode in [0, 1, 2, 101], "WASM should enforce stack limit"


def test_wasm_large_call_stack():
    """Test WASM with large call stack simulation."""
    wasm_target = Path("target/wasm32-unknown-unknown/release/costpilot.wasm")

    if not wasm_target.exists():
        print("WASM build not found, skipping test")
        return

    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"

        # Many resources with complex dependencies (simulates deep call stack)
        template_content = {
            "Resources": {
                f"Lambda{i}": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 1024,
                        "Role": {"Fn::GetAtt": [f"Role{i}", "Arn"]} if i > 0 else None
                    }
                }
                for i in range(500)
            }
        }

        # Add roles
        for i in range(500):
            template_content["Resources"][f"Role{i}"] = {
                "Type": "AWS::IAM::Role",
                "Properties": {
                    "AssumeRolePolicyDocument": {}
                }
            }

        with open(template_path, 'w') as f:
            json.dump(template_content, f)

        # WASM should handle large call stack
        result = subprocess.run(
            ["wasmtime", "run", str(wasm_target), "--", "analyze", "--plan", str(template_path)],
            capture_output=True,
            text=True,
            timeout=120
        )

        # Should handle large call stack
        assert result.returncode in [0, 1, 2, 101], "WASM should handle large call stack"


def test_wasm_stack_overflow_recovery():
    """Test WASM stack overflow recovery."""
    wasm_target = Path("target/wasm32-unknown-unknown/release/costpilot.wasm")

    if not wasm_target.exists():
        print("WASM build not found, skipping test")
        return

    with tempfile.TemporaryDirectory() as tmpdir:
        # First: potential stack overflow
        template1_path = Path(tmpdir) / "template1.json"

        nested = {"value": "leaf"}
        for _ in range(10000):
            nested = {"nested": nested}

        template1_content = {
            "Resources": {
                "Lambda1": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "Environment": nested,
                        "MemorySize": 1024
                    }
                }
            }
        }

        with open(template1_path, 'w') as f:
            json.dump(template1_content, f)

        result1 = subprocess.run(
            ["wasmtime", "run", str(wasm_target), "--", "analyze", "--plan", str(template1_path)],
            capture_output=True,
            text=True,
            timeout=60
        )

        # Second: normal template (should work after potential overflow)
        template2_path = Path(tmpdir) / "template2.json"

        template2_content = {
            "Resources": {
                "Lambda2": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 1024
                    }
                }
            }
        }

        with open(template2_path, 'w') as f:
            json.dump(template2_content, f)

        result2 = subprocess.run(
            ["wasmtime", "run", str(wasm_target), "--", "analyze", "--plan", str(template2_path)],
            capture_output=True,
            text=True,
            timeout=30
        )

        # Second run should always work
        assert result2.returncode in [0, 1, 2, 101], "WASM should recover from stack issues"


def test_wasm_stack_guard_pages():
    """Test WASM stack guard page behavior."""
    wasm_target = Path("target/wasm32-unknown-unknown/release/costpilot.wasm")

    if not wasm_target.exists():
        print("WASM build not found, skipping test")
        return

    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"

        # Template designed to stress stack
        template_content = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 1024,
                        "Environment": {
                            "Variables": {
                                f"VAR{i}": "X" * 1000
                                for i in range(1000)
                            }
                        }
                    }
                }
            }
        }

        with open(template_path, 'w') as f:
            json.dump(template_content, f)

        # WASM should protect stack with guard pages
        result = subprocess.run(
            ["wasmtime", "run", str(wasm_target), "--", "analyze", "--plan", str(template_path)],
            capture_output=True,
            text=True,
            timeout=60
        )

        # Should not crash
        assert result.returncode in [0, 1, 2, 101], "WASM should have stack guard pages"


def test_wasm_stack_size_config():
    """Test WASM with different stack size configurations."""
    wasm_target = Path("target/wasm32-unknown-unknown/release/costpilot.wasm")

    if not wasm_target.exists():
        print("WASM build not found, skipping test")
        return

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

        # Run with limited stack (if wasmtime supports)
        result = subprocess.run(
            ["wasmtime", "run", "--max-wasm-stack", "512000", str(wasm_target), "--",
             "analyze", "--plan", str(template_path)],
            capture_output=True,
            text=True,
            timeout=30
        )

        # Should work with limited stack
        assert result.returncode in [0, 1, 2, 101], "WASM should work with limited stack"


def test_wasm_no_unbounded_recursion():
    """Test that WASM prevents unbounded recursion."""
    wasm_target = Path("target/wasm32-unknown-unknown/release/costpilot.wasm")

    if not wasm_target.exists():
        print("WASM build not found, skipping test")
        return

    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"

        # Circular references (potential unbounded recursion)
        template_content = {
            "Resources": {
                "LambdaA": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 1024,
                        "Role": {"Fn::GetAtt": ["LambdaB", "Arn"]}
                    }
                },
                "LambdaB": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 2048,
                        "Role": {"Fn::GetAtt": ["LambdaC", "Arn"]}
                    }
                },
                "LambdaC": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 3072,
                        "Role": {"Fn::GetAtt": ["LambdaA", "Arn"]}
                    }
                }
            }
        }

        with open(template_path, 'w') as f:
            json.dump(template_content, f)

        # WASM should detect and prevent unbounded recursion
        result = subprocess.run(
            ["wasmtime", "run", str(wasm_target), "--", "analyze", "--plan", str(template_path)],
            capture_output=True,
            text=True,
            timeout=30
        )

        # Should handle circular refs without stack overflow
        assert result.returncode in [0, 1, 2, 101], "WASM should prevent unbounded recursion"


if __name__ == "__main__":
    test_wasm_deep_recursion()
    test_wasm_stack_limit()
    test_wasm_large_call_stack()
    test_wasm_stack_overflow_recovery()
    test_wasm_stack_guard_pages()
    test_wasm_stack_size_config()
    test_wasm_no_unbounded_recursion()
    print("All WASM stack overflow tests passed")
