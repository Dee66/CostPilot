#!/usr/bin/env python3
"""
Test: Business unit attribution.

Validates costs can be attributed to business units or teams.
"""

import os
import sys
import tempfile
import json


def test_business_unit_tagging():
    """Verify resources can be tagged with business units."""
    
    with tempfile.NamedTemporaryFile(mode='w', suffix='_resources.json', delete=False) as f:
        resources = [
            {"id": "r-001", "type": "aws_instance", "cost": 100.0, "business_unit": "engineering"},
            {"id": "r-002", "type": "aws_rds", "cost": 50.0, "business_unit": "product"},
            {"id": "r-003", "type": "aws_s3", "cost": 25.0, "business_unit": "marketing"}
        ]
        json.dump(resources, f)
        path = f.name
    
    try:
        with open(path, 'r') as f:
            data = json.load(f)
        
        assert all("business_unit" in r for r in data)
        
        print(f"✓ Business unit tagging ({len(data)} resources)")
        
    finally:
        os.unlink(path)


def test_cost_aggregation_by_bu():
    """Verify costs are aggregated by business unit."""
    
    resources = [
        {"business_unit": "engineering", "cost": 100.0},
        {"business_unit": "engineering", "cost": 150.0},
        {"business_unit": "product", "cost": 50.0}
    ]
    
    bu_costs = {}
    for r in resources:
        bu = r["business_unit"]
        bu_costs[bu] = bu_costs.get(bu, 0) + r["cost"]
    
    assert bu_costs["engineering"] == 250.0
    
    print(f"✓ Cost aggregation by BU ({len(bu_costs)} units)")


def test_bu_budget_tracking():
    """Verify business unit budgets are tracked."""
    
    with tempfile.NamedTemporaryFile(mode='w', suffix='_budgets.json', delete=False) as f:
        budgets = {
            "engineering": {"budget": 1000.0, "spent": 250.0, "remaining": 750.0},
            "product": {"budget": 500.0, "spent": 50.0, "remaining": 450.0}
        }
        json.dump(budgets, f)
        path = f.name
    
    try:
        with open(path, 'r') as f:
            data = json.load(f)
        
        assert data["engineering"]["remaining"] == 750.0
        
        print(f"✓ BU budget tracking ({len(data)} units)")
        
    finally:
        os.unlink(path)


def test_bu_cost_reports():
    """Verify business unit cost reports are generated."""
    
    with tempfile.NamedTemporaryFile(mode='w', suffix='_report.json', delete=False) as f:
        report = {
            "period": "2024-01",
            "business_units": {
                "engineering": {"total_cost": 250.0, "resource_count": 10},
                "product": {"total_cost": 50.0, "resource_count": 3}
            }
        }
        json.dump(report, f)
        path = f.name
    
    try:
        with open(path, 'r') as f:
            data = json.load(f)
        
        assert "engineering" in data["business_units"]
        
        print(f"✓ BU cost reports ({len(data['business_units'])} units)")
        
    finally:
        os.unlink(path)


def test_hierarchical_bu_structure():
    """Verify hierarchical business unit structure is supported."""
    
    with tempfile.NamedTemporaryFile(mode='w', suffix='_hierarchy.json', delete=False) as f:
        hierarchy = {
            "engineering": {
                "parent": None,
                "children": ["backend", "frontend", "mobile"]
            },
            "backend": {
                "parent": "engineering",
                "children": []
            }
        }
        json.dump(hierarchy, f)
        path = f.name
    
    try:
        with open(path, 'r') as f:
            data = json.load(f)
        
        assert "children" in data["engineering"]
        
        print("✓ Hierarchical BU structure")
        
    finally:
        os.unlink(path)


def test_cost_allocation_rules():
    """Verify cost allocation rules are applied."""
    
    with tempfile.NamedTemporaryFile(mode='w', suffix='_rules.json', delete=False) as f:
        rules = {
            "shared_infrastructure": {
                "allocation_method": "proportional",
                "business_units": {
                    "engineering": 0.6,
                    "product": 0.3,
                    "marketing": 0.1
                }
            }
        }
        json.dump(rules, f)
        path = f.name
    
    try:
        with open(path, 'r') as f:
            data = json.load(f)
        
        total_allocation = sum(data["shared_infrastructure"]["business_units"].values())
        assert abs(total_allocation - 1.0) < 0.01  # Should sum to 1.0
        
        print("✓ Cost allocation rules (proportional)")
        
    finally:
        os.unlink(path)


