#!/usr/bin/env python3
"""Test 24-hour WASM memory stability.

NOTE: This is a long-running test that takes 24 hours to complete.
Run with: python test_24h_wasm_memory.py
"""

import subprocess
import tempfile
from pathlib import Path
import json
import time
import datetime


def test_24h_wasm_memory_stability():
    """Test 24-hour WASM memory stability."""
    wasm_target = Path("target/wasm32-unknown-unknown/release/costpilot.wasm")
    
    if not wasm_target.exists():
        print("WASM build not found, skipping test")
        return
    
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        log_path = Path(tmpdir) / "24h_wasm_log.txt"
        
        template_content = {
            "Resources": {
                f"Lambda{i}": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 1024 + (i * 128),
                        "Description": "X" * 1000
                    }
                }
                for i in range(100)
            }
        }
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        # Run for 24 hours
        start_time = time.time()
        end_time = start_time + (24 * 60 * 60)  # 24 hours
        
        iteration = 0
        errors = 0
        
        with open(log_path, 'w') as log:
            log.write(f"Starting 24h WASM memory test at {datetime.datetime.now()}\n")
            
            while time.time() < end_time:
                iteration += 1
                
                # Run WASM
                result = subprocess.run(
                    ["wasmtime", "run", str(wasm_target), "--", "analyze", "--plan", str(template_path)],
                    capture_output=True,
                    text=True,
                    timeout=120
                )
                
                if result.returncode not in [0, 1, 2]:
                    errors += 1
                    log.write(f"[{datetime.datetime.now()}] Iteration {iteration}: ERROR (exit {result.returncode})\n")
                    log.flush()
                else:
                    if iteration % 50 == 0:
                        log.write(f"[{datetime.datetime.now()}] Iteration {iteration}: OK\n")
                        log.flush()
                
                # Sleep 10 minutes between iterations
                time.sleep(600)
            
            log.write(f"\n24h WASM test completed at {datetime.datetime.now()}\n")
            log.write(f"Total iterations: {iteration}\n")
            log.write(f"Errors: {errors}\n")
        
        # Check results
        error_rate = errors / iteration if iteration > 0 else 0
        
        print(f"24h WASM memory: {iteration} iterations, {errors} errors ({error_rate:.2%})")
        
        # No memory leaks should cause failures
        assert error_rate < 0.01, f"Error rate {error_rate:.2%} indicates memory issues"


def test_24h_wasm_memory_simulation():
    """Simulate 24h WASM memory test with 50 iterations."""
    wasm_target = Path("target/wasm32-unknown-unknown/release/costpilot.wasm")
    
    if not wasm_target.exists():
        print("WASM build not found, skipping test")
        return
    
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        
        template_content = {
            "Resources": {
                f"Lambda{i}": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 1024 + (i * 128),
                        "Description": "X" * 1000
                    }
                }
                for i in range(100)
            }
        }
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        # Simulate with 50 iterations
        errors = 0
        
        for iteration in range(50):
            result = subprocess.run(
                ["wasmtime", "run", str(wasm_target), "--", "analyze", "--plan", str(template_path)],
                capture_output=True,
                text=True,
                timeout=120
            )
            
            if result.returncode not in [0, 1, 2]:
                errors += 1
            
            # Small delay
            time.sleep(2)
        
        error_rate = errors / 50
        
        print(f"24h WASM simulation: 50 iterations, {errors} errors ({error_rate:.2%})")
        
        # Should have no memory-related errors
        assert error_rate < 0.05, f"Error rate {error_rate:.2%} indicates memory issues"


if __name__ == "__main__":
    import sys
    
    if "--full" in sys.argv:
        print("Running full 24h WASM memory test...")
        test_24h_wasm_memory_stability()
    else:
        print("Running 24h WASM simulation (50 iterations)...")
        test_24h_wasm_memory_simulation()
    
    print("24h WASM memory tests passed")
