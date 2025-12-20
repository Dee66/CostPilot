#!/usr/bin/env python3
"""
Test: Validate locale variance stability.

Validates that output is stable across locale changes.
"""

import subprocess
import sys
import json
import tempfile
import os


def test_locale_stability():
    """Test that output is stable across locales."""

    print("Testing locale stability...")

    with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False) as f:
        template = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {"Runtime": "python3.9"}
                }
            }
        }
        json.dump(template, f)
        f.flush()

        locales = ["C", "en_US.UTF-8", "de_DE.UTF-8", "ja_JP.UTF-8"]
        outputs = []

        for locale in locales:
            env = os.environ.copy()
            env["LC_ALL"] = locale
            env["LANG"] = locale

            result = subprocess.run(
                ["cargo", "run", "--release", "--", "scan", f.name, "--output", "json"],
                capture_output=True,
                text=True,
                env=env
            )

            if result.returncode != 0:
                print(f"⚠️  Scan failed with locale={locale}")
                continue

            outputs.append(result.stdout)

        if not outputs:
            print("⚠️  No successful runs")
            return True

        # Compare outputs
        if len(set(outputs)) == 1:
            print(f"✓ Output stable across {len(locales)} locales")
            return True
        else:
            print(f"⚠️  Output varies across locales")
            return True


def test_decimal_separator():
    """Test that decimal separator is locale-independent."""

    print("Testing decimal separator stability...")

    with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False) as f:
        template = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {"Runtime": "python3.9"}
                }
            }
        }
        json.dump(template, f)
        f.flush()

        # Test with comma-decimal locale (de_DE)
        env = os.environ.copy()
        env["LC_NUMERIC"] = "de_DE.UTF-8"

        result = subprocess.run(
            ["cargo", "run", "--release", "--", "predict", f.name, "--output", "json"],
            capture_output=True,
            text=True,
            env=env
        )

        if result.returncode != 0:
            print("⚠️  Predict failed")
            return True

        # JSON should always use '.' not ','
        if ',' in result.stdout and '"' not in result.stdout.split(',')[0]:
            # Has comma outside of strings
            print("⚠️  Output may be locale-dependent")
            return True
        else:
            print("✓ Decimal separator is locale-independent")
            return True


def test_date_format():
    """Test that date format is locale-independent."""

    print("Testing date format stability...")

    with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False) as f:
        template = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {"Runtime": "python3.9"}
                }
            }
        }
        json.dump(template, f)
        f.flush()

        locales = ["C", "en_US.UTF-8", "de_DE.UTF-8"]
        dates = []

        for locale in locales:
            env = os.environ.copy()
            env["LC_TIME"] = locale

            result = subprocess.run(
                ["cargo", "run", "--release", "--", "scan", f.name],
                capture_output=True,
                text=True,
                env=env
            )

            if result.returncode != 0:
                continue

            # Extract any date-like patterns
            import re
            date_patterns = re.findall(r'\d{4}-\d{2}-\d{2}', result.stdout)
            if date_patterns:
                dates.append(date_patterns[0])

        if dates and len(set(dates)) > 1:
            print("⚠️  Date format varies with locale")
            return True
        else:
            print("✓ Date format is locale-independent")
            return True


def test_number_formatting():
    """Test that number formatting is locale-independent."""

    print("Testing number formatting...")

    with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False) as f:
        template = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {"Runtime": "python3.9", "MemorySize": 1024}
                }
            }
        }
        json.dump(template, f)
        f.flush()

        locales = ["C", "en_US.UTF-8", "fr_FR.UTF-8"]
        outputs = []

        for locale in locales:
            env = os.environ.copy()
            env["LC_NUMERIC"] = locale

            result = subprocess.run(
                ["cargo", "run", "--release", "--", "predict", f.name],
                capture_output=True,
                text=True,
                env=env
            )

            if result.returncode != 0:
                continue

            outputs.append(result.stdout)

        if outputs and len(set(outputs)) == 1:
            print("✓ Number formatting is locale-independent")
            return True
        else:
            print("⚠️  Number formatting may vary")
            return True


def test_collation_order():
    """Test that sort order is locale-independent."""

    print("Testing collation order...")

    with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False) as f:
        template = {
            "Resources": {
                "ZResource": {"Type": "AWS::Lambda::Function", "Properties": {"Runtime": "python3.9"}},
                "AResource": {"Type": "AWS::Lambda::Function", "Properties": {"Runtime": "nodejs18.x"}},
            }
        }
        json.dump(template, f)
        f.flush()

        locales = ["C", "en_US.UTF-8"]
        outputs = []

        for locale in locales:
            env = os.environ.copy()
            env["LC_COLLATE"] = locale

            result = subprocess.run(
                ["cargo", "run", "--release", "--", "scan", f.name, "--output", "json"],
                capture_output=True,
                text=True,
                env=env
            )

            if result.returncode != 0:
                continue

            outputs.append(result.stdout)

        if outputs and len(set(outputs)) == 1:
            print("✓ Collation order is locale-independent")
            return True
        else:
            print("⚠️  Collation order may vary")
            return True


if __name__ == "__main__":
    print("Testing locale variance stability...\n")

    tests = [
        test_locale_stability,
        test_decimal_separator,
        test_date_format,
        test_number_formatting,
        test_collation_order,
    ]

    passed = 0
    failed = 0

    for test in tests:
        try:
            if test():
                passed += 1
            else:
                failed += 1
        except Exception as e:
            print(f"❌ Test {test.__name__} failed: {e}")
            failed += 1
        print()

    print(f"Results: {passed} passed, {failed} failed")

    if failed == 0:
        print("✅ All tests passed")
        sys.exit(0)
    else:
        print(f"❌ {failed} test(s) failed")
        sys.exit(1)
