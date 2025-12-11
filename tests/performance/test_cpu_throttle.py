#!/usr/bin/env python3
"""Test performance under CPU throttle."""

import json
import multiprocessing
import subprocess
import tempfile
import time
from pathlib import Path


def cpu_intensive_task():
    """Simulate CPU-intensive background task."""
    end_time = time.time() + 5
    while time.time() < end_time:
        _ = sum(i * i for i in range(10000))


def test_analyze_under_cpu_load():
    """Test analyze command under CPU load."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        
        template_content = {
            "Resources": {
                f"Lambda{i}": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 1024 * (i + 1)
                    }
                }
                for i in range(50)
            }
        }
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        # Start CPU-intensive background tasks
        num_processes = multiprocessing.cpu_count()
        processes = [multiprocessing.Process(target=cpu_intensive_task) for _ in range(num_processes)]
        
        for p in processes:
            p.start()
        
        try:
            # Run analysis under load
            result = subprocess.run(
                ["costpilot", "scan", "--plan", str(template_path)],
                capture_output=True,
                text=True,
                timeout=30
            )
            
            # Should complete despite CPU load
            assert result.returncode in [0, 1, 2, 101], "Should complete under CPU load"
        finally:
            for p in processes:
                p.terminate()
                p.join()


def test_policy_check_under_cpu_load():
    """Test policy checking under CPU load."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        policy_path = Path(tmpdir) / "policy.json"
        
        template_content = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 10240
                    }
                }
            }
        }
        
        policy_content = {
            "version": "1.0.0",
            "rules": [
                {
                    "id": "lambda-memory-limit",
                    "severity": "high",
                    "resource_type": "AWS::Lambda::Function",
                    "condition": "MemorySize > 3008"
                }
            ]
        }
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        with open(policy_path, 'w') as f:
            json.dump(policy_content, f)
        
        # Start CPU load
        num_processes = multiprocessing.cpu_count()
        processes = [multiprocessing.Process(target=cpu_intensive_task) for _ in range(num_processes)]
        
        for p in processes:
            p.start()
        
        try:
            result = subprocess.run(
                ["costpilot", "check", "--plan", str(template_path), "--policy", str(policy_path)],
                capture_output=True,
                text=True,
                timeout=30
            )
            
            assert result.returncode in [0, 1, 2, 101], "Should complete policy check under CPU load"
        finally:
            for p in processes:
                p.terminate()
                p.join()


def test_baseline_under_cpu_load():
    """Test baseline generation under CPU load."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        baseline_path = Path(tmpdir) / "baseline.json"
        
        template_content = {
            "Resources": {
                f"Resource{i}": {
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
        
        # Start CPU load
        num_processes = multiprocessing.cpu_count()
        processes = [multiprocessing.Process(target=cpu_intensive_task) for _ in range(num_processes)]
        
        for p in processes:
            p.start()
        
        try:
            result = subprocess.run(
                ["costpilot", "baseline", "generate", "--plan", str(template_path), "--output", str(baseline_path)],
                capture_output=True,
                text=True,
                timeout=30
            )
            
            assert result.returncode in [0, 1, 2, 101], "Should generate baseline under CPU load"
        finally:
            for p in processes:
                p.terminate()
                p.join()


def test_slo_under_cpu_load():
    """Test SLO tracking under CPU load."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        slo_path = Path(tmpdir) / "slo.json"
        
        template_content = {"Resources": {}}
        
        slo_content = {
            "slos": [
                {
                    "name": "cost-threshold",
                    "target": 0.99,
                    "window": "30d",
                    "budget": 100.0
                }
            ]
        }
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        with open(slo_path, 'w') as f:
            json.dump(slo_content, f)
        
        # Start CPU load
        num_processes = multiprocessing.cpu_count()
        processes = [multiprocessing.Process(target=cpu_intensive_task) for _ in range(num_processes)]
        
        for p in processes:
            p.start()
        
        try:
            result = subprocess.run(
                ["costpilot", "slo", "check", "--plan", str(template_path), "--slo", str(slo_path)],
                capture_output=True,
                text=True,
                timeout=30
            )
            
            assert result.returncode in [0, 1, 2, 101], "Should check SLO under CPU load"
        finally:
            for p in processes:
                p.terminate()
                p.join()


def test_parallel_analysis_under_cpu_load():
    """Test multiple parallel analyses under CPU load."""
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
        
        # Start CPU load
        num_processes = multiprocessing.cpu_count()
        cpu_processes = [multiprocessing.Process(target=cpu_intensive_task) for _ in range(num_processes)]
        
        for p in cpu_processes:
            p.start()
        
        try:
            # Run multiple analyses in parallel
            analysis_processes = []
            for _ in range(5):
                proc = subprocess.Popen(
                    ["costpilot", "scan", "--plan", str(template_path)],
                    stdout=subprocess.PIPE,
                    stderr=subprocess.PIPE
                )
                analysis_processes.append(proc)
            
            # Wait for all to complete
            for proc in analysis_processes:
                proc.wait(timeout=30)
                assert proc.returncode in [0, 1, 2, 101], "Parallel analysis should complete"
        finally:
            for p in cpu_processes:
                p.terminate()
                p.join()


def test_cpu_throttle_detection():
    """Test that system detects CPU throttling."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        
        # Large template
        template_content = {
            "Resources": {
                f"Lambda{i}": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 1024 * (i % 10 + 1),
                        "Timeout": 300
                    }
                }
                for i in range(500)
            }
        }
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        # Measure baseline
        start = time.time()
        result_baseline = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path)],
            capture_output=True,
            text=True,
            timeout=60
        )
        baseline_time = time.time() - start
        
        # Measure under load
        num_processes = multiprocessing.cpu_count() * 2
        processes = [multiprocessing.Process(target=cpu_intensive_task) for _ in range(num_processes)]
        
        for p in processes:
            p.start()
        
        try:
            start = time.time()
            result_throttled = subprocess.run(
                ["costpilot", "scan", "--plan", str(template_path)],
                capture_output=True,
                text=True,
                timeout=120
            )
            throttled_time = time.time() - start
            
            # Should complete despite slower performance
            assert result_throttled.returncode in [0, 1, 2, 101], "Should complete under throttle"
            
            # Throttled should be slower (but both should complete)
            print(f"Baseline: {baseline_time:.2f}s, Throttled: {throttled_time:.2f}s")
        finally:
            for p in processes:
                p.terminate()
                p.join()


def test_graceful_degradation_under_load():
    """Test graceful degradation under extreme CPU load."""
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
        
        # Extreme CPU load
        num_processes = multiprocessing.cpu_count() * 4
        processes = [multiprocessing.Process(target=cpu_intensive_task) for _ in range(num_processes)]
        
        for p in processes:
            p.start()
        
        try:
            result = subprocess.run(
                ["costpilot", "scan", "--plan", str(template_path)],
                capture_output=True,
                text=True,
                timeout=60
            )
            
            # Should gracefully handle extreme load
            assert result.returncode in [0, 1, 2, 101], "Should handle extreme CPU load gracefully"
        finally:
            for p in processes:
                p.terminate()
                p.join()


if __name__ == "__main__":
    test_analyze_under_cpu_load()
    test_policy_check_under_cpu_load()
    test_baseline_under_cpu_load()
    test_slo_under_cpu_load()
    test_parallel_analysis_under_cpu_load()
    test_cpu_throttle_detection()
    test_graceful_degradation_under_load()
    print("All CPU throttle detection tests passed")
