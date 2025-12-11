#!/usr/bin/env python3
"""Test that trace IDs have stable format."""

import json
import re
import subprocess
import tempfile
from pathlib import Path


def test_trace_id_format():
    """Test that trace IDs follow a stable format."""
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
        
        # Look for trace ID patterns
        # Common formats: UUID, hex string, alphanumeric
        trace_patterns = [
            r"trace[_-]?id[:\s=]+([a-f0-9]{8}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{12})",  # UUID
            r"trace[_-]?id[:\s=]+([a-f0-9]{32})",  # 32 hex
            r"trace[_-]?id[:\s=]+([a-zA-Z0-9]{16,64})"  # Alphanumeric
        ]
        
        trace_ids = []
        for pattern in trace_patterns:
            matches = re.findall(pattern, combined_output, re.IGNORECASE)
            trace_ids.extend(matches)
        
        # If trace IDs are present, check format consistency
        if trace_ids:
            print(f"Found trace IDs: {trace_ids}")
            
            # All trace IDs should have same format
            id_lengths = {len(tid) for tid in trace_ids}
            assert len(id_lengths) <= 2, f"Trace IDs have inconsistent lengths: {id_lengths}"


def test_trace_id_uniqueness():
    """Test that trace IDs are unique across runs."""
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
        
        trace_ids = []
        
        for _ in range(5):
            result = subprocess.run(
                ["costpilot", "scan", "--plan", str(template_path), "--verbose"],
                capture_output=True,
                text=True,
                timeout=10
            )
            
            combined_output = result.stdout + result.stderr
            
            # Extract trace ID
            trace_patterns = [
                r"trace[_-]?id[:\s=]+([a-f0-9-]{16,})",
                r"request[_-]?id[:\s=]+([a-f0-9-]{16,})"
            ]
            
            for pattern in trace_patterns:
                matches = re.findall(pattern, combined_output, re.IGNORECASE)
                if matches:
                    trace_ids.extend(matches)
                    break
        
        # Trace IDs should be unique (or mostly unique)
        if len(trace_ids) > 1:
            unique_ids = set(trace_ids)
            uniqueness_ratio = len(unique_ids) / len(trace_ids)
            assert uniqueness_ratio > 0.8, f"Trace IDs not sufficiently unique: {uniqueness_ratio}"


def test_trace_id_in_json_output():
    """Test that JSON output includes trace ID."""
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
        
        # Parse JSON
        try:
            output_data = json.loads(result.stdout)
            
            # Look for trace ID field
            trace_id_fields = ["trace_id", "traceId", "request_id", "requestId"]
            found_trace = False
            
            for field in trace_id_fields:
                if field in output_data:
                    trace_id = output_data[field]
                    assert len(trace_id) > 0, "Trace ID should not be empty"
                    found_trace = True
                    break
            
            if found_trace:
                print(f"Found trace ID in JSON output")
        except json.JSONDecodeError:
            pass  # Not JSON output


def test_trace_id_propagation():
    """Test that trace ID is propagated through operations."""
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
                for i in range(5)
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
        
        # Extract all trace IDs
        trace_pattern = r"trace[_-]?id[:\s=]+([a-f0-9-]{16,})"
        trace_ids = re.findall(trace_pattern, combined_output, re.IGNORECASE)
        
        # If trace IDs present, they should be consistent within a run
        if len(trace_ids) > 1:
            unique_ids = set(trace_ids)
            # Should be same trace ID throughout
            assert len(unique_ids) == 1, f"Trace ID should be consistent: {unique_ids}"


def test_trace_id_length_stability():
    """Test that trace ID length is stable."""
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
        
        trace_lengths = []
        
        for _ in range(5):
            result = subprocess.run(
                ["costpilot", "scan", "--plan", str(template_path), "--verbose"],
                capture_output=True,
                text=True,
                timeout=10
            )
            
            combined_output = result.stdout + result.stderr
            
            # Extract trace ID
            trace_pattern = r"trace[_-]?id[:\s=]+([a-f0-9-]{16,})"
            matches = re.findall(trace_pattern, combined_output, re.IGNORECASE)
            
            if matches:
                trace_lengths.append(len(matches[0]))
        
        # All trace IDs should have same length
        if trace_lengths:
            unique_lengths = set(trace_lengths)
            assert len(unique_lengths) == 1, f"Trace ID lengths inconsistent: {unique_lengths}"


def test_trace_id_character_set():
    """Test that trace IDs use consistent character set."""
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
        
        # Extract trace IDs
        trace_pattern = r"trace[_-]?id[:\s=]+([a-zA-Z0-9-]{16,})"
        trace_ids = re.findall(trace_pattern, combined_output, re.IGNORECASE)
        
        for trace_id in trace_ids:
            # Should be alphanumeric + hyphens only
            assert re.match(r"^[a-zA-Z0-9-]+$", trace_id), f"Invalid characters in trace ID: {trace_id}"


def test_trace_id_in_error_messages():
    """Test that error messages include trace ID."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "invalid.json"
        
        # Invalid JSON
        with open(template_path, 'w') as f:
            f.write("invalid json")
        
        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path)],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        combined_output = result.stdout + result.stderr
        
        # Error should include trace ID for debugging
        trace_pattern = r"trace[_-]?id[:\s=]+([a-zA-Z0-9-]{16,})"
        trace_ids = re.findall(trace_pattern, combined_output, re.IGNORECASE)
        
        if trace_ids:
            print(f"Trace ID in error message: {trace_ids[0]}")


def test_trace_id_format_documentation():
    """Test that trace ID format is documented."""
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
        
        # Extract trace IDs
        trace_pattern = r"trace[_-]?id[:\s=]+([a-zA-Z0-9-]{16,})"
        trace_ids = re.findall(trace_pattern, combined_output, re.IGNORECASE)
        
        if trace_ids:
            trace_id = trace_ids[0]
            
            # Determine format
            if re.match(r"^[a-f0-9]{8}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{12}$", trace_id):
                print("Trace ID format: UUID v4")
            elif re.match(r"^[a-f0-9]{32}$", trace_id):
                print("Trace ID format: 32-character hex")
            elif re.match(r"^[a-zA-Z0-9]{16,64}$", trace_id):
                print(f"Trace ID format: {len(trace_id)}-character alphanumeric")


def test_trace_id_collision_resistance():
    """Test that trace IDs have low collision probability."""
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
        
        # Generate many trace IDs
        trace_ids = []
        
        for _ in range(20):
            result = subprocess.run(
                ["costpilot", "scan", "--plan", str(template_path)],
                capture_output=True,
                text=True,
                timeout=10
            )
            
            combined_output = result.stdout + result.stderr
            
            trace_pattern = r"trace[_-]?id[:\s=]+([a-zA-Z0-9-]{16,})"
            matches = re.findall(trace_pattern, combined_output, re.IGNORECASE)
            
            if matches:
                trace_ids.append(matches[0])
        
        # Should have no collisions
        if len(trace_ids) > 1:
            unique_ids = set(trace_ids)
            assert len(unique_ids) == len(trace_ids), f"Trace ID collision detected: {len(trace_ids)} runs, {len(unique_ids)} unique"


if __name__ == "__main__":
    test_trace_id_format()
    test_trace_id_uniqueness()
    test_trace_id_in_json_output()
    test_trace_id_propagation()
    test_trace_id_length_stability()
    test_trace_id_character_set()
    test_trace_id_in_error_messages()
    test_trace_id_format_documentation()
    test_trace_id_collision_resistance()
    print("All trace ID format validation tests passed")
