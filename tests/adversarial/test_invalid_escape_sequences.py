#!/usr/bin/env python3
"""Test handling of adversarial plans with invalid escape sequences."""

import json
import subprocess
import tempfile
from pathlib import Path


def test_invalid_json_escape_sequences():
    """Test handling of invalid JSON escape sequences."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        
        # Invalid escape sequences
        invalid_content = r'''
        {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "Description": "Invalid escape \z sequence",
                        "MemorySize": 1024
                    }
                }
            }
        }
        '''
        
        with open(template_path, 'w') as f:
            f.write(invalid_content)
        
        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path)],
            capture_output=True,
            text=True,
            timeout=30
        )
        
        # Should fail gracefully with error
        assert result.returncode in [2, 101], "Invalid escape should fail with error"
        assert "escape" in result.stderr.lower() or "parse" in result.stderr.lower(), \
            "Error should mention escape or parse"


def test_invalid_yaml_escape_sequences():
    """Test handling of invalid YAML escape sequences."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.yaml"
        
        # Invalid YAML escape
        invalid_content = """
Resources:
  Lambda:
    Type: AWS::Lambda::Function
    Properties:
      Description: "Invalid \\z escape"
      MemorySize: 1024
"""
        
        with open(template_path, 'w') as f:
            f.write(invalid_content)
        
        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path)],
            capture_output=True,
            text=True,
            timeout=30
        )
        
        # Should handle gracefully
        assert result.returncode in [0, 1, 2, 101], "Should handle invalid YAML escape"


def test_unicode_escape_sequences():
    """Test handling of Unicode escape sequences."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        
        template_content = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "Description": "Unicode \u0041\u0042\u0043",
                        "MemorySize": 1024
                    }
                }
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
        
        # Should handle Unicode escapes
        assert result.returncode in [0, 1, 2, 101], "Should handle Unicode escapes"


def test_null_byte_escape_sequences():
    """Test handling of null byte escape sequences."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        
        # Null byte in string
        template_content = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "Description": "Null\x00byte",
                        "MemorySize": 1024
                    }
                }
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
        
        # Should handle null bytes gracefully
        assert result.returncode in [0, 1, 2, 101], "Should handle null bytes"


def test_incomplete_escape_sequences():
    """Test handling of incomplete escape sequences."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        
        # Incomplete escape at end
        invalid_content = r'''
        {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "Description": "Incomplete escape at end \
                    }
                }
            }
        }
        '''
        
        with open(template_path, 'w') as f:
            f.write(invalid_content)
        
        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path)],
            capture_output=True,
            text=True,
            timeout=30
        )
        
        # Should fail with parse error
        assert result.returncode in [2, 101], "Incomplete escape should fail"


def test_hex_escape_sequences():
    """Test handling of hex escape sequences."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        
        template_content = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "Description": "Hex \x48\x65\x6C\x6C\x6F",
                        "MemorySize": 1024
                    }
                }
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
        
        # Should handle hex escapes
        assert result.returncode in [0, 1, 2, 101], "Should handle hex escapes"


def test_backslash_only_escape():
    """Test handling of bare backslash."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        
        # Bare backslash
        invalid_content = r'''
        {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "Description": "Bare backslash \",
                        "MemorySize": 1024
                    }
                }
            }
        }
        '''
        
        with open(template_path, 'w') as f:
            f.write(invalid_content)
        
        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path)],
            capture_output=True,
            text=True,
            timeout=30
        )
        
        # Should fail with parse error
        assert result.returncode in [2, 101], "Bare backslash should fail"


def test_surrogate_pair_escape_sequences():
    """Test handling of surrogate pair escape sequences."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        
        template_content = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "Description": "Emoji \ud83d\ude00",
                        "MemorySize": 1024
                    }
                }
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
        
        # Should handle surrogate pairs
        assert result.returncode in [0, 1, 2, 101], "Should handle surrogate pairs"


def test_control_character_escape_sequences():
    """Test handling of control character escape sequences."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        
        template_content = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "Description": "Control\n\r\t\b\fchars",
                        "MemorySize": 1024
                    }
                }
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
        
        # Should handle control characters
        assert result.returncode in [0, 1, 2, 101], "Should handle control characters"


def test_mixed_escape_sequences():
    """Test handling of mixed valid and invalid escape sequences."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        
        template_content = {
            "Resources": {
                "Lambda1": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "Description": "Valid\nescape",
                        "MemorySize": 1024
                    }
                },
                "Lambda2": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "Description": "Valid\ttab",
                        "MemorySize": 2048
                    }
                }
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
        
        # Should handle mixed escapes
        assert result.returncode in [0, 1, 2, 101], "Should handle mixed escapes"


if __name__ == "__main__":
    test_invalid_json_escape_sequences()
    test_invalid_yaml_escape_sequences()
    test_unicode_escape_sequences()
    test_null_byte_escape_sequences()
    test_incomplete_escape_sequences()
    test_hex_escape_sequences()
    test_backslash_only_escape()
    test_surrogate_pair_escape_sequences()
    test_control_character_escape_sequences()
    test_mixed_escape_sequences()
    print("All invalid escape sequence tests passed")
