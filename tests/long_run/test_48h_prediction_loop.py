#!/usr/bin/env python3
"""Test 48-hour prediction loop stability.

NOTE: This is a long-running test that takes 48 hours to complete.
Run with: python test_48h_prediction_loop.py
"""

import subprocess
import tempfile
from pathlib import Path
import json
import time
import datetime


def test_48h_prediction_loop():
    """Test 48-hour continuous prediction loop."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        log_path = Path(tmpdir) / "48h_log.txt"
        
        template_content = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 1024,
                        "Timeout": 300
                    }
                },
                "DynamoDB": {
                    "Type": "AWS::DynamoDB::Table",
                    "Properties": {
                        "BillingMode": "PROVISIONED",
                        "ProvisionedThroughput": {
                            "ReadCapacityUnits": 5,
                            "WriteCapacityUnits": 5
                        }
                    }
                }
            }
        }
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        # Run for 48 hours
        start_time = time.time()
        end_time = start_time + (48 * 60 * 60)  # 48 hours
        
        iteration = 0
        errors = 0
        
        with open(log_path, 'w') as log:
            log.write(f"Starting 48h prediction loop at {datetime.datetime.now()}\n")
            
            while time.time() < end_time:
                iteration += 1
                
                # Run prediction
                result = subprocess.run(
                    ["costpilot", "predict", "--plan", str(template_path)],
                    capture_output=True,
                    text=True,
                    timeout=60
                )
                
                if result.returncode not in [0, 1]:
                    errors += 1
                    log.write(f"[{datetime.datetime.now()}] Iteration {iteration}: ERROR (exit {result.returncode})\n")
                    log.flush()
                else:
                    if iteration % 100 == 0:
                        log.write(f"[{datetime.datetime.now()}] Iteration {iteration}: OK\n")
                        log.flush()
                
                # Sleep 5 minutes between iterations
                time.sleep(300)
            
            log.write(f"\n48h loop completed at {datetime.datetime.now()}\n")
            log.write(f"Total iterations: {iteration}\n")
            log.write(f"Errors: {errors}\n")
        
        # Check results
        error_rate = errors / iteration if iteration > 0 else 0
        
        print(f"48h prediction loop: {iteration} iterations, {errors} errors ({error_rate:.2%})")
        
        # Accept up to 1% error rate
        assert error_rate < 0.01, f"Error rate {error_rate:.2%} too high"


def test_48h_prediction_loop_simulation():
    """Simulate 48h prediction loop with 100 iterations (5 minutes)."""
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
        
        # Simulate with 100 iterations (faster test)
        errors = 0
        
        for iteration in range(100):
            result = subprocess.run(
                ["costpilot", "predict", "--plan", str(template_path)],
                capture_output=True,
                text=True,
                timeout=60
            )
            
            if result.returncode not in [0, 1]:
                errors += 1
            
            # Small delay
            time.sleep(1)
        
        error_rate = errors / 100
        
        print(f"48h simulation: 100 iterations, {errors} errors ({error_rate:.2%})")
        
        # Should have very low error rate
        assert error_rate < 0.05, f"Error rate {error_rate:.2%} too high"


if __name__ == "__main__":
    import sys
    
    if "--full" in sys.argv:
        print("Running full 48h prediction loop...")
        test_48h_prediction_loop()
    else:
        print("Running 48h simulation (100 iterations)...")
        test_48h_prediction_loop_simulation()
    
    print("48h prediction loop tests passed")
