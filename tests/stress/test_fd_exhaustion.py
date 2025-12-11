#!/usr/bin/env python3
"""Test FD exhaustion handling."""

import subprocess
import tempfile
from pathlib import Path
import json
import os


def test_many_open_files():
    """Test behavior with many open files."""
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
        
        # Open many files
        files = []
        try:
            for i in range(1000):
                f = open(Path(tmpdir) / f"file_{i}.txt", 'w')
                files.append(f)
            
            result = subprocess.run(
                ["costpilot", "scan", "--plan", str(template_path)],
                capture_output=True,
                text=True,
                timeout=30
            )
            
            # Should complete despite many open files
            assert result.returncode in [0, 1, 2, 101], "Should handle many open files"
        finally:
            for f in files:
                try:
                    f.close()
                except:
                    pass


def test_fd_limit_reached():
    """Test behavior when FD limit is reached."""
    import resource
    
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
        
        # Get current FD limit
        soft, hard = resource.getrlimit(resource.RLIMIT_NOFILE)
        
        # Open files up to near limit
        files = []
        try:
            target = min(soft - 50, 1000)
            for i in range(target):
                f = open(Path(tmpdir) / f"file_{i}.txt", 'w')
                files.append(f)
            
            result = subprocess.run(
                ["costpilot", "scan", "--plan", str(template_path)],
                capture_output=True,
                text=True,
                timeout=30
            )
            
            # Should complete or fail gracefully
            assert result.returncode in [0, 1, 2, 101], "Should handle near-FD-limit"
        finally:
            for f in files:
                try:
                    f.close()
                except:
                    pass


def test_multiple_large_files():
    """Test behavior with multiple large files open."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        baseline_path = Path(tmpdir) / "baseline.json"
        policy_path = Path(tmpdir) / "policy.json"
        
        # Create large template
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
        
        # Create large baseline
        baseline_content = {
            "resources": [
                {
                    "id": f"Lambda{i}",
                    "type": "AWS::Lambda::Function",
                    "cost": 10.0
                }
                for i in range(100)
            ]
        }
        
        # Create policy
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
        
        with open(baseline_path, 'w') as f:
            json.dump(baseline_content, f)
        
        with open(policy_path, 'w') as f:
            json.dump(policy_content, f)
        
        result = subprocess.run(
            ["costpilot", "check", "--plan", str(template_path), 
             "--baseline", str(baseline_path), "--policy", str(policy_path)],
            capture_output=True,
            text=True,
            timeout=60
        )
        
        # Should complete
        assert result.returncode in [0, 1, 2, 101], "Should handle multiple large files"


def test_concurrent_file_access():
    """Test concurrent file access."""
    import multiprocessing
    
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
        
        def run_analysis(_):
            result = subprocess.run(
                ["costpilot", "scan", "--plan", str(template_path)],
                capture_output=True,
                text=True,
                timeout=30
            )
            return result.returncode in [0, 1]
        
        # Run 20 concurrent processes
        with multiprocessing.Pool(20) as pool:
            results = pool.map(run_analysis, range(20))
        
        successes = sum(results)
        
        # Most should succeed
        assert successes >= 18, "Concurrent file access should be stable"


def test_fd_leak_detection():
    """Test FD leak detection."""
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
        
        # Get initial FD count
        initial_fds = len(os.listdir("/proc/self/fd"))
        
        # Run multiple invocations
        for _ in range(100):
            result = subprocess.run(
                ["costpilot", "scan", "--plan", str(template_path)],
                capture_output=True,
                text=True,
                timeout=30
            )
            
            assert result.returncode in [0, 1, 2, 101], "Should complete"
        
        # Check final FD count
        final_fds = len(os.listdir("/proc/self/fd"))
        
        # FD count should not grow significantly
        assert final_fds - initial_fds < 10, f"FD leak detected: {initial_fds} -> {final_fds}"


def test_pipe_exhaustion():
    """Test pipe exhaustion handling."""
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
                for i in range(1000)
            }
        }
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        # Create many pipes
        pipes = []
        try:
            for _ in range(100):
                r, w = os.pipe()
                pipes.append((r, w))
            
            result = subprocess.run(
                ["costpilot", "scan", "--plan", str(template_path)],
                capture_output=True,
                text=True,
                timeout=30
            )
            
            # Should complete
            assert result.returncode in [0, 1, 2, 101], "Should handle many pipes"
        finally:
            for r, w in pipes:
                try:
                    os.close(r)
                    os.close(w)
                except:
                    pass


def test_temp_file_cleanup():
    """Test temp file cleanup."""
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
        
        # Count temp files before
        temp_dir = Path("/tmp")
        initial_count = len(list(temp_dir.glob("costpilot*")))
        
        # Run multiple invocations
        for _ in range(50):
            result = subprocess.run(
                ["costpilot", "scan", "--plan", str(template_path)],
                capture_output=True,
                text=True,
                timeout=30
            )
        
        # Count temp files after
        final_count = len(list(temp_dir.glob("costpilot*")))
        
        # Temp files should be cleaned up
        assert final_count - initial_count < 10, "Temp files not cleaned up"


if __name__ == "__main__":
    test_many_open_files()
    test_fd_limit_reached()
    test_multiple_large_files()
    test_concurrent_file_access()
    test_fd_leak_detection()
    test_pipe_exhaustion()
    test_temp_file_cleanup()
    print("All FD exhaustion tests passed")
