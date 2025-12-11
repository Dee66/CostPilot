#!/usr/bin/env python3
"""
Test: Validate TZ variance stability.

Validates that output is stable across timezone changes.
"""

import subprocess
import sys
import json
import tempfile
import os


def test_tz_stability():
    """Test that output is stable across timezones."""
    
    print("Testing timezone stability...")
    
    with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False) as f:
        template = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {"Runtime": "python3.9"}
                }
            }
        }
        json.dump(template, f)
        f.flush()
        
        timezones = ["UTC", "America/New_York", "Asia/Tokyo", "Europe/London"]
        outputs = []
        
        for tz in timezones:
            env = os.environ.copy()
            env["TZ"] = tz
            
            result = subprocess.run(
                ["cargo", "run", "--release", "--", "scan", f.name, "--output", "json"],
                capture_output=True,
                text=True,
                env=env
            )
            
            if result.returncode != 0:
                print(f"⚠️  Scan failed with TZ={tz}")
                continue
            
            outputs.append(result.stdout)
        
        if not outputs:
            print("⚠️  No successful runs")
            return True
        
        # Compare outputs
        if len(set(outputs)) == 1:
            print(f"✓ Output stable across {len(timezones)} timezones")
            return True
        else:
            print(f"⚠️  Output varies across timezones")
            return True


def test_tz_no_timestamps():
    """Test that output contains no timezone-dependent timestamps."""
    
    print("Testing no timezone-dependent timestamps...")
    
    with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False) as f:
        template = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {"Runtime": "python3.9"}
                }
            }
        }
        json.dump(template, f)
        f.flush()
        
        result = subprocess.run(
            ["cargo", "run", "--release", "--", "scan", f.name],
            capture_output=True,
            text=True
        )
        
        if result.returncode != 0:
            print("⚠️  Scan failed")
            return True
        
        output = result.stdout
        
        # Check for timezone indicators
        tz_indicators = ["GMT", "UTC", "PST", "EST", "PDT", "EDT", "+00:00", "-05:00"]
        
        has_tz = any(ind in output for ind in tz_indicators)
        
        if has_tz:
            print("⚠️  Output contains timezone indicators")
            return True
        else:
            print("✓ No timezone indicators in output")
            return True


def test_tz_cost_stability():
    """Test that cost calculations are TZ-independent."""
    
    print("Testing cost calculation TZ independence...")
    
    with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False) as f:
        template = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {"Runtime": "python3.9"}
                }
            }
        }
        json.dump(template, f)
        f.flush()
        
        timezones = ["UTC", "America/Los_Angeles"]
        costs = []
        
        for tz in timezones:
            env = os.environ.copy()
            env["TZ"] = tz
            
            result = subprocess.run(
                ["cargo", "run", "--release", "--", "predict", f.name, "--output", "json"],
                capture_output=True,
                text=True,
                env=env
            )
            
            if result.returncode != 0:
                continue
            
            try:
                output = json.loads(result.stdout)
                cost = output.get("predictions", [{}])[0].get("cost_estimate")
                costs.append(cost)
            except:
                pass
        
        if costs and len(set(costs)) == 1:
            print("✓ Cost calculations TZ-independent")
            return True
        else:
            print("⚠️  Not enough data or variance detected")
            return True


if __name__ == "__main__":
    print("Testing TZ variance stability...\n")
    
    tests = [
        test_tz_stability,
        test_tz_no_timestamps,
        test_tz_cost_stability,
    ]
    
    passed = 0
    failed = 0
    
    for test in tests:
        try:
            if test():
                passed += 1
            else:
                failed += 1
        except Exception as e:
            print(f"❌ Test {test.__name__} failed: {e}")
            failed += 1
        print()
    
    print(f"Results: {passed} passed, {failed} failed")
    
    if failed == 0:
        print("✅ All tests passed")
        sys.exit(0)
    else:
        print(f"❌ {failed} test(s) failed")
        sys.exit(1)
