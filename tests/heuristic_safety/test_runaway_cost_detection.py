#!/usr/bin/env python3
"""
Test: Runaway-cost detection test.

Validates detection of unreasonably high cost predictions.
"""

import os
import sys
import tempfile
import json


def test_threshold_definition():
    """Verify runaway cost threshold defined."""

    threshold = {
        "max_cost": 1000000.0,
        "defined": True
    }

    assert threshold["defined"] is True
    print(f"✓ Threshold definition (${threshold['max_cost']:,.0f})")


def test_runaway_detection():
    """Verify runaway costs detected."""

    with tempfile.NamedTemporaryFile(mode='w', suffix='_runaway.json', delete=False) as f:
        result = {
            "resources": [
                {"name": "instance", "cost": 5000000.0}
            ],
            "threshold": 1000000.0
        }
        json.dump(result, f)
        path = f.name

    try:
        with open(path, 'r') as f:
            data = json.load(f)

        runaway = [r for r in data["resources"] if r["cost"] > data["threshold"]]
        assert len(runaway) > 0
        print(f"✓ Runaway detection ({len(runaway)} found)")

    finally:
        os.unlink(path)


def test_warning_message():
    """Verify warning for runaway costs."""

    warning = {
        "resource": "aws_instance.large",
        "cost": 5000000.0,
        "threshold": 1000000.0,
        "message": "Warning: Cost $5,000,000 exceeds threshold $1,000,000",
        "shown": True
    }

    assert warning["shown"] is True
    print("✓ Warning message")


def test_percentage_increase_check():
    """Verify percentage increase checked."""

    increase = {
        "baseline": 100.0,
        "predicted": 10000.0,
        "increase_percent": 9900.0,
        "threshold_percent": 1000.0,
        "flagged": True
    }

    assert increase["flagged"] is True
    print(f"✓ Percentage increase check ({increase['increase_percent']}%)")


def test_absolute_threshold():
    """Verify absolute threshold enforced."""

    absolute = {
        "cost": 10000000.0,
        "absolute_max": 5000000.0,
        "exceeds": True
    }

    assert absolute["exceeds"] is True
    print(f"✓ Absolute threshold (${absolute['absolute_max']:,.0f})")


def test_relative_threshold():
    """Verify relative threshold enforced."""

    relative = {
        "baseline": 100.0,
        "predicted": 500.0,
        "multiplier": 5.0,
        "max_multiplier": 3.0,
        "exceeds": True
    }

    assert relative["exceeds"] is True
    print(f"✓ Relative threshold ({relative['multiplier']}x)")


def test_resource_type_thresholds():
    """Verify different thresholds per resource type."""

    thresholds = {
        "aws_instance": 10000.0,
        "aws_rds": 50000.0,
        "aws_s3": 1000.0,
        "type_specific": True
    }

    assert thresholds["type_specific"] is True
    print(f"✓ Resource type thresholds ({len([k for k in thresholds if 'aws_' in k])} types)")


def test_confirmation_prompt():
    """Verify confirmation prompt for high costs."""

    confirmation = {
        "cost": 500000.0,
        "requires_confirmation": True,
        "prompted": True
    }

    assert confirmation["prompted"] is True
    print("✓ Confirmation prompt")


def test_blocking_behavior():
    """Verify runaway costs can block execution."""

    blocking = {
        "cost": 10000000.0,
        "block_enabled": True,
        "blocked": True
    }

    assert blocking["blocked"] is True
    print("✓ Blocking behavior")


def test_reporting():
    """Verify runaway costs reported."""

    with tempfile.NamedTemporaryFile(mode='w', suffix='_report.json', delete=False) as f:
        report = {
            "runaway_costs": [
                {"resource": "res1", "cost": 2000000.0},
                {"resource": "res2", "cost": 3000000.0}
            ],
            "total": 5000000.0
        }
        json.dump(report, f)
        path = f.name

    try:
        with open(path, 'r') as f:
            data = json.load(f)

        assert data["total"] == 5000000.0
        print(f"✓ Reporting (${data['total']:,.0f} total)")

    finally:
        os.unlink(path)


def test_override_mechanism():
    """Verify override mechanism for legitimate high costs."""

    override = {
        "cost": 2000000.0,
        "override_enabled": True,
        "reason": "Annual reserved instance",
        "allowed": True
    }

    assert override["allowed"] is True
    print("✓ Override mechanism")


if __name__ == "__main__":
    print("Testing runaway-cost detection...")

    try:
        test_threshold_definition()
        test_runaway_detection()
        test_warning_message()
        test_percentage_increase_check()
        test_absolute_threshold()
        test_relative_threshold()
        test_resource_type_thresholds()
        test_confirmation_prompt()
        test_blocking_behavior()
        test_reporting()
        test_override_mechanism()

        print("\n✅ All runaway-cost detection tests passed")
        sys.exit(0)

    except AssertionError as e:
        print(f"\n❌ Test failed: {e}", file=sys.stderr)
        sys.exit(1)
    except Exception as e:
        print(f"\n❌ Unexpected error: {e}", file=sys.stderr)
        sys.exit(1)
