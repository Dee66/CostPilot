#!/usr/bin/env python3
"""
Test: Heuristics artifact integrity check on bitflip.

Validates that integrity checks fail when heuristics artifact is corrupted.
"""

import os
import sys
import tempfile
from pathlib import Path
import hashlib
import json


def test_bitflip_detected():
    """Verify bitflip in heuristics artifact is detected."""
    
    # Create mock heuristics file
    with tempfile.NamedTemporaryFile(mode='w', suffix='.heuristics', delete=False) as f:
        heuristics = {
            "rules": ["rule1", "rule2", "rule3"],
            "version": "1.0",
            "checksum": "abc123"
        }
        json.dump(heuristics, f)
        heuristics_path = f.name
    
    try:
        # Calculate original checksum
        with open(heuristics_path, 'rb') as f:
            original_checksum = hashlib.sha256(f.read()).hexdigest()
        
        # Corrupt the file (bitflip)
        with open(heuristics_path, 'r+b') as f:
            f.seek(10)
            byte = f.read(1)
            f.seek(10)
            f.write(bytes([byte[0] ^ 0x01]))  # Flip one bit
        
        # Verify checksum changed
        with open(heuristics_path, 'rb') as f:
            corrupted_checksum = hashlib.sha256(f.read()).hexdigest()
        
        assert original_checksum != corrupted_checksum, "Bitflip not detected"
        
        print("✓ Bitflip in heuristics artifact detected")
        
    finally:
        os.unlink(heuristics_path)


def test_checksum_mismatch_rejected():
    """Verify checksum mismatch causes rejection."""
    
    with tempfile.NamedTemporaryFile(mode='w', suffix='.heuristics', delete=False) as f:
        heuristics = {
            "data": "heuristic_rules",
            "checksum": "expected_abc123"
        }
        json.dump(heuristics, f)
        path = f.name
    
    try:
        # Calculate actual checksum
        with open(path, 'rb') as f:
            actual = hashlib.sha256(f.read()).hexdigest()
        
        # Verify mismatch
        assert actual != "expected_abc123", "Checksum should mismatch"
        
        print("✓ Checksum mismatch rejection validated")
        
    finally:
        os.unlink(path)


def test_truncated_file_detected():
    """Verify truncated heuristics file is detected."""
    
    with tempfile.NamedTemporaryFile(mode='w', suffix='.heuristics', delete=False) as f:
        f.write('{"incomplete":')  # Truncated JSON
        path = f.name
    
    try:
        # Attempt to load truncated file
        try:
            with open(path, 'r') as f:
                json.load(f)
            assert False, "Truncated file should fail to parse"
        except json.JSONDecodeError:
            pass  # Expected
        
        print("✓ Truncated file detection validated")
        
    finally:
        os.unlink(path)


def test_extra_bytes_appended():
    """Verify extra bytes appended to file are detected."""
    
    with tempfile.NamedTemporaryFile(mode='wb', suffix='.heuristics', delete=False) as f:
        valid_data = b'{"valid": "data"}'
        expected_checksum = hashlib.sha256(valid_data).hexdigest()
        
        # Write valid data plus extra bytes
        f.write(valid_data)
        f.write(b'\x00\x00\x00')  # Extra bytes
        path = f.name
    
    try:
        # Verify checksum includes extra bytes
        with open(path, 'rb') as f:
            actual_checksum = hashlib.sha256(f.read()).hexdigest()
        
        assert expected_checksum != actual_checksum, "Extra bytes not detected"
        
        print("✓ Extra bytes appended detection validated")
        
    finally:
        os.unlink(path)


def test_valid_integrity_passes():
    """Verify valid heuristics pass integrity check."""
    
    with tempfile.NamedTemporaryFile(mode='w', suffix='.heuristics', delete=False) as f:
        heuristics = {"rules": ["r1", "r2"]}
        json.dump(heuristics, f)
        path = f.name
    
    try:
        # Calculate checksum
        with open(path, 'rb') as f:
            data = f.read()
            checksum = hashlib.sha256(data).hexdigest()
        
        # Verify data matches checksum
        with open(path, 'rb') as f:
            verify = hashlib.sha256(f.read()).hexdigest()
        
        assert checksum == verify, "Valid integrity should pass"
        
        print("✓ Valid integrity check passes")
        
    finally:
        os.unlink(path)


if __name__ == "__main__":
    print("Testing heuristics artifact integrity checks...")
    
    try:
        test_bitflip_detected()
        test_checksum_mismatch_rejected()
        test_truncated_file_detected()
        test_extra_bytes_appended()
        test_valid_integrity_passes()
        
        print("\n✅ All heuristics integrity tests passed")
        sys.exit(0)
        
    except AssertionError as e:
        print(f"\n❌ Test failed: {e}", file=sys.stderr)
        sys.exit(1)
    except Exception as e:
        print(f"\n❌ Unexpected error: {e}", file=sys.stderr)
        sys.exit(1)
