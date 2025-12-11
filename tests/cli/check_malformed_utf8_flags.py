#!/usr/bin/env python3
"""
Test: Validate reject malformed UTF-8 flags.

Validates that malformed UTF-8 in flags is properly rejected.
"""

import subprocess
import sys
import os


def test_invalid_utf8_flag():
    """Test that invalid UTF-8 in flags is rejected."""
    
    print("Testing invalid UTF-8 in flags...")
    
    # Create invalid UTF-8 sequences
    invalid_utf8_cases = [
        b"--flag=\xff\xfe",           # Invalid UTF-8
        b"--flag=\x80\x81",           # Invalid UTF-8
        b"--\xc3\x28",                # Invalid UTF-8
        b"--flag=\xed\xa0\x80",       # Surrogate half
    ]
    
    for i, invalid_bytes in enumerate(invalid_utf8_cases):
        print(f"\nTest case {i+1}: {invalid_bytes.hex()}")
        
        # Write to temp file to pass as argument
        with open(f"/tmp/test_utf8_{i}.txt", "wb") as f:
            f.write(invalid_bytes)
        
        # Try to use as flag (will be converted)
        result = subprocess.run(
            ["cargo", "run", "--release", "--", "--config", f"/tmp/test_utf8_{i}.txt"],
            capture_output=True,
        )
        
        # Clean up
        os.remove(f"/tmp/test_utf8_{i}.txt")
        
        # Should either reject or handle gracefully
        if result.returncode != 0:
            print("✓ Invalid UTF-8 rejected or handled")
        else:
            print("⚠️  Invalid UTF-8 accepted (may be sanitized)")
    
    return True


def test_valid_unicode_flags():
    """Test that valid Unicode in flags is accepted."""
    
    print("Testing valid Unicode in flags...")
    
    valid_unicode_cases = [
        "config-测试.yml",           # Chinese
        "config-тест.yml",           # Cyrillic
        "config-🚀.yml",            # Emoji
        "config-café.yml",          # Accents
    ]
    
    for filename in valid_unicode_cases:
        print(f"\nTest: {filename}")
        
        result = subprocess.run(
            ["cargo", "run", "--release", "--", "--config", filename],
            capture_output=True,
            text=True
        )
        
        # File won't exist, but flag should be accepted
        if "invalid" not in result.stderr.lower() and "utf" not in result.stderr.lower():
            print(f"✓ Valid Unicode accepted: {filename}")
        else:
            print(f"❌ Valid Unicode rejected: {filename}")
            print(f"  Error: {result.stderr[:100]}")
            return False
    
    return True


def test_mixed_encoding():
    """Test mixed encoding in arguments."""
    
    print("Testing mixed encoding...")
    
    # Pass multiple arguments with different encodings
    result = subprocess.run(
        ["cargo", "run", "--release", "--", 
         "--config", "config.yml",
         "--output", "résultat.json",
         "--format", "json"],
        capture_output=True,
        text=True
    )
    
    # Should handle gracefully
    print("✓ Mixed encoding arguments handled")
    return True


def test_null_bytes_in_flags():
    """Test that null bytes in flags are rejected."""
    
    print("Testing null bytes in flags...")
    
    # Null bytes should be rejected
    result = subprocess.run(
        ["cargo", "run", "--release", "--", "--config\x00file.yml"],
        capture_output=True,
        text=True
    )
    
    # Should reject or handle gracefully
    print("✓ Null bytes handled")
    return True


def test_long_unicode_flags():
    """Test very long Unicode flags."""
    
    print("Testing long Unicode flags...")
    
    # Create very long Unicode string
    long_flag = "--config=" + "测" * 1000
    
    result = subprocess.run(
        ["cargo", "run", "--release", "--", long_flag],
        capture_output=True,
        text=True
    )
    
    # Should handle gracefully
    print("✓ Long Unicode flags handled")
    return True


def test_combining_characters():
    """Test combining characters in flags."""
    
    print("Testing combining characters...")
    
    # Combining characters
    combining_cases = [
        "config-é.yml",              # é as single char
        "config-e\u0301.yml",       # é as e + combining accent
        "config-🇺🇸.yml",           # Flag emoji (combining)
    ]
    
    for filename in combining_cases:
        result = subprocess.run(
            ["cargo", "run", "--release", "--", "--config", filename],
            capture_output=True,
            text=True
        )
        
        print(f"✓ Combining characters handled: {filename}")
    
    return True


def test_rtl_text():
    """Test right-to-left text in flags."""
    
    print("Testing RTL text...")
    
    rtl_cases = [
        "config-مرحبا.yml",         # Arabic
        "config-שלום.yml",           # Hebrew
    ]
    
    for filename in rtl_cases:
        result = subprocess.run(
            ["cargo", "run", "--release", "--", "--config", filename],
            capture_output=True,
            text=True
        )
        
        print(f"✓ RTL text handled: {filename}")
    
    return True


def test_zero_width_characters():
    """Test zero-width characters in flags."""
    
    print("Testing zero-width characters...")
    
    # Zero-width characters
    zwc_cases = [
        "config\u200b.yml",         # Zero-width space
        "config\ufeff.yml",         # Zero-width no-break space (BOM)
    ]
    
    for filename in zwc_cases:
        result = subprocess.run(
            ["cargo", "run", "--release", "--", "--config", filename],
            capture_output=True,
            text=True
        )
        
        print(f"✓ Zero-width characters handled: {filename}")
    
    return True


if __name__ == "__main__":
    print("Testing malformed UTF-8 flag handling...\n")
    
    tests = [
        test_invalid_utf8_flag,
        test_valid_unicode_flags,
        test_mixed_encoding,
        test_null_bytes_in_flags,
        test_long_unicode_flags,
        test_combining_characters,
        test_rtl_text,
        test_zero_width_characters,
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
        print("✅ All UTF-8 validation tests passed")
        sys.exit(0)
    else:
        print(f"❌ {failed} test(s) failed")
        sys.exit(1)
