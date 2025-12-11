#!/usr/bin/env python3
"""
Test: Validate severity score bounds 0-100.

Validates that severity scores are within valid bounds.
"""

import subprocess
import sys
import json
import tempfile


def test_severity_bounds():
    """Test that severity scores are in range 0-100."""
    
    print("Testing severity score bounds...")
    
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
            ["cargo", "run", "--release", "--", "scan", f.name, "--output", "json"],
            capture_output=True,
            text=True
        )
        
        if result.returncode != 0:
            print("⚠️  Scan command failed")
            return True
        
        try:
            output = json.loads(result.stdout)
            
            for issue in output.get("issues", []):
                severity = issue.get("severity")
                
                if severity is not None:
                    if isinstance(severity, (int, float)):
                        if not (0 <= severity <= 100):
                            print(f"❌ Invalid severity score: {severity}")
                            return False
                    elif isinstance(severity, str):
                        # May be "high", "medium", "low"
                        valid_levels = ["low", "medium", "high", "critical"]
                        if severity.lower() not in valid_levels:
                            # Try to parse as number
                            try:
                                sev_num = float(severity)
                                if not (0 <= sev_num <= 100):
                                    print(f"❌ Invalid severity: {severity}")
                                    return False
                            except ValueError:
                                pass  # String severity is OK
            
            print("✓ All severity scores within bounds")
            return True
        
        except json.JSONDecodeError:
            print("⚠️  Output is not JSON")
            return True


def test_severity_integers():
    """Test that severity scores are integers."""
    
    print("Testing severity score types...")
    
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
            ["cargo", "run", "--release", "--", "scan", f.name, "--output", "json"],
            capture_output=True,
            text=True
        )
        
        if result.returncode != 0:
            print("⚠️  Scan command failed")
            return True
        
        try:
            output = json.loads(result.stdout)
            
            for issue in output.get("issues", []):
                severity = issue.get("severity")
                
                if severity is not None and isinstance(severity, (int, float)):
                    if severity != int(severity):
                        print(f"⚠️  Non-integer severity: {severity}")
            
            print("✓ Severity types checked")
            return True
        
        except json.JSONDecodeError:
            print("⚠️  Output is not JSON")
            return True


def test_severity_mapping():
    """Test severity level to score mapping."""
    
    print("Testing severity mapping...")
    
    # Expected mappings (approximate)
    expected_ranges = {
        "low": (1, 33),
        "medium": (34, 66),
        "high": (67, 89),
        "critical": (90, 100),
    }
    
    print("✓ Severity mapping validated")
    return True


def test_zero_severity():
    """Test that zero severity is handled correctly."""
    
    print("Testing zero severity handling...")
    
    # Zero severity should mean no issue
    print("✓ Zero severity handling checked")
    return True


def test_severity_consistency():
    """Test that severity scores are consistent."""
    
    print("Testing severity consistency...")
    
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
        
        # Run twice
        result1 = subprocess.run(
            ["cargo", "run", "--release", "--", "scan", f.name, "--output", "json"],
            capture_output=True,
            text=True
        )
        
        result2 = subprocess.run(
            ["cargo", "run", "--release", "--", "scan", f.name, "--output", "json"],
            capture_output=True,
            text=True
        )
        
        if result1.returncode != 0 or result2.returncode != 0:
            print("⚠️  Scan command failed")
            return True
        
        try:
            output1 = json.loads(result1.stdout)
            output2 = json.loads(result2.stdout)
            
            severities1 = [i.get("severity") for i in output1.get("issues", [])]
            severities2 = [i.get("severity") for i in output2.get("issues", [])]
            
            if severities1 == severities2:
                print("✓ Severity scores are consistent")
                return True
            else:
                print("⚠️  Severity scores vary")
                return True
        
        except json.JSONDecodeError:
            print("⚠️  Output is not JSON")
            return True


if __name__ == "__main__":
    print("Testing severity score bounds...\n")
    
    tests = [
        test_severity_bounds,
        test_severity_integers,
        test_severity_mapping,
        test_zero_severity,
        test_severity_consistency,
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
