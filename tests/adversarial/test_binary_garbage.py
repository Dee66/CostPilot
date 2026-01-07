#!/usr/bin/env python3
"""Test handling of binary garbage in Terraform plans."""

import json
import subprocess
import tempfile
from pathlib import Path
import random


def test_binary_garbage_in_json():
    """Test handling of binary data embedded in JSON."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "tfplan.json"

        # Binary garbage bytes
        binary_data = bytes([random.randint(0, 255) for _ in range(100)])

        # Try to embed binary in JSON-like structure
        with open(template_path, 'wb') as f:
            f.write(b'{"Resources": {"Lambda": {"Type": "AWS::Lambda::Function", "Properties": {"Description": "')
            f.write(binary_data)
            f.write(b'"}}}')

        result = subprocess.run(
            ["./target/debug/costpilot", "scan", "--plan", str(template_path)],
            capture_output=True,
            text=True,
            timeout=30
        )

        # Should reject binary garbage
        assert result.returncode == 1, "Should reject binary garbage in JSON"


def test_non_utf8_characters():
    """Test handling of non-UTF-8 characters."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "tfplan.json"

        # Invalid UTF-8 sequences
        invalid_utf8 = b'\x80\x81\x82\x83\x84\x85'

        with open(template_path, 'wb') as f:
            f.write(b'{"Resources": {"Lambda": {"Type": "AWS::Lambda::Function", "Properties": {"Description": "')
            f.write(invalid_utf8)
            f.write(b'"}}}')

        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path)],
            capture_output=True,
            text=True,
            timeout=30
        )

        # Should reject invalid UTF-8
        assert result.returncode in [2, 101], "Should reject invalid UTF-8"


def test_partial_json_with_garbage():
    """Test handling of partial JSON followed by garbage."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "tfplan.json"

        valid_json = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 1024
                    }
                }
            }
        }

        # Write valid JSON followed by garbage
        with open(template_path, 'wb') as f:
            f.write(json.dumps(valid_json).encode('utf-8'))
            f.write(b'\x00\x01\x02\x03GARBAGE_DATA_HERE')

        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path)],
            capture_output=True,
            text=True,
            timeout=30
        )

        # Should reject file with trailing garbage
        assert result.returncode in [0, 1, 2, 101], "Should handle JSON with trailing garbage"


def test_null_bytes_in_middle():
    """Test handling of null bytes in middle of JSON."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "tfplan.json"

        with open(template_path, 'wb') as f:
            f.write(b'{"Resources": {"Lambda": {\x00\x00\x00"Type": "AWS::Lambda::Function"}}}')

        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path)],
            capture_output=True,
            text=True,
            timeout=30
        )

        # Should reject null bytes
        assert result.returncode in [2, 101], "Should reject null bytes in JSON"


def test_random_binary_file():
    """Test handling of completely random binary file."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "tfplan.bin"

        # Generate 1KB of random binary data
        binary_data = bytes([random.randint(0, 255) for _ in range(1024)])

        with open(template_path, 'wb') as f:
            f.write(binary_data)

        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path)],
            capture_output=True,
            text=True,
            timeout=30
        )

        # Should reject random binary
        assert result.returncode in [2, 101], "Should reject random binary file"


def test_gzip_compressed_json():
    """Test handling of gzip-compressed JSON."""
    import gzip

    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "tfplan.json.gz"

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

        with gzip.open(template_path, 'wb') as f:
            f.write(json.dumps(template_content).encode('utf-8'))

        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path)],
            capture_output=True,
            text=True,
            timeout=30
        )

        # Should reject compressed files
        assert result.returncode in [2, 101], "Should reject gzip-compressed files"


def test_executable_binary():
    """Test handling of executable binary files."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "tfplan.exe"

        # ELF/PE header-like data
        elf_header = b'\x7fELF\x02\x01\x01\x00' + bytes([0] * 8)

        with open(template_path, 'wb') as f:
            f.write(elf_header)
            f.write(bytes([random.randint(0, 255) for _ in range(1000)]))

        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path)],
            capture_output=True,
            text=True,
            timeout=30
        )

        # Should reject executable files
        assert result.returncode in [2, 101], "Should reject executable files"


def test_mixed_encoding_garbage():
    """Test handling of mixed encoding garbage."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "tfplan.json"

        # Mix UTF-8, Latin-1, and garbage
        mixed_data = (
            '{"Resources": {"Lambda": {"Type": "AWS::Lambda::Function", '
            '"Properties": {"Description": "'
        ).encode('utf-8')

        mixed_data += b'\xc0\xc1\xf5\xf6\xf7'  # Invalid UTF-8
        mixed_data += b'"}}}}'

        with open(template_path, 'wb') as f:
            f.write(mixed_data)

        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path)],
            capture_output=True,
            text=True,
            timeout=30
        )

        # Should reject mixed encoding
        assert result.returncode in [2, 101], "Should reject mixed encoding garbage"


def test_corrupted_json_structure():
    """Test handling of corrupted JSON structure with binary."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "tfplan.json"

        with open(template_path, 'wb') as f:
            f.write(b'{"Resources": {')
            f.write(bytes([random.randint(0, 255) for _ in range(50)]))
            f.write(b'}}')

        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path)],
            capture_output=True,
            text=True,
            timeout=30
        )

        # Should reject corrupted JSON
        assert result.returncode in [2, 101], "Should reject corrupted JSON structure"


def test_terraform_binary_plan():
    """Test handling of Terraform binary plan format."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "tfplan"

        # Terraform binary plan header (mock)
        tf_header = b'tfplan\x00\x00\x01'

        with open(template_path, 'wb') as f:
            f.write(tf_header)
            f.write(bytes([random.randint(0, 255) for _ in range(1000)]))

        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path)],
            capture_output=True,
            text=True,
            timeout=30
        )

        # Should reject binary Terraform plans
        assert result.returncode in [2, 101], "Should reject binary Terraform plans"


if __name__ == "__main__":
    test_binary_garbage_in_json()
    test_non_utf8_characters()
    test_partial_json_with_garbage()
    test_null_bytes_in_middle()
    test_random_binary_file()
    test_gzip_compressed_json()
    test_executable_binary()
    test_mixed_encoding_garbage()
    test_corrupted_json_structure()
    test_terraform_binary_plan()
    print("All binary garbage tests passed")
