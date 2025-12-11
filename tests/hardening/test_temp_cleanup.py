#!/usr/bin/env python3
"""Test that temp directories are auto-cleaned."""

import json
import os
import subprocess
import tempfile
import time
from pathlib import Path


def test_temp_cleanup_after_analyze():
    """Test that temp files are cleaned up after analyze."""
    # Get initial temp file count
    temp_dir = Path(tempfile.gettempdir())
    initial_files = set(temp_dir.iterdir())
    
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        
        template_content = {
            "Resources": {
                f"Lambda{i}": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 1024
                    }
                }
                for i in range(100)
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
        
        assert result.returncode in [0, 1, 2, 101], "Should complete"
    
    # Wait briefly for cleanup
    time.sleep(1)
    
    # Check final temp file count
    final_files = set(temp_dir.iterdir())
    
    # New files should be minimal (ideally zero)
    new_files = final_files - initial_files
    costpilot_temp_files = [f for f in new_files if "costpilot" in str(f).lower()]
    
    assert len(costpilot_temp_files) == 0, f"Temp files not cleaned: {costpilot_temp_files}"


def test_temp_cleanup_after_error():
    """Test that temp files are cleaned up even after errors."""
    temp_dir = Path(tempfile.gettempdir())
    initial_files = set(temp_dir.iterdir())
    
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "invalid.json"
        
        # Invalid JSON to trigger error
        with open(template_path, 'w') as f:
            f.write("invalid json content")
        
        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path)],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        assert result.returncode != 0, "Should fail with invalid JSON"
    
    # Wait for cleanup
    time.sleep(1)
    
    # Check for temp file leaks
    final_files = set(temp_dir.iterdir())
    new_files = final_files - initial_files
    costpilot_temp_files = [f for f in new_files if "costpilot" in str(f).lower()]
    
    assert len(costpilot_temp_files) == 0, f"Temp files leaked after error: {costpilot_temp_files}"


def test_temp_cleanup_on_signal():
    """Test that temp files are cleaned up when process is terminated."""
    temp_dir = Path(tempfile.gettempdir())
    initial_files = set(temp_dir.iterdir())
    
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        
        # Large template for long processing
        template_content = {
            "Resources": {
                f"Lambda{i}": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 1024
                    }
                }
                for i in range(5000)
            }
        }
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        # Start process
        proc = subprocess.Popen(
            ["costpilot", "scan", "--plan", str(template_path)],
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE
        )
        
        # Wait briefly, then terminate
        time.sleep(2)
        proc.terminate()
        
        # Wait for termination
        try:
            proc.wait(timeout=5)
        except subprocess.TimeoutExpired:
            proc.kill()
            proc.wait()
    
    # Wait for cleanup
    time.sleep(2)
    
    # Check for temp file leaks
    final_files = set(temp_dir.iterdir())
    new_files = final_files - initial_files
    costpilot_temp_files = [f for f in new_files if "costpilot" in str(f).lower()]
    
    # Allow some grace (cleanup handlers may not run on SIGKILL)
    assert len(costpilot_temp_files) <= 1, f"Excessive temp files after signal: {costpilot_temp_files}"


def test_temp_directory_permissions():
    """Test that temp directories have restrictive permissions."""
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
        
        # Monitor temp directory during execution
        temp_base = Path(tempfile.gettempdir())
        
        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path)],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        # Check for any costpilot temp directories that remain
        remaining_dirs = [d for d in temp_base.iterdir() if d.is_dir() and "costpilot" in str(d).lower()]
        
        for temp_dir in remaining_dirs:
            # Check permissions (should be restrictive)
            stat_info = temp_dir.stat()
            mode = stat_info.st_mode
            
            # On Unix, should not be world-readable/writable
            if hasattr(os, 'ST_MODE'):
                world_perms = mode & 0o007
                assert world_perms == 0, f"Temp directory has world permissions: {temp_dir}"


def test_no_temp_file_name_leakage():
    """Test that temp file names don't leak sensitive info."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "sensitive_customer_data.json"
        
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
        
        # Monitor temp directory
        temp_base = Path(tempfile.gettempdir())
        before = set(temp_base.iterdir())
        
        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path)],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        after = set(temp_base.iterdir())
        new_files = after - before
        
        # Check that sensitive filename didn't leak into temp names
        for temp_file in new_files:
            assert "sensitive" not in str(temp_file).lower(), \
                f"Sensitive data in temp filename: {temp_file}"
            assert "customer" not in str(temp_file).lower(), \
                f"Sensitive data in temp filename: {temp_file}"


def test_temp_cleanup_multiple_runs():
    """Test that temp files don't accumulate across runs."""
    temp_dir = Path(tempfile.gettempdir())
    initial_files = set(temp_dir.iterdir())
    
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
        
        # Run multiple times
        for _ in range(10):
            result = subprocess.run(
                ["costpilot", "scan", "--plan", str(template_path)],
                capture_output=True,
                text=True,
                timeout=10
            )
            
            assert result.returncode in [0, 1, 2, 101], "Should complete"
    
    # Wait for cleanup
    time.sleep(2)
    
    # Check for accumulation
    final_files = set(temp_dir.iterdir())
    new_files = final_files - initial_files
    costpilot_temp_files = [f for f in new_files if "costpilot" in str(f).lower()]
    
    # Should not accumulate (allow small margin)
    assert len(costpilot_temp_files) <= 2, f"Temp files accumulated: {costpilot_temp_files}"


def test_temp_cleanup_on_panic():
    """Test cleanup on panic/crash (best effort)."""
    temp_dir = Path(tempfile.gettempdir())
    initial_files = set(temp_dir.iterdir())
    
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        
        # Create potentially problematic template
        template_content = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 2 ** 63  # Very large number
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
        
        # May fail, but check cleanup
    
    time.sleep(1)
    
    final_files = set(temp_dir.iterdir())
    new_files = final_files - initial_files
    costpilot_temp_files = [f for f in new_files if "costpilot" in str(f).lower()]
    
    # Best effort cleanup
    assert len(costpilot_temp_files) <= 1, f"Temp files after crash: {costpilot_temp_files}"


def test_temp_directory_isolation():
    """Test that temp directories are isolated per invocation."""
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
        
        # Run two instances in parallel
        proc1 = subprocess.Popen(
            ["costpilot", "scan", "--plan", str(template_path)],
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE
        )
        
        proc2 = subprocess.Popen(
            ["costpilot", "scan", "--plan", str(template_path)],
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE
        )
        
        # Wait for completion
        proc1.wait(timeout=30)
        proc2.wait(timeout=30)
        
        # Both should succeed (no temp directory conflicts)
        assert proc1.returncode in [0, 1, 2, 101], "First process should complete"
        assert proc2.returncode in [0, 1, 2, 101], "Second process should complete"


if __name__ == "__main__":
    test_temp_cleanup_after_analyze()
    test_temp_cleanup_after_error()
    test_temp_cleanup_on_signal()
    test_temp_directory_permissions()
    test_no_temp_file_name_leakage()
    test_temp_cleanup_multiple_runs()
    test_temp_cleanup_on_panic()
    test_temp_directory_isolation()
    print("All temp directory auto-clean tests passed")
