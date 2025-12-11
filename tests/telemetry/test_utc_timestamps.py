#!/usr/bin/env python3
"""Test that logs use UTC timestamps."""

import json
import re
import subprocess
import tempfile
from datetime import datetime, timezone
from pathlib import Path


def test_utc_timestamps_in_logs():
    """Test that log timestamps are in UTC."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        
        template_content = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 1024
                    }
                }
            }
        }
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path), "--verbose"],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        combined_output = result.stdout + result.stderr
        
        # Look for timestamp patterns
        # ISO 8601 with Z suffix: 2024-01-15T10:30:45Z
        # ISO 8601 with +00:00: 2024-01-15T10:30:45+00:00
        utc_patterns = [
            r"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z",
            r"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\+00:00",
            r"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.?\d*Z"
        ]
        
        found_utc = False
        for pattern in utc_patterns:
            if re.search(pattern, combined_output):
                found_utc = True
                break
        
        # If timestamps are present, they should be UTC
        if found_utc:
            print("UTC timestamps found in output")
        
        # Should not have timezone offsets other than +00:00 or Z
        non_utc_pattern = r"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}[+-](?!00:00)\d{2}:\d{2}"
        non_utc_matches = re.findall(non_utc_pattern, combined_output)
        assert len(non_utc_matches) == 0, f"Non-UTC timestamps found: {non_utc_matches}"


def test_metadata_timestamps_utc():
    """Test that metadata timestamps are in UTC."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        baseline_path = Path(tmpdir) / "baseline.json"
        
        template_content = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 1024
                    }
                }
            }
        }
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        result = subprocess.run(
            ["costpilot", "baseline", "generate", "--plan", str(template_path), "--output", str(baseline_path)],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        # Check baseline file for UTC timestamps
        if baseline_path.exists():
            with open(baseline_path) as f:
                baseline_data = json.load(f)
            
            # Check for timestamp fields
            def check_timestamps(obj, path=""):
                if isinstance(obj, dict):
                    for key, value in obj.items():
                        if key in ["created_at", "updated_at", "timestamp", "created", "updated"]:
                            if isinstance(value, str):
                                # Should be UTC format
                                assert value.endswith("Z") or "+00:00" in value, \
                                    f"Timestamp at {path}.{key} should be UTC: {value}"
                        check_timestamps(value, f"{path}.{key}")
                elif isinstance(obj, list):
                    for i, item in enumerate(obj):
                        check_timestamps(item, f"{path}[{i}]")
            
            check_timestamps(baseline_data)


def test_json_output_timestamps_utc():
    """Test that JSON output timestamps are in UTC."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        
        template_content = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 1024
                    }
                }
            }
        }
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path), "--format", "json"],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        # Parse JSON and check timestamps
        try:
            output_data = json.loads(result.stdout)
            
            def check_timestamps(obj):
                if isinstance(obj, dict):
                    for key, value in obj.items():
                        if "time" in key.lower() or "date" in key.lower():
                            if isinstance(value, str) and "T" in value:
                                # Should be UTC
                                assert value.endswith("Z") or "+00:00" in value, \
                                    f"Timestamp field '{key}' should be UTC: {value}"
                        check_timestamps(value)
                elif isinstance(obj, list):
                    for item in obj:
                        check_timestamps(item)
            
            check_timestamps(output_data)
        except json.JSONDecodeError:
            pass  # Not JSON output


def test_log_entry_timestamps_format():
    """Test that log entry timestamps follow consistent UTC format."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        
        template_content = {
            "Resources": {
                f"Lambda{i}": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 1024
                    }
                }
                for i in range(10)
            }
        }
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path), "--verbose"],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        combined_output = result.stdout + result.stderr
        
        # Find all timestamps
        timestamp_pattern = r"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d+)?(?:Z|[+-]\d{2}:\d{2})"
        timestamps = re.findall(timestamp_pattern, combined_output)
        
        # All timestamps should end with Z or +00:00
        for ts in timestamps:
            assert ts.endswith("Z") or "+00:00" in ts, f"Timestamp not UTC: {ts}"


