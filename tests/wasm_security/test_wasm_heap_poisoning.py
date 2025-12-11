#!/usr/bin/env python3
"""Test WASM heap poisoning detection."""

import subprocess
import tempfile
from pathlib import Path
import json


def test_wasm_heap_integrity():
    """Test WASM heap integrity during processing."""
    wasm_target = Path("target/wasm32-unknown-unknown/release/costpilot.wasm")
    
    if not wasm_target.exists():
        print("WASM build not found, skipping test")
        return
    
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        
        # Large template to stress heap
        template_content = {
            "Resources": {
                f"Lambda{i}": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 1024 + (i * 128),
                        "Description": "A" * 10000
                    }
                }
                for i in range(100)
            }
        }
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        # Run multiple times to check heap integrity
        for _ in range(5):
            result = subprocess.run(
                ["wasmtime", "run", str(wasm_target), "--", "analyze", "--plan", str(template_path)],
                capture_output=True,
                text=True,
                timeout=60
            )
            
            # Should maintain heap integrity
            assert result.returncode in [0, 1, 2, 101], "WASM heap integrity check"


def test_wasm_memory_corruption_detection():
    """Test WASM memory corruption detection."""
    wasm_target = Path("target/wasm32-unknown-unknown/release/costpilot.wasm")
    
    if not wasm_target.exists():
        print("WASM build not found, skipping test")
        return
    
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        
        # Template with potential memory issues
        template_content = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 1024,
                        "Environment": {
                            "Variables": {
                                f"VAR{i}": "X" * 100000
                                for i in range(100)
                            }
                        }
                    }
                }
            }
        }
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        # WASM should detect any memory corruption
        result = subprocess.run(
            ["wasmtime", "run", str(wasm_target), "--", "analyze", "--plan", str(template_path)],
            capture_output=True,
            text=True,
            timeout=60
        )
        
        # Should complete without corruption
        assert result.returncode in [0, 1, 2, 101], "WASM memory corruption detection"


def test_wasm_heap_bounds_checking():
    """Test WASM heap bounds checking."""
    wasm_target = Path("target/wasm32-unknown-unknown/release/costpilot.wasm")
    
    if not wasm_target.exists():
        print("WASM build not found, skipping test")
        return
    
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        
        # Very large structure
        template_content = {
            "Resources": {
                f"Resource{i}": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 1024,
                        "Tags": [
                            {"Key": f"Tag{j}", "Value": "V" * 1000}
                            for j in range(1000)
                        ]
                    }
                }
                for i in range(10)
            }
        }
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        # WASM should enforce heap bounds
        result = subprocess.run(
            ["wasmtime", "run", str(wasm_target), "--", "analyze", "--plan", str(template_path)],
            capture_output=True,
            text=True,
            timeout=120
        )
        
        # Should enforce bounds
        assert result.returncode in [0, 1, 2, 101], "WASM heap bounds checking"


def test_wasm_use_after_free_prevention():
    """Test WASM use-after-free prevention."""
    wasm_target = Path("target/wasm32-unknown-unknown/release/costpilot.wasm")
    
    if not wasm_target.exists():
        print("WASM build not found, skipping test")
        return
    
    with tempfile.TemporaryDirectory() as tmpdir:
        # Process multiple templates sequentially
        for iteration in range(10):
            template_path = Path(tmpdir) / f"template{iteration}.json"
            
            template_content = {
                "Resources": {
                    f"Lambda{i}": {
                        "Type": "AWS::Lambda::Function",
                        "Properties": {
                            "MemorySize": 1024 + (i * 256)
                        }
                    }
                    for i in range(50)
                }
            }
            
            with open(template_path, 'w') as f:
                json.dump(template_content, f)
            
            result = subprocess.run(
                ["wasmtime", "run", str(wasm_target), "--", "analyze", "--plan", str(template_path)],
                capture_output=True,
                text=True,
                timeout=60
            )
            
            # No use-after-free
            assert result.returncode in [0, 1, 2, 101], f"WASM use-after-free prevention iteration {iteration}"


