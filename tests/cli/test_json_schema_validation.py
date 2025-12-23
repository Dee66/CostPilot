#!/usr/bin/env python3
"""
Test: JSON output conforms to schema (strict validation).

Validates that JSON schemas are valid JSON Schema documents.
"""

import json
import sys
import os

# Add the project root to Python path for imports
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', '..'))

try:
    import jsonschema
except ImportError:
    print("❌ jsonschema not available, installing...")
    subprocess.run([sys.executable, "-m", "pip", "install", "jsonschema"], check=True)
    import jsonschema


def load_schema(schema_path):
    """Load JSON schema from file."""
    with open(schema_path, 'r') as f:
        return json.load(f)


def validate_schema_against_metaschema(schema):
    """Validate a schema against the JSON Schema meta-schema."""
    try:
        # Use Draft 7 meta-schema
        metaschema = jsonschema.Draft7Validator.META_SCHEMA
        jsonschema.validate(instance=schema, schema=metaschema)
        return True
    except jsonschema.ValidationError as e:
        print(f"❌ Schema validation against meta-schema failed: {e.message}")
        return False


def test_schema_validity():
    """Test that all JSON schemas are valid JSON Schema documents."""
    print("Testing JSON schema validity...")

    schema_files = [
        "tests/golden/schemas/detection_output.schema.json",
        "tests/golden/schemas/mapping_output.schema.json",
        "tests/golden/schemas/prediction_output.schema.json",
    ]

    all_valid = True
    for schema_file in schema_files:
        print(f"  Validating {schema_file}...")
        try:
            schema = load_schema(schema_file)
            if validate_schema_against_metaschema(schema):
                print(f"  ✅ {schema_file} is valid")
            else:
                print(f"  ❌ {schema_file} is invalid")
                all_valid = False
        except Exception as e:
            print(f"  ❌ Failed to load or validate {schema_file}: {e}")
            all_valid = False

    if all_valid:
        print("✅ All JSON schemas are valid")
    else:
        print("❌ Some JSON schemas are invalid")

    return all_valid


if __name__ == "__main__":
    if test_schema_validity():
        sys.exit(0)
    else:
        sys.exit(1)
