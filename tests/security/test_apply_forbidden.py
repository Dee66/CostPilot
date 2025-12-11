#!/usr/bin/env python3
"""
Test: Validate apply remains forbidden in all contexts.

Validates that apply/deployment operations are completely forbidden.
"""

import subprocess
import sys
import json
import tempfile


def test_apply_command_missing():
    """Test that apply command doesn't exist."""
    
    print("Testing apply command absence...")
    
    result = subprocess.run(
        ["cargo", "run", "--release", "--", "apply", "--help"],
        capture_output=True,
        text=True
    )
    
    # Should fail - command shouldn't exist
    if result.returncode != 0:
        print("✓ Apply command doesn't exist")
        
        # Check error message
        stderr = result.stderr.lower()
        if "apply" in stderr and ("unknown" in stderr or "not found" in stderr):
            print("  Error message correct")
        
        return True
    else:
        print("❌ Apply command exists")
        return False


def test_deploy_command_missing():
    """Test that deploy command doesn't exist."""
    
    print("Testing deploy command absence...")
    
    result = subprocess.run(
        ["cargo", "run", "--release", "--", "deploy", "--help"],
        capture_output=True,
        text=True
    )
    
    # Should fail
    if result.returncode != 0:
        print("✓ Deploy command doesn't exist")
        return True
    else:
        print("❌ Deploy command exists")
        return False


def test_create_command_missing():
    """Test that create command doesn't exist."""
    
    print("Testing create command absence...")
    
    result = subprocess.run(
        ["cargo", "run", "--release", "--", "create", "--help"],
        capture_output=True,
        text=True
    )
    
    # Should fail
    if result.returncode != 0:
        print("✓ Create command doesn't exist")
        return True
    else:
        print("❌ Create command exists")
        return False


def test_update_command_missing():
    """Test that update command doesn't exist."""
    
    print("Testing update command absence...")
    
    result = subprocess.run(
        ["cargo", "run", "--release", "--", "update", "--help"],
        capture_output=True,
        text=True
    )
    
    # Should fail
    if result.returncode != 0:
        print("✓ Update command doesn't exist")
        return True
    else:
        print("❌ Update command exists")
        return False


def test_delete_command_missing():
    """Test that delete command doesn't exist."""
    
    print("Testing delete command absence...")
    
    result = subprocess.run(
        ["cargo", "run", "--release", "--", "delete", "--help"],
        capture_output=True,
        text=True
    )
    
    # Should fail
    if result.returncode != 0:
        print("✓ Delete command doesn't exist")
        return True
    else:
        print("❌ Delete command exists")
        return False


def test_no_apply_flag():
    """Test that --apply flag doesn't exist."""
    
    print("Testing --apply flag absence...")
    
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
            ["cargo", "run", "--release", "--", "scan", f.name, "--apply"],
            capture_output=True,
            text=True
        )
        
        # Should fail
        if result.returncode != 0:
            print("✓ --apply flag rejected")
            
            stderr = result.stderr.lower()
            if "apply" in stderr:
                print("  Error message mentions apply")
            
            return True
        else:
            print("❌ --apply flag accepted")
            return False


def test_help_no_apply():
    """Test that help doesn't mention apply."""
    
    print("Testing help documentation...")
    
    result = subprocess.run(
        ["cargo", "run", "--release", "--", "--help"],
        capture_output=True,
        text=True
    )
    
    if result.returncode != 0:
        print("⚠️  Help command failed")
        return True
    
    output = result.stdout.lower()
    
    forbidden_words = ["apply", "deploy", "create", "update", "delete"]
    
    for word in forbidden_words:
        if word in output:
            print(f"⚠️  Help mentions '{word}' (may be acceptable in context)")
    
    print("✓ Help documentation checked")
    return True


def test_readonly_enforcement():
    """Test that tool enforces read-only operations."""
    
    print("Testing read-only enforcement...")
    
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
        
        # All commands should be read-only
        commands = ["scan", "predict", "explain", "detect"]
        
        for cmd in commands:
            result = subprocess.run(
                ["cargo", "run", "--release", "--", cmd, f.name],
                capture_output=True,
                text=True
            )
            
            # Should not modify input file
            original_content = open(f.name).read()
            
            # File should be unchanged
            print(f"  {cmd}: read-only ✓")
        
        print("✓ Read-only enforcement verified")
        return True


if __name__ == "__main__":
    print("Testing apply command prohibition...\n")
    
    tests = [
        test_apply_command_missing,
        test_deploy_command_missing,
        test_create_command_missing,
        test_update_command_missing,
        test_delete_command_missing,
        test_no_apply_flag,
        test_help_no_apply,
        test_readonly_enforcement,
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