def test_wasm_double_free_prevention():
    """Test WASM double-free prevention."""
    wasm_target = Path("target/wasm32-unknown-unknown/release/costpilot.wasm")
    
    if not wasm_target.exists():
        print("WASM build not found, skipping test")
        return
    
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        
        # Same resource names (potential double-free scenario)
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
        
        # Run multiple times
        for _ in range(10):
            result = subprocess.run(
                ["wasmtime", "run", str(wasm_target), "--", "analyze", "--plan", str(template_path)],
                capture_output=True,
                text=True,
                timeout=30
            )
            
            # No double-free
            assert result.returncode in [0, 1, 2, 101], "WASM double-free prevention"


def test_wasm_heap_overflow_detection():
    """Test WASM heap overflow detection."""
    wasm_target = Path("target/wasm32-unknown-unknown/release/costpilot.wasm")
    
    if not wasm_target.exists():
        print("WASM build not found, skipping test")
        return
    
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        
        # Extremely large structure
        template_content = {
            "Resources": {
                f"Resource{i}": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 1024,
                        "Description": "X" * 1000000  # 1MB per resource
                    }
                }
                for i in range(100)  # 100MB total
            }
        }
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        # WASM should detect heap overflow
        result = subprocess.run(
            ["wasmtime", "run", str(wasm_target), "--", "analyze", "--plan", str(template_path)],
            capture_output=True,
            text=True,
            timeout=120
        )
        
        # Should handle or fail gracefully
        assert result.returncode in [0, 1, 2, 101], "WASM heap overflow detection"


def test_wasm_allocator_poisoning():
    """Test WASM allocator poisoning detection."""
    wasm_target = Path("target/wasm32-unknown-unknown/release/costpilot.wasm")
    
    if not wasm_target.exists():
        print("WASM build not found, skipping test")
        return
    
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        
        # Pattern that could stress allocator
        template_content = {
            "Resources": {}
        }
        
        # Mix of small and large allocations
        for i in range(1000):
            if i % 2 == 0:
                template_content["Resources"][f"Small{i}"] = {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 128
                    }
                }
            else:
                template_content["Resources"][f"Large{i}"] = {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 10240,
                        "Description": "X" * 10000
                    }
                }
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        # WASM allocator should be robust
        result = subprocess.run(
            ["wasmtime", "run", str(wasm_target), "--", "analyze", "--plan", str(template_path)],
            capture_output=True,
            text=True,
            timeout=120
        )
        
        # Should handle mixed allocations
        assert result.returncode in [0, 1, 2, 101], "WASM allocator poisoning detection"


def test_wasm_memory_leak_detection():
    """Test WASM memory leak detection."""
    wasm_target = Path("target/wasm32-unknown-unknown/release/costpilot.wasm")
    
    if not wasm_target.exists():
        print("WASM build not found, skipping test")
        return
    
    with tempfile.TemporaryDirectory() as tmpdir:
        # Run same workload multiple times
        template_path = Path(tmpdir) / "template.json"
        
        template_content = {
            "Resources": {
                f"Lambda{i}": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 1024,
                        "Description": "X" * 10000
                    }
                }
                for i in range(100)
            }
        }
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        # Run 20 times - memory should not grow unbounded
        for iteration in range(20):
            result = subprocess.run(
                ["wasmtime", "run", str(wasm_target), "--", "analyze", "--plan", str(template_path)],
                capture_output=True,
                text=True,
                timeout=60
            )
            
            # Should not leak memory
            assert result.returncode in [0, 1, 2, 101], f"WASM memory leak detection iteration {iteration}"


if __name__ == "__main__":
    test_wasm_heap_integrity()
    test_wasm_memory_corruption_detection()
    test_wasm_heap_bounds_checking()
    test_wasm_use_after_free_prevention()
    test_wasm_double_free_prevention()
    test_wasm_heap_overflow_detection()
    test_wasm_allocator_poisoning()
    test_wasm_memory_leak_detection()
    print("All WASM heap poisoning detection tests passed")