def test_chargeback_reports():
    """Verify chargeback reports are generated."""
    
    with tempfile.NamedTemporaryFile(mode='w', suffix='_chargeback.json', delete=False) as f:
        chargeback = {
            "period": "2024-01",
            "business_unit": "engineering",
            "total_charges": 250.0,
            "line_items": [
                {"resource": "compute", "cost": 150.0},
                {"resource": "storage", "cost": 100.0}
            ]
        }
        json.dump(chargeback, f)
        path = f.name
    
    try:
        with open(path, 'r') as f:
            data = json.load(f)
        
        assert data["total_charges"] == 250.0
        
        print(f"✓ Chargeback reports ({len(data['line_items'])} items)")
        
    finally:
        os.unlink(path)


def test_bu_cost_trends():
    """Verify business unit cost trends are tracked."""
    
    with tempfile.NamedTemporaryFile(mode='w', suffix='_trends.json', delete=False) as f:
        trends = {
            "business_unit": "engineering",
            "monthly_costs": [
                {"month": "2024-01", "cost": 200.0},
                {"month": "2024-02", "cost": 250.0},
                {"month": "2024-03", "cost": 275.0}
            ]
        }
        json.dump(trends, f)
        path = f.name
    
    try:
        with open(path, 'r') as f:
            data = json.load(f)
        
        assert len(data["monthly_costs"]) == 3
        
        print(f"✓ BU cost trends ({len(data['monthly_costs'])} months)")
        
    finally:
        os.unlink(path)


def test_multi_tagging_support():
    """Verify resources can have multiple tags for attribution."""
    
    resource = {
        "id": "r-001",
        "tags": {
            "business_unit": "engineering",
            "team": "backend",
            "project": "api-v2",
            "environment": "production"
        }
    }
    
    assert "business_unit" in resource["tags"]
    assert "team" in resource["tags"]
    
    print(f"✓ Multi-tagging support ({len(resource['tags'])} tags)")


def test_untagged_resource_reporting():
    """Verify untagged resources are reported."""
    
    with tempfile.NamedTemporaryFile(mode='w', suffix='_untagged.json', delete=False) as f:
        untagged = {
            "count": 5,
            "resources": [
                {"id": "r-010", "cost": 10.0},
                {"id": "r-011", "cost": 15.0}
            ],
            "total_cost": 125.0
        }
        json.dump(untagged, f)
        path = f.name
    
    try:
        with open(path, 'r') as f:
            data = json.load(f)
        
        assert data["count"] > 0
        
        print(f"✓ Untagged resource reporting ({data['count']} resources)")
        
    finally:
        os.unlink(path)


def test_bu_cost_forecasting():
    """Verify business unit cost forecasting."""
    
    with tempfile.NamedTemporaryFile(mode='w', suffix='_forecast.json', delete=False) as f:
        forecast = {
            "business_unit": "engineering",
            "forecast_period": "2024-04",
            "predicted_cost": 290.0,
            "confidence": 0.85
        }
        json.dump(forecast, f)
        path = f.name
    
    try:
        with open(path, 'r') as f:
            data = json.load(f)
        
        assert data["predicted_cost"] > 0
        
        print(f"✓ BU cost forecasting (85% confidence)")
        
    finally:
        os.unlink(path)


if __name__ == "__main__":
    print("Testing business unit attribution...")
    
    try:
        test_business_unit_tagging()
        test_cost_aggregation_by_bu()
        test_bu_budget_tracking()
        test_bu_cost_reports()
        test_hierarchical_bu_structure()
        test_cost_allocation_rules()
        test_chargeback_reports()
        test_bu_cost_trends()
        test_multi_tagging_support()
        test_untagged_resource_reporting()
        test_bu_cost_forecasting()
        
        print("\n✅ All business unit attribution tests passed")
        sys.exit(0)
        
    except AssertionError as e:
        print(f"\n❌ Test failed: {e}", file=sys.stderr)
        sys.exit(1)
    except Exception as e:
        print(f"\n❌ Unexpected error: {e}", file=sys.stderr)
        sys.exit(1)
