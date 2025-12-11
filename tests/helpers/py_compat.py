"""Python test compatibility helpers.

Provides adapter functions to convert legacy test fixtures into current runtime shapes.
"""

from typing import Optional, Dict, Any


def make_cost_estimate(
    monthly: float,
    lower: Optional[float] = None,
    upper: Optional[float] = None,
    confidence: Optional[float] = None,
    resource_id: Optional[str] = None
) -> Dict[str, Any]:
    """Create a cost estimate dict matching production JSON shape.
    
    Maps legacy field names to canonical names:
    - monthly -> monthly_cost
    - lower -> prediction_interval_low
    - upper -> prediction_interval_high
    - confidence -> confidence_score
    """
    return {
        "resource_id": resource_id or "test://resource",
        "monthly_cost": monthly,
        "prediction_interval_low": lower if lower is not None else monthly * 0.9,
        "prediction_interval_high": upper if upper is not None else monthly * 1.1,
        "confidence_score": confidence if confidence is not None else 0.75,
        "heuristic_reference": "test-compat",
        "cold_start": False
    }


def make_resource_change(
    before: Optional[Dict[str, Any]],
    after: Optional[Dict[str, Any]],
    action: Optional[str] = None,
    monthly: Optional[float] = None
) -> Dict[str, Any]:
    """Create a resource change dict matching production JSON shape.
    
    Normalizes action strings to canonical form:
    - create/created -> Create
    - update/modify -> Update
    - delete/removed -> Delete
    - replace -> Replace
    - else -> NoOp
    """
    action_map = {
        "create": "Create",
        "created": "Create",
        "update": "Update",
        "modify": "Update",
        "delete": "Delete",
        "removed": "Delete",
        "replace": "Replace"
    }
    
    canonical_action = action_map.get(action.lower() if action else "", "NoOp")
    
    result = {
        "resource_id": "test_resource",
        "action": canonical_action,
        "old_config": before,
        "new_config": after
    }
    
    if monthly is not None:
        result["monthly_cost"] = monthly
    
    return result


def make_detection(
    rule_id: str,
    msg: Optional[str] = None,
    severity: Optional[float] = None,
    estimated: Optional[float] = None
) -> Dict[str, Any]:
    """Create a detection dict matching production JSON shape.
    
    Maps numeric severity to string enum:
    - 0-3 -> Low
    - 3-6 -> Medium
    - >6 -> High
    """
    severity_val = severity if severity is not None else 5.0
    
    if severity_val <= 3.0:
        severity_str = "Low"
    elif severity_val <= 6.0:
        severity_str = "Medium"
    else:
        severity_str = "High"
    
    return {
        "rule_id": rule_id,
        "message": msg or "",
        "severity": severity_str,
        "estimated_cost": estimated or 0.0
    }


def edition_context_free() -> Dict[str, Any]:
    """Return a minimal Free edition context fixture for tests."""
    return {
        "edition": "free",
        "licensed": False
    }
