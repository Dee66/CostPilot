#!/usr/bin/env python3
"""Test large random-object fuzzing."""

import json
import random
import string
import subprocess
import tempfile
from pathlib import Path


def generate_random_object(depth=0, max_depth=5):
    """Generate random JSON object."""
    if depth >= max_depth:
        return random.choice([
            random.randint(0, 1000000),
            random.random(),
            ''.join(random.choices(string.ascii_letters, k=random.randint(1, 50))),
            random.choice([True, False]),
            None
        ])

    obj_type = random.choice(['dict', 'list', 'value'])

    if obj_type == 'dict':
        size = random.randint(1, 10)
        return {
            f"key_{i}": generate_random_object(depth + 1, max_depth)
            for i in range(size)
        }
    elif obj_type == 'list':
        size = random.randint(1, 10)
        return [generate_random_object(depth + 1, max_depth) for _ in range(size)]
    else:
        return generate_random_object(depth=max_depth)


def test_large_random_object_fuzzing():
    """Fuzz with large random objects."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"

        for _ in range(5):
            # Generate large random template
            random_resources = {}
            for i in range(random.randint(10, 50)):
                random_resources[f"Resource{i}"] = generate_random_object(max_depth=3)

            template_content = {
                "Resources": random_resources
            }

            with open(template_path, 'w') as f:
                json.dump(template_content, f)

            result = subprocess.run(
                ["costpilot", "analyze", "--template", str(template_path)],
                capture_output=True,
                text=True,
                timeout=10
            )

            # Should not crash
            assert result.returncode in [0, 1, 2], "Should handle large random objects"


def test_random_array_fuzzing():
    """Fuzz with random arrays."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"

        for _ in range(5):
            array_size = random.randint(100, 1000)
            random_array = [
                {
                    "id": i,
                    "value": random.random(),
                    "name": ''.join(random.choices(string.ascii_letters, k=10))
                }
                for i in range(array_size)
            ]

            template_content = {
                "Resources": {
                    "LargeArray": {
                        "Type": "Custom",
                        "Items": random_array
                    }
                }
            }

            with open(template_path, 'w') as f:
                json.dump(template_content, f)

            result = subprocess.run(
                ["costpilot", "analyze", "--template", str(template_path)],
                capture_output=True,
                text=True,
                timeout=10
            )

            assert result.returncode in [0, 1, 2], "Should handle large arrays"


def test_random_string_fuzzing():
    """Fuzz with random strings."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"

        for _ in range(10):
            string_length = random.randint(1000, 10000)
            random_string = ''.join(random.choices(
                string.ascii_letters + string.digits + string.punctuation,
                k=string_length
            ))

            template_content = {
                "Resources": {
                    "Lambda": {
                        "Type": "AWS::Lambda::Function",
                        "Properties": {
                            "Code": random_string
                        }
                    }
                }
            }

            with open(template_path, 'w') as f:
                json.dump(template_content, f)

            result = subprocess.run(
                ["costpilot", "analyze", "--template", str(template_path)],
                capture_output=True,
                text=True,
                timeout=5
            )

            assert result.returncode in [0, 1, 2], "Should handle large random strings"


def test_random_number_fuzzing():
    """Fuzz with random numbers."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"

        for _ in range(10):
            random_numbers = {
                "int": random.randint(-1000000, 1000000),
                "float": random.random() * 1000000,
                "negative": -random.randint(1, 1000000),
                "zero": 0,
                "large": random.randint(10**10, 10**15)
            }

            template_content = {
                "Resources": {
                    "Lambda": {
                        "Type": "AWS::Lambda::Function",
                        "Properties": random_numbers
                    }
                }
            }

            with open(template_path, 'w') as f:
                json.dump(template_content, f)

            result = subprocess.run(
                ["costpilot", "analyze", "--template", str(template_path)],
                capture_output=True,
                text=True,
                timeout=5
            )

            assert result.returncode in [0, 1, 2], "Should handle random numbers"


def test_mixed_type_fuzzing():
    """Fuzz with mixed types."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"

        for _ in range(5):
            mixed_object = {
                f"field_{i}": random.choice([
                    random.randint(0, 1000),
                    random.random(),
                    ''.join(random.choices(string.ascii_letters, k=20)),
                    random.choice([True, False]),
                    None,
                    [random.randint(0, 100) for _ in range(10)],
                    {"nested": random.random()}
                ])
                for i in range(50)
            }

            template_content = {
                "Resources": {
                    "MixedType": mixed_object
                }
            }

            with open(template_path, 'w') as f:
                json.dump(template_content, f)

            result = subprocess.run(
                ["costpilot", "analyze", "--template", str(template_path)],
                capture_output=True,
                text=True,
                timeout=5
            )

            assert result.returncode in [0, 1, 2], "Should handle mixed types"


def test_sparse_object_fuzzing():
    """Fuzz with sparse objects (many null values)."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"

        sparse_object = {
            f"field_{i}": None if random.random() > 0.3 else random.randint(0, 100)
            for i in range(100)
        }

        template_content = {
            "Resources": {
                "Sparse": sparse_object
            }
        }

        with open(template_path, 'w') as f:
            json.dump(template_content, f)

        result = subprocess.run(
            ["costpilot", "analyze", "--template", str(template_path)],
            capture_output=True,
            text=True,
            timeout=5
        )

        assert result.returncode in [0, 1, 2], "Should handle sparse objects"


def test_duplicate_key_fuzzing():
    """Fuzz with potential duplicate keys."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"

        # JSON will override duplicates, but test handling
        resources = {}
        for i in range(10):
            key = f"Lambda{random.randint(0, 5)}"  # Potential duplicates
            resources[key] = {
                "Type": "AWS::Lambda::Function",
                "Properties": {"MemorySize": random.randint(128, 3008)}
            }

        template_content = {"Resources": resources}

        with open(template_path, 'w') as f:
            json.dump(template_content, f)

        result = subprocess.run(
            ["costpilot", "analyze", "--template", str(template_path)],
            capture_output=True,
            text=True,
            timeout=5
        )

        assert result.returncode in [0, 1, 2], "Should handle potential duplicates"


if __name__ == "__main__":
    test_large_random_object_fuzzing()
    test_random_array_fuzzing()
    test_random_string_fuzzing()
    test_random_number_fuzzing()
    test_mixed_type_fuzzing()
    test_sparse_object_fuzzing()
    test_duplicate_key_fuzzing()
    print("All large random-object fuzzing tests passed")
