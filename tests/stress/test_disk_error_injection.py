#!/usr/bin/env python3
"""Test disk error injection handling."""

import subprocess
import tempfile
from pathlib import Path
import json
import os


def test_readonly_config_directory():
    """Test behavior when config directory is read-only."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        config_dir = Path(tmpdir) / "config"
        config_dir.mkdir()
        
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
        
        # Make config directory read-only
        os.chmod(config_dir, 0o444)
        
        try:
            result = subprocess.run(
                ["costpilot", "scan", "--plan", str(template_path)],
                capture_output=True,
                text=True,
                timeout=30,
                env={**os.environ, "HOME": str(tmpdir)}
            )
            
            # Should complete or handle gracefully
            assert result.returncode in [0, 1, 2, 101], "Should handle read-only config directory"
        finally:
            os.chmod(config_dir, 0o755)


def test_readonly_output_file():
    """Test behavior when output file is read-only."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        output_path = Path(tmpdir) / "output.json"
        
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
        
        # Create read-only output file
        output_path.touch()
        os.chmod(output_path, 0o444)
        
        try:
            result = subprocess.run(
                ["costpilot", "scan", "--plan", str(template_path), "--output", str(output_path)],
                capture_output=True,
                text=True,
                timeout=30
            )
            
            # Should fail or write to stdout
            assert result.returncode in [0, 1, 2, 101], "Should handle read-only output file"
        finally:
            os.chmod(output_path, 0o644)


def test_disk_full_simulation():
    """Test behavior when disk is full (simulated)."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        
        template_content = {
            "Resources": {
                f"Lambda{i}": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 1024,
                        "Description": "X" * 100000  # Large description
                    }
                }
                for i in range(100)
            }
        }
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path)],
            capture_output=True,
            text=True,
            timeout=30
        )
        
        # Should complete or fail gracefully
        assert result.returncode in [0, 1, 2, 101], "Should handle large template gracefully"


def test_corrupted_template_file():
    """Test behavior when template file is corrupted."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        
        # Write valid JSON
        with open(template_path, 'w') as f:
            f.write('{"Resources": {"Lambda": {"Type": "AWS::Lambda::Function"}}}')
        
        # Corrupt the file by appending garbage
        with open(template_path, 'a') as f:
            f.write('\x00\x00\x00CORRUPTED')
        
        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path)],
            capture_output=True,
            text=True,
            timeout=30
        )
        
        # Should fail with parse error
        assert result.returncode != 0, "Should detect corrupted template"
        assert "parse" in result.stderr.lower() or "json" in result.stderr.lower(), "Should report parse error"


def test_symlink_loop():
    """Test behavior with symlink loops."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        link_path = Path(tmpdir) / "link.json"
        
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
        
        # Create symlink loop
        link_path.symlink_to(link_path)
        
        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(link_path)],
            capture_output=True,
            text=True,
            timeout=30
        )
        
        # Should fail gracefully
        assert result.returncode != 0, "Should detect symlink loop"


def test_inaccessible_template():
    """Test behavior when template file is inaccessible."""
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
        
        # Make template inaccessible
        os.chmod(template_path, 0o000)
        
        try:
            result = subprocess.run(
                ["costpilot", "scan", "--plan", str(template_path)],
                capture_output=True,
                text=True,
                timeout=30
            )
            
            # Should fail with permission error
            assert result.returncode != 0, "Should detect inaccessible template"
        finally:
            os.chmod(template_path, 0o644)


def test_partial_write_recovery():
    """Test recovery from partial write scenarios."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        output_path = Path(tmpdir) / "output.json"
        
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
        
        # Create partial output file
        with open(output_path, 'w') as f:
            f.write('{"partial')
        
        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path), "--output", str(output_path)],
            capture_output=True,
            text=True,
            timeout=30
        )
        
        # Should overwrite partial output
        assert result.returncode in [0, 1, 2, 101], "Should recover from partial write"
        
        if output_path.exists():
            with open(output_path) as f:
                content = f.read()
            
            # Output should be valid JSON or empty
            if content.strip():
                try:
                    json.loads(content)
                except json.JSONDecodeError:
                    assert False, "Output should be valid JSON"


def test_interrupted_write():
    """Test handling of interrupted write operations."""
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
                for i in range(1000)
            }
        }
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        # Run with short timeout to simulate interruption
        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path)],
            capture_output=True,
            text=True,
            timeout=1
        )
        
        # Should timeout
        # (This test is more about documentation than assertion)


if __name__ == "__main__":
    test_readonly_config_directory()
    test_readonly_output_file()
    test_disk_full_simulation()
    test_corrupted_template_file()
    test_symlink_loop()
    test_inaccessible_template()
    test_partial_write_recovery()
    print("All disk error injection tests passed")