def test_no_local_timezone_timestamps():
    """Test that logs don't contain local timezone timestamps."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        
        template_content = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 1024
                    }
                }
            }
        }
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path), "--debug"],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        combined_output = result.stdout + result.stderr
        
        # Check for non-UTC timezone offsets (e.g., -05:00, +08:00, etc.)
        # Allow +00:00 and Z
        local_tz_pattern = r"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}[+-](?!00:00)[0-9]{2}:[0-9]{2}"
        local_tz_matches = re.findall(local_tz_pattern, combined_output)
        
        assert len(local_tz_matches) == 0, f"Local timezone timestamps found: {local_tz_matches}"


def test_timestamp_consistency_across_runs():
    """Test that timestamp format is consistent across multiple runs."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        
        template_content = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 1024
                    }
                }
            }
        }
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        # Run multiple times
        timestamp_formats = set()
        
        for _ in range(3):
            result = subprocess.run(
                ["costpilot", "scan", "--plan", str(template_path), "--verbose"],
                capture_output=True,
                text=True,
                timeout=10
            )
            
            combined_output = result.stdout + result.stderr
            
            # Extract timestamp format
            timestamp_pattern = r"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d+)?(?:Z|[+-]\d{2}:\d{2})"
            timestamps = re.findall(timestamp_pattern, combined_output)
            
            for ts in timestamps:
                # Normalize to format pattern (remove actual values)
                if ts.endswith("Z"):
                    timestamp_formats.add("ISO8601_Z")
                elif "+00:00" in ts:
                    timestamp_formats.add("ISO8601_UTC")
        
        # Should use consistent format
        assert len(timestamp_formats) <= 1, f"Inconsistent timestamp formats: {timestamp_formats}"


def test_audit_log_timestamps_utc():
    """Test that audit log timestamps are in UTC."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        audit_path = Path(tmpdir) / "audit.log"
        
        template_content = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 1024
                    }
                }
            }
        }
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path), "--audit-log", str(audit_path)],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        # Check audit log for UTC timestamps
        if audit_path.exists():
            with open(audit_path) as f:
                audit_content = f.read()
            
            # Find timestamps in audit log
            timestamp_pattern = r"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d+)?(?:Z|[+-]\d{2}:\d{2})"
            timestamps = re.findall(timestamp_pattern, audit_content)
            
            # All should be UTC
            for ts in timestamps:
                assert ts.endswith("Z") or "+00:00" in ts, f"Audit log timestamp not UTC: {ts}"


def test_rfc3339_compliance():
    """Test that timestamps comply with RFC 3339 format."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        
        template_content = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 1024
                    }
                }
            }
        }
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path), "--verbose"],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        combined_output = result.stdout + result.stderr
        
        # RFC 3339 format: YYYY-MM-DDTHH:MM:SS[.fraction]Z or with offset
        rfc3339_pattern = r"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d+)?(?:Z|[+-]\d{2}:\d{2})"
        timestamps = re.findall(rfc3339_pattern, combined_output)
        
        # All timestamps should be valid RFC 3339
        for ts in timestamps:
            # Try to parse
            try:
                if ts.endswith("Z"):
                    parsed = datetime.fromisoformat(ts.replace("Z", "+00:00"))
                else:
                    parsed = datetime.fromisoformat(ts)
                
                # Should be UTC
                assert parsed.tzinfo is not None, f"Timestamp missing timezone: {ts}"
            except ValueError:
                assert False, f"Invalid RFC 3339 timestamp: {ts}"


if __name__ == "__main__":
    test_utc_timestamps_in_logs()
    test_metadata_timestamps_utc()
    test_json_output_timestamps_utc()
    test_log_entry_timestamps_format()
    test_no_local_timezone_timestamps()
    test_timestamp_consistency_across_runs()
    test_audit_log_timestamps_utc()
    test_rfc3339_compliance()
    print("All UTC timestamp validation tests passed")
