#!/usr/bin/env python3
"""Test HCL comment fuzzing."""

import random
import string
import subprocess
import tempfile
from pathlib import Path


def generate_random_string(length):
    """Generate random string."""
    return ''.join(random.choices(string.ascii_letters + string.digits + string.punctuation + ' ', k=length))


def test_hcl_single_line_comment_fuzzing():
    """Fuzz HCL single-line comments."""
    with tempfile.TemporaryDirectory() as tmpdir:
        hcl_path = Path(tmpdir) / "test.tf"

        # Generate random comments
        for _ in range(10):
            comment_content = generate_random_string(random.randint(1, 200))

            hcl_content = f"""
# {comment_content}
resource "aws_lambda_function" "test" {{
  function_name = "test"
  memory_size   = 1024
}}
"""

            with open(hcl_path, 'w') as f:
                f.write(hcl_content)

            # Should handle random comments
            result = subprocess.run(
                ["costpilot", "analyze", "--template", str(hcl_path)],
                capture_output=True,
                text=True,
                timeout=5
            )

            # Should not crash
            assert result.returncode in [0, 1, 2], "Should handle random single-line comments"


def test_hcl_multiline_comment_fuzzing():
    """Fuzz HCL multi-line comments."""
    with tempfile.TemporaryDirectory() as tmpdir:
        hcl_path = Path(tmpdir) / "test.tf"

        for _ in range(10):
            comment_lines = [generate_random_string(random.randint(10, 100)) for _ in range(random.randint(1, 20))]
            comment_content = '\n'.join(comment_lines)

            hcl_content = f"""
/*
{comment_content}
*/
resource "aws_lambda_function" "test" {{
  function_name = "test"
  memory_size   = 1024
}}
"""

            with open(hcl_path, 'w') as f:
                f.write(hcl_content)

            result = subprocess.run(
                ["costpilot", "analyze", "--template", str(hcl_path)],
                capture_output=True,
                text=True,
                timeout=5
            )

            assert result.returncode in [0, 1, 2], "Should handle random multi-line comments"


def test_hcl_nested_comment_fuzzing():
    """Fuzz nested HCL comments."""
    with tempfile.TemporaryDirectory() as tmpdir:
        hcl_path = Path(tmpdir) / "test.tf"

        # Nested comments (if supported)
        hcl_content = """
# /* nested comment */
resource "aws_lambda_function" "test" {
  function_name = "test"
  /* # another nested */ memory_size = 1024
}
"""

        with open(hcl_path, 'w') as f:
            f.write(hcl_content)

        result = subprocess.run(
            ["costpilot", "analyze", "--template", str(hcl_path)],
            capture_output=True,
            text=True,
            timeout=5
        )

        assert result.returncode in [0, 1, 2], "Should handle nested comments"


def test_hcl_comment_with_special_chars():
    """Fuzz HCL comments with special characters."""
    with tempfile.TemporaryDirectory() as tmpdir:
        hcl_path = Path(tmpdir) / "test.tf"

        special_chars = ['\\', '"', "'", '`', '\n', '\r', '\t', '\0', '\x00']

        for char in special_chars:
            hcl_content = f"""
# Comment with special char: {repr(char)}
resource "aws_lambda_function" "test" {{
  function_name = "test"
  memory_size   = 1024
}}
"""

            with open(hcl_path, 'w') as f:
                f.write(hcl_content)

            result = subprocess.run(
                ["costpilot", "analyze", "--template", str(hcl_path)],
                capture_output=True,
                text=True,
                timeout=5
            )

            assert result.returncode in [0, 1, 2], f"Should handle special char: {repr(char)}"


def test_hcl_comment_unicode_fuzzing():
    """Fuzz HCL comments with Unicode."""
    with tempfile.TemporaryDirectory() as tmpdir:
        hcl_path = Path(tmpdir) / "test.tf"

        unicode_strings = [
            "北京",
            "🚀🎉",
            "テスト",
            "مرحبا",
            "Привет"
        ]

        for unicode_str in unicode_strings:
            hcl_content = f"""
# Comment: {unicode_str}
resource "aws_lambda_function" "test" {{
  function_name = "test"
  memory_size   = 1024
}}
"""

            with open(hcl_path, 'w', encoding='utf-8') as f:
                f.write(hcl_content)

            result = subprocess.run(
                ["costpilot", "analyze", "--template", str(hcl_path)],
                capture_output=True,
                text=True,
                timeout=5
            )

            assert result.returncode in [0, 1, 2], f"Should handle Unicode: {unicode_str}"


def test_hcl_comment_length_fuzzing():
    """Fuzz HCL comments with varying lengths."""
    with tempfile.TemporaryDirectory() as tmpdir:
        hcl_path = Path(tmpdir) / "test.tf"

        lengths = [1, 10, 100, 1000, 10000]

        for length in lengths:
            comment_content = 'x' * length

            hcl_content = f"""
# {comment_content}
resource "aws_lambda_function" "test" {{
  function_name = "test"
  memory_size   = 1024
}}
"""

            with open(hcl_path, 'w') as f:
                f.write(hcl_content)

            result = subprocess.run(
                ["costpilot", "analyze", "--template", str(hcl_path)],
                capture_output=True,
                text=True,
                timeout=5
            )

            assert result.returncode in [0, 1, 2], f"Should handle comment length: {length}"


def test_hcl_unclosed_comment_fuzzing():
    """Fuzz unclosed HCL comments."""
    with tempfile.TemporaryDirectory() as tmpdir:
        hcl_path = Path(tmpdir) / "test.tf"

        # Unclosed multi-line comment
        hcl_content = """
/*
Unclosed comment
resource "aws_lambda_function" "test" {
  function_name = "test"
  memory_size   = 1024
}
"""

        with open(hcl_path, 'w') as f:
            f.write(hcl_content)

        result = subprocess.run(
            ["costpilot", "analyze", "--template", str(hcl_path)],
            capture_output=True,
            text=True,
            timeout=5
        )

        # Should handle gracefully (likely parse error)
        assert result.returncode in [0, 1, 2], "Should handle unclosed comments"


if __name__ == "__main__":
    test_hcl_single_line_comment_fuzzing()
    test_hcl_multiline_comment_fuzzing()
    test_hcl_nested_comment_fuzzing()
    test_hcl_comment_with_special_chars()
    test_hcl_comment_unicode_fuzzing()
    test_hcl_comment_length_fuzzing()
    test_hcl_unclosed_comment_fuzzing()
    print("All HCL comment fuzzing tests passed")
