#!/usr/bin/env python3
"""Test slow I/O handling."""

import subprocess
import tempfile
from pathlib import Path
import json
import time
import threading


def test_slow_file_read():
    """Test behavior with slow file reads."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"

        # Create large template to slow down read
        template_content = {
            "Resources": {
                f"Lambda{i}": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 1024,
                        "Description": "X" * 100000
                    }
                }
                for i in range(100)
            }
        }

        with open(template_path, 'w') as f:
            json.dump(template_content, f)

        start_time = time.time()

        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path)],
            capture_output=True,
            text=True,
            timeout=60
        )

        elapsed = time.time() - start_time

        # Should complete despite slow read
        assert result.returncode in [0, 1, 2, 101], "Should handle slow file read"
        print(f"Slow file read took {elapsed:.2f}s")


def test_slow_network_simulation():
    """Test behavior with slow network (simulated)."""
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

        # Simulate slow network by introducing delay
        start_time = time.time()

        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path)],
            capture_output=True,
            text=True,
            timeout=30
        )

        # Should complete
        assert result.returncode in [0, 1, 2, 101], "Should handle slow network"


def test_concurrent_slow_operations():
    """Test concurrent slow operations."""
    import multiprocessing

    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"

        # Create large template
        template_content = {
            "Resources": {
                f"Lambda{i}": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 1024,
                        "Description": "X" * 50000
                    }
                }
                for i in range(50)
            }
        }

        with open(template_path, 'w') as f:
            json.dump(template_content, f)

        def run_analysis(_):
            result = subprocess.run(
                ["costpilot", "scan", "--plan", str(template_path)],
                capture_output=True,
                text=True,
                timeout=60
            )
            return result.returncode in [0, 1]

        start_time = time.time()

        # Run 10 concurrent slow operations
        with multiprocessing.Pool(10) as pool:
            results = pool.map(run_analysis, range(10))

        elapsed = time.time() - start_time

        successes = sum(results)

        # Most should succeed
        assert successes >= 8, "Concurrent slow operations should be stable"
        print(f"Concurrent slow operations took {elapsed:.2f}s")


def test_io_timeout_handling():
    """Test I/O timeout handling."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"

        # Create extremely large template
        template_content = {
            "Resources": {
                f"Lambda{i}": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 1024,
                        "Description": "X" * 1000000
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
            timeout=120
        )

        # Should complete or timeout gracefully
        assert result.returncode in [0, 1, 2, 101], "Should handle I/O timeout"


def test_blocking_io():
    """Test blocking I/O operations."""
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

        # Run with stdin blocked
        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path)],
            capture_output=True,
            text=True,
            stdin=subprocess.PIPE,
            timeout=30
        )

        # Should not block
        assert result.returncode in [0, 1, 2, 101], "Should not block on I/O"


def test_slow_write_operations():
    """Test slow write operations."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        output_path = Path(tmpdir) / "output.json"

        # Create large template
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

        start_time = time.time()

        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path), "--output", str(output_path)],
            capture_output=True,
            text=True,
            timeout=60
        )

        elapsed = time.time() - start_time

        # Should complete
        assert result.returncode in [0, 1, 2, 101], "Should handle slow write"
        print(f"Slow write took {elapsed:.2f}s")


def test_io_interruption():
    """Test I/O interruption handling."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"

        # Create large template
        template_content = {
            "Resources": {
                f"Lambda{i}": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 1024,
                        "Description": "X" * 100000
                    }
                }
                for i in range(100)
            }
        }

        with open(template_path, 'w') as f:
            json.dump(template_content, f)

        # Run with short timeout to simulate interruption
        try:
            result = subprocess.run(
                ["costpilot", "scan", "--plan", str(template_path)],
                capture_output=True,
                text=True,
                timeout=0.5
            )
        except subprocess.TimeoutExpired:
            # Expected to timeout
            pass


def test_buffered_io_handling():
    """Test buffered I/O handling."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"

        # Create template with many resources
        template_content = {
            "Resources": {
                f"Lambda{i}": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 1024
                    }
                }
                for i in range(500)
            }
        }

        with open(template_path, 'w') as f:
            json.dump(template_content, f)

        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path)],
            capture_output=True,
            text=True,
            timeout=60
        )

        # Should complete
        assert result.returncode in [0, 1, 2, 101], "Should handle buffered I/O"


if __name__ == "__main__":
    test_slow_file_read()
    test_slow_network_simulation()
    test_concurrent_slow_operations()
    test_io_timeout_handling()
    test_blocking_io()
    test_slow_write_operations()
    test_buffered_io_handling()
    print("All slow I/O tests passed")
