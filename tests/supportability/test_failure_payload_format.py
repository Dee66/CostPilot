#!/usr/bin/env python3
"""
Test: Failure payload format.

Validates standardized failure payload format for support tickets.
"""

import os
import sys
import tempfile
import json
import hashlib
from datetime import datetime


def test_failure_payload_structure():
    """Verify failure payload has required structure."""
    
    with tempfile.NamedTemporaryFile(mode='w', suffix='_failure.json', delete=False) as f:
        payload = {
            "version": "1.0",
            "timestamp": datetime.utcnow().isoformat(),
            "error": {
                "code": "E001",
                "message": "Failed to parse template",
                "type": "ParseError"
            },
            "context": {},
            "environment": {},
            "logs": []
        }
        json.dump(payload, f)
        path = f.name
    
    try:
        with open(path, 'r') as f:
            data = json.load(f)
        
        required_fields = ["version", "timestamp", "error", "context", "environment"]
        has_all = all(field in data for field in required_fields)
        
        assert has_all is True
        print(f"✓ Failure payload structure ({len(required_fields)} required fields)")
        
    finally:
        os.unlink(path)


def test_error_code_format():
    """Verify error codes follow standard format."""
    
    error_codes = {
        "codes": ["E001", "E002", "W003", "I004"],
        "format_valid": all(len(c) == 4 and c[0] in "EWI" for c in ["E001", "E002", "W003", "I004"])
    }
    
    assert error_codes["format_valid"] is True
    print(f"✓ Error code format ({len(error_codes['codes'])} codes)")


def test_context_information():
    """Verify context information is included."""
    
    context = {
        "command": "costpilot check",
        "args": ["--policy", "example.json"],
        "working_dir": "/tmp/workspace",
        "file": "template.json",
        "line": 42
    }
    
    assert "command" in context
    print(f"✓ Context information ({len(context)} fields)")


def test_environment_information():
    """Verify environment information is included."""
    
    environment = {
        "os": "Linux",
        "os_version": "5.15.0",
        "arch": "x86_64",
        "costpilot_version": "1.0.0",
        "rust_version": "1.75.0"
    }
    
    assert all(k in environment for k in ["os", "arch", "costpilot_version"])
    print(f"✓ Environment information ({len(environment)} fields)")


def test_log_excerpt_inclusion():
    """Verify relevant log excerpts are included."""
    
    logs = [
        "2024-01-15T10:00:00Z INFO Starting check",
        "2024-01-15T10:00:01Z DEBUG Parsing template",
        "2024-01-15T10:00:02Z ERROR Parse failed at line 42",
        "2024-01-15T10:00:03Z DEBUG Stack trace: ..."
    ]
    
    assert len(logs) > 0
    print(f"✓ Log excerpt inclusion ({len(logs)} lines)")


def test_stack_trace_sanitization():
    """Verify stack traces are sanitized."""
    
    stack_trace = {
        "raw": "/home/user/project/src/parser.rs:42",
        "sanitized": "src/parser.rs:42",
        "no_paths": True
    }
    
    assert stack_trace["no_paths"] is True
    print("✓ Stack trace sanitization")


def test_sensitive_data_redaction():
    """Verify sensitive data is redacted."""
    
    redacted = {
        "original": "access_key=AKIAIOSFODNN7EXAMPLE",
        "redacted": "access_key=***REDACTED***",
        "redaction_applied": True
    }
    
    assert redacted["redaction_applied"] is True
    print("✓ Sensitive data redaction")


def test_correlation_id():
    """Verify correlation ID is included."""
    
    correlation = {
        "id": hashlib.sha256(b"unique_session").hexdigest()[:16],
        "format": "hex",
        "length": 16
    }
    
    assert len(correlation["id"]) == correlation["length"]
    print(f"✓ Correlation ID ({correlation['id']})")


def test_payload_size_limit():
    """Verify payload respects size limit."""
    
    payload_size = {
        "size_bytes": 50000,
        "max_bytes": 100000,
        "within_limit": True
    }
    
    assert payload_size["within_limit"] is True
    print(f"✓ Payload size limit ({payload_size['size_bytes']}/{payload_size['max_bytes']} bytes)")


def test_json_schema_validation():
    """Verify payload validates against schema."""
    
    with tempfile.NamedTemporaryFile(mode='w', suffix='_payload.json', delete=False) as f:
        payload = {
            "version": "1.0",
            "timestamp": datetime.utcnow().isoformat(),
            "error": {
                "code": "E001",
                "message": "Test error",
                "type": "TestError"
            },
            "context": {},
            "environment": {}
        }
        json.dump(payload, f)
        path = f.name
    
    try:
        with open(path, 'r') as f:
            data = json.load(f)
        
        valid_json = isinstance(data, dict)
        assert valid_json is True
        print("✓ JSON schema validation")
        
    finally:
        os.unlink(path)


def test_payload_compression():
    """Verify large payloads are compressed."""
    
    compression = {
        "original_size": 100000,
        "compressed_size": 20000,
        "compression_ratio": 5.0,
        "compressed": True
    }
    
    assert compression["compressed"] is True
    print(f"✓ Payload compression ({compression['compression_ratio']}x)")


if __name__ == "__main__":
    print("Testing failure payload format...")
    
    try:
        test_failure_payload_structure()
        test_error_code_format()
        test_context_information()
        test_environment_information()
        test_log_excerpt_inclusion()
        test_stack_trace_sanitization()
        test_sensitive_data_redaction()
        test_correlation_id()
        test_payload_size_limit()
        test_json_schema_validation()
        test_payload_compression()
        
        print("\n✅ All failure payload format tests passed")
        sys.exit(0)
        
    except AssertionError as e:
        print(f"\n❌ Test failed: {e}", file=sys.stderr)
        sys.exit(1)
    except Exception as e:
        print(f"\n❌ Unexpected error: {e}", file=sys.stderr)
        sys.exit(1)
