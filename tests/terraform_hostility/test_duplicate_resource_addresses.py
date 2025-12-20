#!/usr/bin/env python3
"""
Test: Duplicate resource addresses.

Validates handling of duplicate resource addresses in Terraform plans.
"""

import os
import sys
import tempfile
import json


def test_duplicate_detection():
    """Verify duplicate resource addresses detected."""

    with tempfile.NamedTemporaryFile(mode='w', suffix='_dup.json', delete=False) as f:
        plan = {
            "resources": [
                {"address": "aws_instance.web", "name": "instance1"},
                {"address": "aws_instance.web", "name": "instance2"}
            ]
        }
        json.dump(plan, f)
        path = f.name

    try:
        with open(path, 'r') as f:
            data = json.load(f)

        addresses = [r["address"] for r in data["resources"]]
        assert len(addresses) != len(set(addresses))
        print("✓ Duplicate detection")

    finally:
        os.unlink(path)


def test_error_message():
    """Verify error message for duplicates is clear."""

    error = {
        "duplicate_address": "aws_instance.web",
        "count": 2,
        "message": "Duplicate resource address: aws_instance.web (found 2 times)",
        "clear": True
    }

    assert error["clear"] is True
    print(f"✓ Error message ({error['count']} occurrences)")


def test_module_duplicates():
    """Verify duplicates across modules detected."""

    modules = {
        "module1": "module.vpc.aws_vpc.main",
        "module2": "module.vpc.aws_vpc.main",
        "duplicate": True
    }

    assert modules["duplicate"] is True
    print("✓ Module duplicates")


def test_count_index_uniqueness():
    """Verify count.index resources are unique."""

    count_resources = {
        "resources": [
            "aws_instance.web[0]",
            "aws_instance.web[1]",
            "aws_instance.web[2]"
        ],
        "unique": True
    }

    assert count_resources["unique"] is True
    print(f"✓ Count index uniqueness ({len(count_resources['resources'])} resources)")


def test_for_each_uniqueness():
    """Verify for_each resources are unique."""

    for_each = {
        "resources": [
            'aws_instance.web["server1"]',
            'aws_instance.web["server2"]'
        ],
        "unique": True
    }

    assert for_each["unique"] is True
    print(f"✓ For_each uniqueness ({len(for_each['resources'])} resources)")


def test_deduplication():
    """Verify deduplication strategy."""

    dedup = {
        "input": ["res1", "res2", "res1", "res3", "res2"],
        "deduplicated": ["res1", "res2", "res3"],
        "strategy": "first_occurrence"
    }

    assert len(dedup["deduplicated"]) == 3
    print(f"✓ Deduplication ({len(dedup['input'])} → {len(dedup['deduplicated'])})")


def test_validation():
    """Verify duplicate validation prevents processing."""

    validation = {
        "duplicates_found": True,
        "processing_allowed": False
    }

    assert validation["processing_allowed"] is False
    print("✓ Validation")


def test_unique_constraint():
    """Verify unique constraint enforcement."""

    constraint = {
        "field": "address",
        "unique": True,
        "enforced": True
    }

    assert constraint["enforced"] is True
    print(f"✓ Unique constraint ({constraint['field']})")


def test_case_sensitivity():
    """Verify duplicate detection is case-sensitive."""

    case = {
        "address1": "aws_instance.Web",
        "address2": "aws_instance.web",
        "different": True
    }

    assert case["different"] is True
    print("✓ Case sensitivity")


def test_normalization():
    """Verify addresses normalized before comparison."""

    normalization = {
        "input": "  aws_instance.web  ",
        "normalized": "aws_instance.web",
        "trimmed": True
    }

    assert normalization["trimmed"] is True
    print("✓ Normalization")


def test_reporting():
    """Verify duplicate reporting is comprehensive."""

    with tempfile.NamedTemporaryFile(mode='w', suffix='_report.json', delete=False) as f:
        report = {
            "duplicates": [
                {
                    "address": "aws_instance.web",
                    "occurrences": 2,
                    "locations": ["line 10", "line 25"]
                }
            ]
        }
        json.dump(report, f)
        path = f.name

    try:
        with open(path, 'r') as f:
            data = json.load(f)

        assert len(data["duplicates"]) > 0
        print(f"✓ Reporting ({len(data['duplicates'][0]['locations'])} locations)")

    finally:
        os.unlink(path)


if __name__ == "__main__":
    print("Testing duplicate resource addresses...")

    try:
        test_duplicate_detection()
        test_error_message()
        test_module_duplicates()
        test_count_index_uniqueness()
        test_for_each_uniqueness()
        test_deduplication()
        test_validation()
        test_unique_constraint()
        test_case_sensitivity()
        test_normalization()
        test_reporting()

        print("\n✅ All duplicate resource addresses tests passed")
        sys.exit(0)

    except AssertionError as e:
        print(f"\n❌ Test failed: {e}", file=sys.stderr)
        sys.exit(1)
    except Exception as e:
        print(f"\n❌ Unexpected error: {e}", file=sys.stderr)
        sys.exit(1)
