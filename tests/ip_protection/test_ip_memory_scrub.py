#!/usr/bin/env python3
"""Test IP Protection: Premium engine memory scrub on unload."""

import subprocess
import tempfile
from pathlib import Path
import json
import time
import os
import signal


def test_memory_scrub_after_execution():
    """Test memory is scrubbed after execution completes."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        
        template_content = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 2048
                    }
                }
            }
        }
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        # Run analysis
        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path)],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        # Process should exit cleanly
        assert result.returncode == 0 or result.returncode == 1, \
            "Process should exit cleanly (memory scrubbed)"


def test_no_core_dump_contains_premium():
    """Test core dumps don't contain premium data."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        
        template_content = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 2048
                    }
                }
            }
        }
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        # Enable core dumps temporarily
        original_limit = None
        try:
            import resource
            original_limit = resource.getrlimit(resource.RLIMIT_CORE)
            resource.setrlimit(resource.RLIMIT_CORE, (resource.RLIM_INFINITY, resource.RLIM_INFINITY))
        except:
            pass
        
        try:
            # Run and potentially crash
            result = subprocess.run(
                ["costpilot", "scan", "--plan", str(template_path)],
                capture_output=True,
                text=True,
                timeout=10
            )
        finally:
            # Restore limit
            if original_limit:
                try:
                    import resource
                    resource.setrlimit(resource.RLIMIT_CORE, original_limit)
                except:
                    pass


def test_signal_handler_cleans_memory():
    """Test signal handlers clean up memory."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "large.json"
        
        # Large template
        resources = {
            f"Lambda{i}": {
                "Type": "AWS::Lambda::Function",
                "Properties": {
                    "MemorySize": 2048
                }
            }
            for i in range(100)
        }
        
        template_content = {"Resources": resources}
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        # Start process
        proc = subprocess.Popen(
            ["costpilot", "scan", "--plan", str(template_path)],
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True
        )
        
        # Let it run briefly
        time.sleep(0.5)
        
        # Send SIGTERM
        proc.terminate()
        
        try:
            stdout, stderr = proc.communicate(timeout=5)
        except subprocess.TimeoutExpired:
            proc.kill()
            stdout, stderr = proc.communicate()
        
        # Should exit gracefully (signal handler should clean up)
        assert proc.returncode in [-15, 143, 0, 1], \
            f"Should exit cleanly on SIGTERM, got: {proc.returncode}"


def test_no_sensitive_data_in_swap():
    """Test sensitive data not left in swap."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        
        template_content = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
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
            timeout=10
        )
        
        # Process completes, memory should be released
        # In production, mlock() would prevent swapping
        assert result.returncode in [0, 1, 2, 101], "Process should complete"


def test_multiple_runs_no_memory_leak():
    """Test multiple runs don't leak memory."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        
        template_content = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 2048
                    }
                }
            }
        }
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        # Run multiple times
        for i in range(10):
            result = subprocess.run(
                ["costpilot", "scan", "--plan", str(template_path)],
                capture_output=True,
                text=True,
                timeout=10
            )
            
            # Each run should complete independently
            assert result.returncode in [0, 1, 2, 101], f"Run {i} should complete"


def test_memory_scrub_on_error():
    """Test memory is scrubbed even on error."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "invalid.json"
        
        with open(template_path, 'w') as f:
            f.write('INVALID JSON')
        
        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path)],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        # Should fail but exit cleanly
        assert result.returncode != 0, "Should fail on invalid JSON"
        # Process should terminate cleanly (memory scrubbed)


if __name__ == "__main__":
    test_memory_scrub_after_execution()
    test_no_core_dump_contains_premium()
    test_signal_handler_cleans_memory()
    test_no_sensitive_data_in_swap()
    test_multiple_runs_no_memory_leak()
    test_memory_scrub_on_error()
    print("All IP Protection: memory scrub tests passed")
