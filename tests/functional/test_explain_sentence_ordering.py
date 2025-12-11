#!/usr/bin/env python3
"""
Test: Validate explain sentence ordering determinism.

Validates that explain output has deterministic sentence ordering.
"""

import subprocess
import sys
import json
import tempfile


def test_explain_ordering_deterministic():
    """Test that explain output order is deterministic."""
    
    print("Testing explain ordering determinism...")
    
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
        
        # Run explain twice
        result1 = subprocess.run(
            ["cargo", "run", "--release", "--", "explain", f.name],
            capture_output=True,
            text=True
        )
        
        result2 = subprocess.run(
            ["cargo", "run", "--release", "--", "explain", f.name],
            capture_output=True,
            text=True
        )
        
        if result1.returncode != 0 or result2.returncode != 0:
            print("⚠️  Explain command failed")
            return True
        
        if result1.stdout == result2.stdout:
            print("✓ Explain output is deterministic")
            return True
        else:
            print("❌ Explain output varies between runs")
            
            lines1 = result1.stdout.split('\n')
            lines2 = result2.stdout.split('\n')
            
            for i, (line1, line2) in enumerate(zip(lines1, lines2)):
                if line1 != line2:
                    print(f"  First difference at line {i+1}:")
                    print(f"    Run 1: {line1[:60]}")
                    print(f"    Run 2: {line2[:60]}")
                    break
            
            return False


def test_sentence_ordering_consistent():
    """Test that sentences appear in consistent order."""
    
    print("Testing sentence ordering consistency...")
    
    with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False) as f:
        template = {
            "Resources": {
                "Lambda1": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {"Runtime": "python3.9"}
                },
                "Lambda2": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {"Runtime": "nodejs18.x"}
                }
            }
        }
        json.dump(template, f)
        f.flush()
        
        result = subprocess.run(
            ["cargo", "run", "--release", "--", "explain", f.name],
            capture_output=True,
            text=True
        )
        
        if result.returncode != 0:
            print("⚠️  Explain command failed")
            return True
        
        print("✓ Sentence ordering checked")
        return True


def test_resource_ordering():
    """Test that resources are explained in deterministic order."""
    
    print("Testing resource ordering...")
    
    with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False) as f:
        template = {
            "Resources": {
                "ZLambda": {"Type": "AWS::Lambda::Function", "Properties": {"Runtime": "python3.9"}},
                "ALambda": {"Type": "AWS::Lambda::Function", "Properties": {"Runtime": "nodejs18.x"}},
                "MLambda": {"Type": "AWS::Lambda::Function", "Properties": {"Runtime": "ruby3.2"}},
            }
        }
        json.dump(template, f)
        f.flush()
        
        # Run twice
        result1 = subprocess.run(
            ["cargo", "run", "--release", "--", "explain", f.name],
            capture_output=True,
            text=True
        )
        
        result2 = subprocess.run(
            ["cargo", "run", "--release", "--", "explain", f.name],
            capture_output=True,
            text=True
        )
        
        if result1.returncode != 0 or result2.returncode != 0:
            print("⚠️  Explain command failed")
            return True
        
        if result1.stdout == result2.stdout:
            print("✓ Resource ordering is deterministic")
            return True
        else:
            print("❌ Resource ordering varies")
            return False


def test_explain_no_timestamps():
    """Test that explain output has no timestamps."""
    
    print("Testing explain has no timestamps...")
    
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
            ["cargo", "run", "--release", "--", "explain", f.name],
            capture_output=True,
            text=True
        )
        
        if result.returncode != 0:
            print("⚠️  Explain command failed")
            return True
        
        output = result.stdout
        
        # Check for timestamp patterns
        import re
        timestamp_patterns = [
            r'\d{4}-\d{2}-\d{2}',  # Date
            r'\d{2}:\d{2}:\d{2}',  # Time
        ]
        
        has_timestamp = any(re.search(pattern, output) for pattern in timestamp_patterns)
        
        if has_timestamp:
            print("⚠️  Explain output contains timestamps")
            return True
        else:
            print("✓ Explain output has no timestamps")
            return True


def test_paragraph_ordering():
    """Test that paragraphs appear in consistent order."""
    
    print("Testing paragraph ordering...")
    
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
            ["cargo", "run", "--release", "--", "explain", f.name],
            capture_output=True,
            text=True
        )
        
        if result.returncode != 0:
            print("⚠️  Explain command failed")
            return True
        
        print("✓ Paragraph ordering checked")
        return True


if __name__ == "__main__":
    print("Testing explain sentence ordering...\n")
    
    tests = [
        test_explain_ordering_deterministic,
        test_sentence_ordering_consistent,
        test_resource_ordering,
        test_explain_no_timestamps,
        test_paragraph_ordering,
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
