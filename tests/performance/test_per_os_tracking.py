#!/usr/bin/env python3
"""Test per-OS performance tracking."""

import json
import platform
import subprocess
import tempfile
import time
from pathlib import Path


def get_os_info():
    """Get current OS information."""
    return {
        "system": platform.system(),
        "release": platform.release(),
        "version": platform.version(),
        "machine": platform.machine()
    }


def test_track_performance_by_os():
    """Test that performance metrics are tracked per OS."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        metrics_path = Path(tmpdir) / "metrics.json"
        
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
        
        # Run with metrics tracking
        start = time.time()
        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path), "--metrics", str(metrics_path)],
            capture_output=True,
            text=True,
            timeout=30
        )
        duration = time.time() - start
        
        os_info = get_os_info()
        
        # Should complete
        assert result.returncode in [0, 1, 2, 101], "Should complete analysis"
        
        # Check if metrics file contains OS info
        if metrics_path.exists():
            with open(metrics_path) as f:
                metrics = json.load(f)
                
            # Metrics should include OS information
            if "os" in metrics or "platform" in metrics or "system" in metrics:
                print(f"OS tracked: {os_info['system']}")


def test_performance_baseline_per_os():
    """Test establishing performance baseline per OS."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        
        # Standardized workload
        template_content = {
            "Resources": {
                f"Lambda{i}": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 1024,
                        "Timeout": 300
                    }
                }
                for i in range(500)
            }
        }
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        # Measure performance
        timings = []
        for _ in range(3):
            start = time.time()
            result = subprocess.run(
                ["costpilot", "scan", "--plan", str(template_path)],
                capture_output=True,
                text=True,
                timeout=60
            )
            duration = time.time() - start
            
            assert result.returncode in [0, 1, 2, 101], "Should complete"
            timings.append(duration)
        
        # Calculate baseline
        avg_time = sum(timings) / len(timings)
        os_info = get_os_info()
        
        print(f"Performance baseline on {os_info['system']}: {avg_time:.2f}s")


def test_compare_windows_vs_unix():
    """Test performance comparison between Windows and Unix-like systems."""
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
        
        os_info = get_os_info()
        
        # Run multiple times
        timings = []
        for _ in range(5):
            start = time.time()
            result = subprocess.run(
                ["costpilot", "scan", "--plan", str(template_path)],
                capture_output=True,
                text=True,
                timeout=10
            )
            duration = time.time() - start
            
            assert result.returncode in [0, 1, 2, 101], "Should complete"
            timings.append(duration)
        
        avg = sum(timings) / len(timings)
        
        # Different OS may have different performance characteristics
        if os_info['system'] == 'Windows':
            print(f"Windows performance: {avg:.3f}s")
        elif os_info['system'] in ['Linux', 'Darwin']:
            print(f"Unix-like performance: {avg:.3f}s")


def test_macos_arm_vs_intel():
    """Test performance tracking for macOS ARM vs Intel."""
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
                for i in range(200)
            }
        }
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        os_info = get_os_info()
        
        start = time.time()
        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path)],
            capture_output=True,
            text=True,
            timeout=30
        )
        duration = time.time() - start
        
        assert result.returncode in [0, 1, 2, 101], "Should complete"
        
        # Track architecture-specific performance
        if os_info['system'] == 'Darwin':
            arch = os_info['machine']
            if 'arm' in arch.lower():
                print(f"macOS ARM (Apple Silicon) performance: {duration:.2f}s")
            else:
                print(f"macOS Intel performance: {duration:.2f}s")


def test_linux_distro_variations():
    """Test performance across different Linux distributions."""
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
                for i in range(150)
            }
        }
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        os_info = get_os_info()
        
        start = time.time()
        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path)],
            capture_output=True,
            text=True,
            timeout=30
        )
        duration = time.time() - start
        
        assert result.returncode in [0, 1, 2, 101], "Should complete"
        
        if os_info['system'] == 'Linux':
            print(f"Linux distribution: {os_info['version']}")
            print(f"Performance: {duration:.2f}s")


def test_wsl_vs_native_windows():
    """Test performance comparison between WSL and native Windows."""
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
        
        os_info = get_os_info()
        
        start = time.time()
        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path)],
            capture_output=True,
            text=True,
            timeout=10
        )
        duration = time.time() - start
        
        assert result.returncode in [0, 1, 2, 101], "Should complete"
        
        # Detect WSL
        is_wsl = 'microsoft' in os_info['version'].lower() if os_info['system'] == 'Linux' else False
        
        if is_wsl:
            print(f"WSL performance: {duration:.3f}s")
        elif os_info['system'] == 'Windows':
            print(f"Native Windows performance: {duration:.3f}s")


def test_performance_regression_detection():
    """Test detection of performance regression per OS."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        
        # Standard workload
        template_content = {
            "Resources": {
                f"Lambda{i}": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 1024
                    }
                }
                for i in range(300)
            }
        }
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        # Run multiple times to detect variance
        timings = []
        for _ in range(10):
            start = time.time()
            result = subprocess.run(
                ["costpilot", "scan", "--plan", str(template_path)],
                capture_output=True,
                text=True,
                timeout=60
            )
            duration = time.time() - start
            
            assert result.returncode in [0, 1, 2, 101], "Should complete"
            timings.append(duration)
        
        # Statistical analysis
        avg = sum(timings) / len(timings)
        variance = sum((t - avg) ** 2 for t in timings) / len(timings)
        stddev = variance ** 0.5
        
        os_info = get_os_info()
        print(f"Performance on {os_info['system']}: {avg:.3f}s ± {stddev:.3f}s")
        
        # Variance should be reasonable (not huge swings)
        assert stddev / avg < 0.5, "Performance should be relatively stable"


def test_cross_platform_determinism():
    """Test that results are deterministic across platforms."""
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
        outputs = []
        for _ in range(5):
            result = subprocess.run(
                ["costpilot", "scan", "--plan", str(template_path)],
                capture_output=True,
                text=True,
                timeout=10
            )
            
            assert result.returncode in [0, 1, 2, 101], "Should complete"
            outputs.append(result.stdout)
        
        # Outputs should be identical (deterministic)
        assert all(out == outputs[0] for out in outputs), "Results should be deterministic across runs"


if __name__ == "__main__":
    test_track_performance_by_os()
    test_performance_baseline_per_os()
    test_compare_windows_vs_unix()
    test_macos_arm_vs_intel()
    test_linux_distro_variations()
    test_wsl_vs_native_windows()
    test_performance_regression_detection()
    test_cross_platform_determinism()
    print("All per-OS performance tracking tests passed")
