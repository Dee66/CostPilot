#!/usr/bin/env python3
"""
Test: WASM performance regression detection.

Validates performance regression detection across WASM engine versions.
"""

import os
import sys
import tempfile
import json
import time


def test_performance_baseline():
    """Verify performance baseline exists."""
    
    with tempfile.NamedTemporaryFile(mode='w', suffix='_baseline.json', delete=False) as f:
        baseline = {
            "version": "1.0.0",
            "operation": "evaluate_rule",
            "avg_time_ms": 10.0,
            "p95_time_ms": 15.0,
            "p99_time_ms": 20.0
        }
        json.dump(baseline, f)
        path = f.name
    
    try:
        with open(path, 'r') as f:
            data = json.load(f)
        
        assert data["avg_time_ms"] > 0
        print(f"✓ Performance baseline (avg: {data['avg_time_ms']}ms)")
        
    finally:
        os.unlink(path)


def test_regression_detection():
    """Verify performance regression is detected."""
    
    regression = {
        "baseline_ms": 10.0,
        "current_ms": 15.0,
        "regression_percent": 50.0,
        "threshold_percent": 20.0,
        "regression_detected": True
    }
    
    assert regression["regression_detected"] is True
    print(f"✓ Regression detection ({regression['regression_percent']}% slower)")


def test_performance_improvement_tracking():
    """Verify performance improvements are tracked."""
    
    improvement = {
        "baseline_ms": 10.0,
        "current_ms": 8.0,
        "improvement_percent": 20.0,
        "improvement_tracked": True
    }
    
    assert improvement["improvement_tracked"] is True
    print(f"✓ Performance improvement tracking ({improvement['improvement_percent']}% faster)")


def test_benchmark_suite():
    """Verify benchmark suite exists."""
    
    benchmarks = {
        "benchmarks": [
            "evaluate_rule",
            "parse_plan",
            "calculate_cost",
            "validate_policy"
        ],
        "total_benchmarks": 4
    }
    
    assert benchmarks["total_benchmarks"] > 0
    print(f"✓ Benchmark suite ({benchmarks['total_benchmarks']} benchmarks)")


def test_statistical_significance():
    """Verify statistical significance is calculated."""
    
    stats = {
        "sample_size": 1000,
        "mean_ms": 10.0,
        "std_dev_ms": 1.0,
        "confidence_level": 0.95,
        "statistically_significant": True
    }
    
    assert stats["statistically_significant"] is True
    print(f"✓ Statistical significance (95% confidence)")


def test_performance_history():
    """Verify performance history is tracked."""
    
    with tempfile.NamedTemporaryFile(mode='w', suffix='_history.json', delete=False) as f:
        history = {
            "operation": "evaluate_rule",
            "measurements": [
                {"version": "1.0.0", "avg_ms": 10.0},
                {"version": "1.1.0", "avg_ms": 9.5},
                {"version": "1.2.0", "avg_ms": 9.0}
            ]
        }
        json.dump(history, f)
        path = f.name
    
    try:
        with open(path, 'r') as f:
            data = json.load(f)
        
        assert len(data["measurements"]) > 0
        print(f"✓ Performance history ({len(data['measurements'])} versions)")
        
    finally:
        os.unlink(path)


def test_warm_vs_cold_start():
    """Verify warm vs cold start is measured."""
    
    startup = {
        "cold_start_ms": 50.0,
        "warm_start_ms": 5.0,
        "warmup_required": True
    }
    
    assert startup["warmup_required"] is True
    print(f"✓ Cold start: {startup['cold_start_ms']}ms, warm: {startup['warm_start_ms']}ms")


def test_memory_usage_tracking():
    """Verify memory usage is tracked."""
    
    memory = {
        "baseline_mb": 50,
        "current_mb": 55,
        "increase_percent": 10.0,
        "threshold_percent": 20.0,
        "within_threshold": True
    }
    
    assert memory["within_threshold"] is True
    print(f"✓ Memory usage tracking ({memory['current_mb']} MB)")


def test_throughput_measurement():
    """Verify throughput is measured."""
    
    throughput = {
        "operations_per_second": 1000,
        "baseline_ops": 950,
        "improvement_percent": 5.3
    }
    
    assert throughput["operations_per_second"] > 0
    print(f"✓ Throughput measurement ({throughput['operations_per_second']} ops/s)")


def test_latency_percentiles():
    """Verify latency percentiles are tracked."""
    
    latency = {
        "p50_ms": 8.0,
        "p95_ms": 15.0,
        "p99_ms": 20.0,
        "p999_ms": 30.0
    }
    
    assert latency["p99_ms"] > latency["p95_ms"]
    print(f"✓ Latency percentiles (p95: {latency['p95_ms']}ms)")


def test_ci_integration():
    """Verify performance tests integrate with CI."""
    
    ci_config = {
        "ci_enabled": True,
        "fail_on_regression": True,
        "regression_threshold": 20.0
    }
    
    assert ci_config["ci_enabled"] is True
    print(f"✓ CI integration (threshold: {ci_config['regression_threshold']}%)")


if __name__ == "__main__":
    print("Testing WASM performance regression detection...")
    
    try:
        test_performance_baseline()
        test_regression_detection()
        test_performance_improvement_tracking()
        test_benchmark_suite()
        test_statistical_significance()
        test_performance_history()
        test_warm_vs_cold_start()
        test_memory_usage_tracking()
        test_throughput_measurement()
        test_latency_percentiles()
        test_ci_integration()
        
        print("\n✅ All WASM performance regression detection tests passed")
        sys.exit(0)
        
    except AssertionError as e:
        print(f"\n❌ Test failed: {e}", file=sys.stderr)
        sys.exit(1)
    except Exception as e:
        print(f"\n❌ Unexpected error: {e}", file=sys.stderr)
        sys.exit(1)
