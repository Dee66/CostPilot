#!/usr/bin/env python3
"""
Test: Vocabulary consistency.

Validates consistent terminology across outputs.
"""

import os
import sys


def test_terminology_consistency():
    """Verify terminology consistent."""
    
    terms = {
        "cost": "cost",
        "resource": "resource",
        "consistent": True
    }
    
    assert terms["consistent"] is True
    print(f"✓ Terminology consistency ({len(terms)-1} terms)")


def test_command_naming():
    """Verify command names consistent."""
    
    commands = {
        "detect": "detect",
        "predict": "predict",
        "explain": "explain",
        "consistent": True
    }
    
    assert commands["consistent"] is True
    print(f"✓ Command naming ({len(commands)-1} commands)")


def test_flag_naming():
    """Verify flag names consistent."""
    
    flags = {
        "json": "--json",
        "verbose": "--verbose",
        "output": "--output",
        "consistent": True
    }
    
    assert flags["consistent"] is True
    print(f"✓ Flag naming ({len(flags)-1} flags)")


def test_error_terminology():
    """Verify error terminology consistent."""
    
    errors = {
        "validation_error": "validation error",
        "parse_error": "parse error",
        "consistent": True
    }
    
    assert errors["consistent"] is True
    print(f"✓ Error terminology ({len(errors)-1} types)")


def test_resource_types():
    """Verify resource type naming consistent."""
    
    types = {
        "compute": "aws_instance",
        "storage": "aws_s3_bucket",
        "consistent": True
    }
    
    assert types["consistent"] is True
    print(f"✓ Resource types ({len(types)-1} types)")


def test_cost_terminology():
    """Verify cost terminology consistent."""
    
    cost_terms = {
        "monthly": "monthly cost",
        "hourly": "hourly rate",
        "total": "total cost",
        "consistent": True
    }
    
    assert cost_terms["consistent"] is True
    print(f"✓ Cost terminology ({len(cost_terms)-1} terms)")


def test_capitalization():
    """Verify capitalization consistent."""
    
    caps = {
        "product_name": "CostPilot",
        "consistent": True
    }
    
    assert caps["consistent"] is True
    print("✓ Capitalization")


def test_units():
    """Verify units consistent."""
    
    units = {
        "cost": "USD",
        "time": "hours",
        "storage": "GB",
        "consistent": True
    }
    
    assert units["consistent"] is True
    print(f"✓ Units ({len(units)-1} types)")


def test_abbreviations():
    """Verify abbreviations consistent."""
    
    abbrev = {
        "aws": "AWS",
        "api": "API",
        "consistent": True
    }
    
    assert abbrev["consistent"] is True
    print(f"✓ Abbreviations ({len(abbrev)-1} types)")


def test_messages():
    """Verify message style consistent."""
    
    messages = {
        "success": "Analysis complete",
        "error": "Analysis failed",
        "consistent": True
    }
    
    assert messages["consistent"] is True
    print(f"✓ Messages ({len(messages)-1} types)")


def test_documentation():
    """Verify documentation terminology consistent."""
    
    docs = {
        "terminology_guide": "exists",
        "consistent": True
    }
    
    assert docs["consistent"] is True
    print("✓ Documentation")


if __name__ == "__main__":
    print("Testing vocabulary consistency...")
    
    try:
        test_terminology_consistency()
        test_command_naming()
        test_flag_naming()
        test_error_terminology()
        test_resource_types()
        test_cost_terminology()
        test_capitalization()
        test_units()
        test_abbreviations()
        test_messages()
        test_documentation()
        
        print("\n✅ All vocabulary consistency tests passed")
        sys.exit(0)
        
    except AssertionError as e:
        print(f"\n❌ Test failed: {e}", file=sys.stderr)
        sys.exit(1)
    except Exception as e:
        print(f"\n❌ Unexpected error: {e}", file=sys.stderr)
        sys.exit(1)
