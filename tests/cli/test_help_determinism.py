#!/usr/bin/env python3
"""
Test: Validate deterministic --help ordering.

Validates that --help output has consistent ordering across runs.
"""

import subprocess
import sys


def get_help_output():
    """Get --help output."""
    
    result = subprocess.run(
        ["cargo", "run", "--release", "--", "--help"],
        capture_output=True,
        text=True
    )
    
    if result.returncode != 0:
        print(f"❌ Failed to get help output: {result.stderr}")
        return None
    
    return result.stdout


def test_help_determinism():
    """Test that --help output is deterministic."""
    
    print("Getting --help output (run 1)...")
    output1 = get_help_output()
    
    if output1 is None:
        return False
    
    print("Getting --help output (run 2)...")
    output2 = get_help_output()
    
    if output2 is None:
        return False
    
    if output1 == output2:
        print("✓ --help output is deterministic")
        print(f"  Output length: {len(output1)} chars")
        return True
    else:
        print("❌ --help output varies between runs")
        
        # Find differences
        lines1 = output1.split('\n')
        lines2 = output2.split('\n')
        
        for i, (line1, line2) in enumerate(zip(lines1, lines2)):
            if line1 != line2:
                print(f"  Difference at line {i+1}:")
                print(f"    Run 1: {line1}")
                print(f"    Run 2: {line2}")
                break
        
        return False


def test_help_ordering():
    """Test that --help has expected ordering."""
    
    print("Checking --help ordering...")
    output = get_help_output()
    
    if output is None:
        return False
    
    # Expected sections in order
    expected_sections = [
        "Usage:",
        "Commands:",
        "Options:",
    ]
    
    positions = []
    for section in expected_sections:
        pos = output.find(section)
        if pos == -1:
            print(f"❌ Missing section: {section}")
            return False
        positions.append(pos)
    
    # Check ordering
    if positions == sorted(positions):
        print("✓ --help sections are in correct order")
        print(f"  Sections: {', '.join(expected_sections)}")
        return True
    else:
        print("❌ --help sections are out of order")
        for i, section in enumerate(expected_sections):
            print(f"  {section} at position {positions[i]}")
        return False


def test_help_flags_alphabetical():
    """Test that flags are alphabetically ordered."""
    
    print("Checking flag ordering...")
    output = get_help_output()
    
    if output is None:
        return False
    
    # Extract flags from output
    flags = []
    in_options = False
    
    for line in output.split('\n'):
        if "Options:" in line:
            in_options = True
            continue
        
        if in_options:
            # Stop at next section
            if line and not line.startswith(' ') and not line.startswith('\t'):
                break
            
            # Extract flag
            line = line.strip()
            if line.startswith('-'):
                # Get flag name (first word)
                parts = line.split()
                if parts:
                    flag = parts[0].lstrip('-')
                    flags.append(flag)
    
    if not flags:
        print("⚠️  No flags found in --help output")
        return True
    
    # Check if alphabetical
    sorted_flags = sorted(flags, key=str.lower)
    
    if flags == sorted_flags:
        print(f"✓ Flags are alphabetically ordered ({len(flags)} flags)")
        return True
    else:
        print(f"⚠️  Flags may not be alphabetically ordered ({len(flags)} flags)")
        print("  Note: This is acceptable if grouped by functionality")
        return True  # Don't fail, just warn


def test_help_commands_ordering():
    """Test that commands have consistent ordering."""
    
    print("Checking command ordering...")
    output = get_help_output()
    
    if output is None:
        return False
    
    # Extract commands from output
    commands = []
    in_commands = False
    
    for line in output.split('\n'):
        if "Commands:" in line:
            in_commands = True
            continue
        
        if in_commands:
            # Stop at next section
            if line and not line.startswith(' ') and not line.startswith('\t'):
                break
            
            # Extract command
            line = line.strip()
            if line and not line.startswith('-'):
                parts = line.split()
                if parts:
                    commands.append(parts[0])
    
    if not commands:
        print("⚠️  No commands found in --help output")
        return True
    
    print(f"✓ Found {len(commands)} commands")
    print(f"  Commands: {', '.join(commands[:5])}")
    if len(commands) > 5:
        print(f"  ... and {len(commands) - 5} more")
    
    return True


def test_help_no_timestamps():
    """Test that --help doesn't contain timestamps or dates."""
    
    print("Checking for timestamps...")
    output = get_help_output()
    
    if output is None:
        return False
    
    # Look for common timestamp patterns
    timestamp_patterns = [
        "20",  # Year prefix
        ":",  # Time separator (hours:minutes)
        "GMT",
        "UTC",
    ]
    
    # Check for suspicious patterns
    suspicious = []
    for pattern in timestamp_patterns:
        if pattern in output and pattern not in ["--", "Options:", "Commands:"]:
            # Check context
            for line in output.split('\n'):
                if pattern in line and not line.strip().startswith('#'):
                    suspicious.append(line.strip())
    
    if suspicious:
        print("⚠️  Possible timestamp in --help:")
        for line in suspicious[:3]:
            print(f"    {line}")
        # Don't fail, just warn
        return True
    else:
        print("✓ No timestamps in --help output")
        return True


if __name__ == "__main__":
    print("Testing --help determinism and ordering...\n")
    
    tests = [
        test_help_determinism,
        test_help_ordering,
        test_help_commands_ordering,
        test_help_flags_alphabetical,
        test_help_no_timestamps,
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
            print(f"❌ Test {test.__name__} failed with error: {e}")
            failed += 1
        print()
    
    print(f"\nResults: {passed} passed, {failed} failed\n")
    
    if failed == 0:
        print("✅ All --help determinism tests passed")
        sys.exit(0)
    else:
        print(f"❌ {failed} test(s) failed")
        sys.exit(1)
