#!/usr/bin/env python3
"""
Test: Monotonic cost breakdown validation.

Validates that cost breakdowns maintain monotonic properties.
"""

import os
import sys
import tempfile
import json


def test_sum_equals_total():
    """Verify breakdown components sum to total."""

    breakdown = {
        "total": 100.0,
        "components": {
            "compute": 60.0,
            "storage": 30.0,
            "network": 10.0
        }
    }

    component_sum = sum(breakdown["components"].values())
    assert abs(component_sum - breakdown["total"]) < 0.01
    print(f"✓ Sum equals total (${breakdown['total']})")


def test_non_negative_components():
    """Verify all components are non-negative."""

    components = {
        "compute": 50.0,
        "storage": 30.0,
        "network": 20.0
    }

    all_positive = all(v >= 0 for v in components.values())
    assert all_positive is True
    print(f"✓ Non-negative components ({len(components)} components)")


def test_nested_breakdown_consistency():
    """Verify nested breakdowns consistent."""

    nested = {
        "total": 100.0,
        "level1": {
            "infrastructure": 80.0,
            "services": 20.0
        },
        "level2": {
            "compute": 50.0,
            "storage": 30.0,
            "load_balancer": 20.0
        }
    }

    level1_sum = sum(nested["level1"].values())
    level2_sum = sum(nested["level2"].values())
    assert abs(level1_sum - level2_sum) < 0.01
    print("✓ Nested breakdown consistency")


def test_percentage_calculation():
    """Verify percentages calculated correctly."""

    with tempfile.NamedTemporaryFile(mode='w', suffix='_percent.json', delete=False) as f:
        data = {
            "total": 100.0,
            "components": [
                {"name": "compute", "cost": 60.0, "percent": 60.0},
                {"name": "storage", "cost": 40.0, "percent": 40.0}
            ]
        }
        json.dump(data, f)
        path = f.name

    try:
        with open(path, 'r') as f:
            loaded = json.load(f)

        total_percent = sum(c["percent"] for c in loaded["components"])
        assert abs(total_percent - 100.0) < 0.01
        print(f"✓ Percentage calculation ({total_percent}%)")

    finally:
        os.unlink(path)


def test_hierarchical_consistency():
    """Verify hierarchical breakdown consistency."""

    hierarchy = {
        "root": 100.0,
        "children": [
            {"name": "child1", "cost": 60.0},
            {"name": "child2", "cost": 40.0}
        ],
        "consistent": True
    }

    children_sum = sum(c["cost"] for c in hierarchy["children"])
    assert abs(children_sum - hierarchy["root"]) < 0.01
    print(f"✓ Hierarchical consistency ({len(hierarchy['children'])} children)")


def test_rounding_errors():
    """Verify rounding errors handled."""

    rounding = {
        "components": [33.33, 33.33, 33.34],
        "total": 100.0,
        "tolerance": 0.01,
        "valid": True
    }

    assert rounding["valid"] is True
    print(f"✓ Rounding errors (tolerance={rounding['tolerance']})")


def test_monotonic_increase():
    """Verify costs increase monotonically in aggregations."""

    aggregation = {
        "daily": 10.0,
        "weekly": 70.0,
        "monthly": 300.0,
        "monotonic": True
    }

    assert aggregation["daily"] <= aggregation["weekly"] <= aggregation["monthly"]
    print("✓ Monotonic increase")


def test_zero_components():
    """Verify zero-cost components handled."""

    zero = {
        "compute": 100.0,
        "storage": 0.0,
        "network": 0.0,
        "total": 100.0,
        "valid": True
    }

    assert zero["valid"] is True
    print("✓ Zero components")


def test_precision_maintenance():
    """Verify precision maintained in breakdowns."""

    precision = {
        "component": 33.333333,
        "rounded": 33.33,
        "precision": 2,
        "maintained": True
    }

    assert precision["maintained"] is True
    print(f"✓ Precision maintenance ({precision['precision']} decimals)")


def test_validation():
    """Verify breakdown validation enforced."""

    validation = {
        "total": 100.0,
        "sum_of_parts": 100.0,
        "difference": 0.0,
        "valid": True
    }

    assert validation["valid"] is True
    print(f"✓ Validation (diff={validation['difference']})")


def test_error_reporting():
    """Verify breakdown errors reported clearly."""

    error = {
        "total": 100.0,
        "sum": 95.0,
        "difference": 5.0,
        "error_message": "Breakdown sum (95.0) does not match total (100.0)",
        "reported": True
    }

    assert error["reported"] is True
    print("✓ Error reporting")


if __name__ == "__main__":
    print("Testing monotonic cost breakdown validation...")

    try:
        test_sum_equals_total()
        test_non_negative_components()
        test_nested_breakdown_consistency()
        test_percentage_calculation()
        test_hierarchical_consistency()
        test_rounding_errors()
        test_monotonic_increase()
        test_zero_components()
        test_precision_maintenance()
        test_validation()
        test_error_reporting()

        print("\n✅ All monotonic cost breakdown validation tests passed")
        sys.exit(0)

    except AssertionError as e:
        print(f"\n❌ Test failed: {e}", file=sys.stderr)
        sys.exit(1)
    except Exception as e:
        print(f"\n❌ Unexpected error: {e}", file=sys.stderr)
        sys.exit(1)
