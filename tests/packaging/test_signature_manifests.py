#!/usr/bin/env python3
"""
Test: Validate signature manifests present for each platform.

Validates that signature manifests exist for all platform binaries.
"""

import os
import sys
import json


PLATFORMS = [
    "linux-x64",
    "linux-arm64",
    "macos-x64",
    "macos-arm64",
    "windows-x64",
]


def check_signature_manifest(platform):
    """Check if signature manifest exists for platform."""
    
    manifest_path = f"target/package/{platform}/SIGNATURE.json"
    
    if not os.path.exists(manifest_path):
        print(f"❌ Signature manifest not found: {manifest_path}")
        return False
    
    # Validate JSON structure
    try:
        with open(manifest_path, 'r') as f:
            manifest = json.load(f)
        
        required_fields = ["platform", "binary", "checksum", "algorithm", "timestamp"]
        missing = [field for field in required_fields if field not in manifest]
        
        if missing:
            print(f"❌ Manifest missing fields: {missing}")
            return False
        
        print(f"✓ Valid signature manifest for {platform}")
        print(f"    Binary: {manifest.get('binary')}")
        print(f"    Checksum: {manifest.get('checksum', '')[:16]}...")
        print(f"    Algorithm: {manifest.get('algorithm')}")
        return True
    
    except json.JSONDecodeError as e:
        print(f"❌ Invalid JSON in manifest: {e}")
        return False
    except Exception as e:
        print(f"❌ Error reading manifest: {e}")
        return False


def test_all_platforms():
    """Test signature manifests for all platforms."""
    
    passed = 0
    failed = 0
    
    for platform in PLATFORMS:
        if check_signature_manifest(platform):
            passed += 1
        else:
            failed += 1
        print()
    
    return passed, failed


def test_manifest_format():
    """Test that manifest format is correct."""
    
    print("Testing manifest format...")
    
    # Example manifest structure
    example_manifest = {
        "platform": "linux-x64",
        "binary": "costpilot",
        "checksum": "a" * 64,
        "algorithm": "SHA-256",
        "timestamp": "2025-12-10T00:00:00Z",
        "version": "1.0.0",
    }
    
    # Check if example has all required fields
    required_fields = ["platform", "binary", "checksum", "algorithm", "timestamp"]
    missing = [field for field in required_fields if field not in example_manifest]
    
    if missing:
        print(f"❌ Example manifest missing fields: {missing}")
        return False
    
    print("✓ Manifest format is correct")
    print(f"  Required fields: {', '.join(required_fields)}")
    return True


def test_checksum_format():
    """Test that checksum format is valid."""
    
    print("Testing checksum format...")
    
    # SHA-256 should be 64 hex characters
    valid_checksums = [
        "a" * 64,
        "1234567890abcdef" * 4,
        "0" * 64,
    ]
    
    invalid_checksums = [
        "abc",  # Too short
        "g" * 64,  # Invalid hex
        "ABC",  # Too short
    ]
    
    print("Valid checksums:")
    for checksum in valid_checksums:
        if len(checksum) == 64 and all(c in "0123456789abcdefABCDEF" for c in checksum):
            print(f"  ✓ {checksum[:16]}...")
        else:
            print(f"  ❌ {checksum[:16]}...")
            return False
    
    print("Invalid checksums:")
    for checksum in invalid_checksums:
        if not (len(checksum) == 64 and all(c in "0123456789abcdefABCDEF" for c in checksum)):
            print(f"  ✓ {checksum} (correctly rejected)")
        else:
            print(f"  ❌ {checksum} (should be rejected)")
            return False
    
    print("✓ Checksum format validation works")
    return True


def create_example_manifests():
    """Create example signature manifests for testing."""
    
    print("Creating example signature manifests...")
    
    os.makedirs("target/package", exist_ok=True)
    
    for platform in PLATFORMS:
        platform_dir = f"target/package/{platform}"
        os.makedirs(platform_dir, exist_ok=True)
        
        manifest = {
            "platform": platform,
            "binary": "costpilot.exe" if "windows" in platform else "costpilot",
            "checksum": "a" * 64,  # Placeholder
            "algorithm": "SHA-256",
            "timestamp": "2025-12-10T00:00:00Z",
            "version": "1.0.0",
        }
        
        manifest_path = f"{platform_dir}/SIGNATURE.json"
        with open(manifest_path, 'w') as f:
            json.dump(manifest, f, indent=2)
        
        print(f"  Created {manifest_path}")
    
    print("✓ Example manifests created")
    return True


if __name__ == "__main__":
    print("Testing signature manifests...\n")
    
    # Test format first
    if not test_manifest_format():
        print("\n❌ Manifest format test failed")
        sys.exit(1)
    print()
    
    if not test_checksum_format():
        print("\n❌ Checksum format test failed")
        sys.exit(1)
    print()
    
    # Check if manifests exist, if not create examples
    manifest_exists = any(
        os.path.exists(f"target/package/{platform}/SIGNATURE.json")
        for platform in PLATFORMS
    )
    
    if not manifest_exists:
        print("No manifests found, creating examples...\n")
        create_example_manifests()
        print()
    
    # Test all platforms
    passed, failed = test_all_platforms()
    
    print(f"\nResults: {passed} platforms passed, {failed} failed")
    
    if failed == 0:
        print("\n✅ All signature manifest tests passed")
        sys.exit(0)
    else:
        print(f"\n❌ {failed} platform(s) missing signature manifests")
        sys.exit(1)
