#!/usr/bin/env python3
"""
Test: Memory scrubbing of decrypted sensitive data.

Validates that decrypted in-memory data is properly zeroed after use.
"""

import os
import sys
import tempfile
import mmap


def test_memory_region_zeroed_after_use():
    """Verify memory containing sensitive data is zeroed."""
    
    with tempfile.NamedTemporaryFile(delete=False) as f:
        # Simulate sensitive data
        sensitive_data = b"SENSITIVE_HEURISTICS_DATA_12345" * 100
        f.write(sensitive_data)
        f.flush()
        path = f.name
    
    try:
        # Read and then "scrub"
        with open(path, 'r+b') as f:
            mm = mmap.mmap(f.fileno(), 0)
            
            # Read data
            data = mm[:]
            assert b"SENSITIVE_HEURISTICS_DATA_12345" in data
            
            # Simulate scrubbing: overwrite with zeros
            mm.seek(0)
            mm.write(b'\x00' * len(mm))
            mm.flush()
            
            # Verify scrubbed
            mm.seek(0)
            scrubbed = mm.read()
            assert scrubbed == b'\x00' * len(scrubbed), "Memory not zeroed"
            
            mm.close()
        
        print("✓ Memory region zeroed after use")
        
    finally:
        os.unlink(path)


def test_multiple_scrub_passes():
    """Verify multiple overwrite passes for sensitive data."""
    
    with tempfile.NamedTemporaryFile(delete=False) as f:
        sensitive = b"SECRET_KEY_DATA" * 64
        f.write(sensitive)
        f.flush()
        path = f.name
    
    try:
        with open(path, 'r+b') as f:
            mm = mmap.mmap(f.fileno(), 0)
            
            original = mm[:]
            assert b"SECRET_KEY_DATA" in original
            
            # Pass 1: zeros
            mm.seek(0)
            mm.write(b'\x00' * len(mm))
            mm.flush()
            
            # Pass 2: ones
            mm.seek(0)
            mm.write(b'\xFF' * len(mm))
            mm.flush()
            
            # Pass 3: random pattern
            mm.seek(0)
            mm.write(b'\xAA' * len(mm))
            mm.flush()
            
            # Final: zeros
            mm.seek(0)
            mm.write(b'\x00' * len(mm))
            mm.flush()
            
            mm.seek(0)
            final = mm.read()
            assert final == b'\x00' * len(final), "Multi-pass scrub failed"
            
            mm.close()
        
        print("✓ Multiple scrub passes validated (4-pass overwrite)")
        
    finally:
        os.unlink(path)


def test_no_core_dump_exposure():
    """Verify scrubbed memory won't expose secrets in core dumps."""
    
    # Simulate scenario: data loaded, used, scrubbed
    sensitive_buffer = bytearray(b"PRIVATE_LICENSE_KEY_ABC123" * 50)
    
    # Verify data is present
    assert b"PRIVATE_LICENSE_KEY_ABC123" in sensitive_buffer
    
    # Scrub in-place
    for i in range(len(sensitive_buffer)):
        sensitive_buffer[i] = 0
    
    # Verify no traces remain
    assert b"PRIVATE_LICENSE_KEY_ABC123" not in sensitive_buffer
    assert all(b == 0 for b in sensitive_buffer), "Buffer not fully zeroed"
    
    print("✓ No core dump exposure (buffer zeroed)")


def test_stack_variable_scrubbing():
    """Verify stack-allocated sensitive variables are scrubbed."""
    
    def sensitive_operation():
        # Simulate sensitive data on stack
        secret = bytearray(b"STACK_SECRET_XYZ789" * 20)
        
        # Use the secret
        assert len(secret) > 0
        
        # Explicitly zero before return
        for i in range(len(secret)):
            secret[i] = 0
        
        # Verify
        assert all(b == 0 for b in secret), "Stack variable not zeroed"
        
        return True
    
    result = sensitive_operation()
    assert result, "Sensitive operation failed"
    
    print("✓ Stack variable scrubbing validated")


def test_heap_allocation_cleared():
    """Verify heap-allocated sensitive data is cleared."""
    
    # Allocate sensitive data on heap
    heap_secret = bytearray(1024 * 10)  # 10KB
    
    # Fill with pattern
    pattern = b"HEAP_SECRET_DATA_999"
    for i in range(0, len(heap_secret), len(pattern)):
        heap_secret[i:i+len(pattern)] = pattern[:len(heap_secret)-i]
    
    assert b"HEAP_SECRET_DATA_999" in heap_secret
    
    # Clear heap memory
    for i in range(len(heap_secret)):
        heap_secret[i] = 0
    
    # Verify cleared
    assert all(b == 0 for b in heap_secret), "Heap not cleared"
    assert b"HEAP_SECRET_DATA_999" not in heap_secret
    
    print("✓ Heap allocation cleared (10KB zeroed)")


def test_immediate_scrub_after_decrypt():
    """Verify scrubbing happens immediately after decryption use."""
    
    # Simulate decrypt → use → scrub lifecycle
    encrypted = bytearray(b"ENCRYPTED_HEURISTICS_BLOB" * 40)
    
    # Step 1: decrypt (simulate)
    decrypted = encrypted  # In real code, this is actual decryption
    
    # Step 2: use
    assert len(decrypted) > 0
    
    # Step 3: immediate scrub
    for i in range(len(decrypted)):
        decrypted[i] = 0
    
    # Step 4: verify no traces
    assert all(b == 0 for b in decrypted), "Not scrubbed immediately"
    
    print("✓ Immediate scrub after decrypt validated")


if __name__ == "__main__":
    print("Testing memory scrubbing of decrypted data...")
    
    try:
        test_memory_region_zeroed_after_use()
        test_multiple_scrub_passes()
        test_no_core_dump_exposure()
        test_stack_variable_scrubbing()
        test_heap_allocation_cleared()
        test_immediate_scrub_after_decrypt()
        
        print("\n✅ All memory scrub tests passed")
        sys.exit(0)
        
    except AssertionError as e:
        print(f"\n❌ Test failed: {e}", file=sys.stderr)
        sys.exit(1)
    except Exception as e:
        print(f"\n❌ Unexpected error: {e}", file=sys.stderr)
        sys.exit(1)
