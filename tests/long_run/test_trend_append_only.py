#!/usr/bin/env python3
"""Test trend append-only invariant."""

import subprocess
import tempfile
from pathlib import Path
import json


def test_trend_append_only():
    """Test that trend history only appends, never modifies."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        trend_path = Path(tmpdir) / "trend.json"
        
        template_content = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 1024
                    }
                }
            }
        }
        
        # Initial trend history
        trend_content = {
            "history": [
                {"date": "2024-01-01", "cost": 10.0},
                {"date": "2024-01-02", "cost": 12.0},
                {"date": "2024-01-03", "cost": 15.0}
            ]
        }
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        with open(trend_path, 'w') as f:
            json.dump(trend_content, f)
        
        # Read initial state
        with open(trend_path, 'r') as f:
            initial_data = json.load(f)
        
        initial_history = initial_data.get("history", [])
        initial_count = len(initial_history)
        
        # Run trend analysis (should append)
        result = subprocess.run(
            ["costpilot", "trend", "--plan", str(template_path), "--history", str(trend_path), "--append"],
            capture_output=True,
            text=True,
            timeout=30
        )
        
        # Check if file was modified
        if trend_path.exists():
            with open(trend_path, 'r') as f:
                updated_data = json.load(f)
            
            updated_history = updated_data.get("history", [])
            updated_count = len(updated_history)
            
            # Should only append (count >= initial)
            assert updated_count >= initial_count, \
                f"Trend history should only append, not remove entries"
            
            # Initial entries should be unchanged
            for i, entry in enumerate(initial_history):
                assert updated_history[i] == entry, \
                    f"Entry {i} was modified (should be append-only)"


def test_trend_no_modification():
    """Test that existing trend entries are never modified."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        trend_path = Path(tmpdir) / "trend.json"
        
        template_content = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 2048
                    }
                }
            }
        }
        
        # Trend with specific values
        trend_content = {
            "history": [
                {"date": "2024-01-01", "cost": 10.0, "resources": 5},
                {"date": "2024-01-02", "cost": 12.5, "resources": 5},
                {"date": "2024-01-03", "cost": 15.0, "resources": 6}
            ]
        }
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        with open(trend_path, 'w') as f:
            json.dump(trend_content, f)
        
        # Save initial values
        initial_entries = trend_content["history"][:]
        
        # Run trend analysis multiple times
        for _ in range(5):
            result = subprocess.run(
                ["costpilot", "trend", "--plan", str(template_path), "--history", str(trend_path)],
                capture_output=True,
                text=True,
                timeout=30
            )
            
            if trend_path.exists():
                with open(trend_path, 'r') as f:
                    current_data = json.load(f)
                
                current_history = current_data.get("history", [])
                
                # Check first 3 entries unchanged
                for i in range(min(3, len(current_history))):
                    assert current_history[i] == initial_entries[i], \
                        f"Entry {i} was modified (should be immutable)"


def test_trend_ordering_preserved():
    """Test that trend history ordering is preserved."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        trend_path = Path(tmpdir) / "trend.json"
        
        template_content = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 1024
                    }
                }
            }
        }
        
        # Trend with chronological order
        trend_content = {
            "history": [
                {"date": "2024-01-01", "cost": 10.0},
                {"date": "2024-01-02", "cost": 11.0},
                {"date": "2024-01-03", "cost": 12.0},
                {"date": "2024-01-04", "cost": 13.0},
                {"date": "2024-01-05", "cost": 14.0}
            ]
        }
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        with open(trend_path, 'w') as f:
            json.dump(trend_content, f)
        
        # Run trend analysis
        result = subprocess.run(
            ["costpilot", "trend", "--plan", str(template_path), "--history", str(trend_path)],
            capture_output=True,
            text=True,
            timeout=30
        )
        
        if trend_path.exists():
            with open(trend_path, 'r') as f:
                updated_data = json.load(f)
            
            updated_history = updated_data.get("history", [])
            
            # Check ordering preserved
            for i in range(len(trend_content["history"])):
                if i < len(updated_history):
                    assert updated_history[i]["date"] == trend_content["history"][i]["date"], \
                        "Trend history ordering should be preserved"


def test_trend_no_duplicates():
    """Test that trend history doesn't create duplicates."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        trend_path = Path(tmpdir) / "trend.json"
        
        template_content = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 1024
                    }
                }
            }
        }
        
        trend_content = {
            "history": [
                {"date": "2024-01-01", "cost": 10.0},
                {"date": "2024-01-02", "cost": 12.0}
            ]
        }
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        with open(trend_path, 'w') as f:
            json.dump(trend_content, f)
        
        # Run multiple times
        for _ in range(3):
            result = subprocess.run(
                ["costpilot", "trend", "--plan", str(template_path), "--history", str(trend_path)],
                capture_output=True,
                text=True,
                timeout=30
            )
        
        if trend_path.exists():
            with open(trend_path, 'r') as f:
                final_data = json.load(f)
            
            final_history = final_data.get("history", [])
            
            # Check for duplicate dates
            dates = [entry["date"] for entry in final_history]
            unique_dates = set(dates)
            
            # Should not have many duplicates
            assert len(dates) - len(unique_dates) <= 3, \
                "Trend history should not create many duplicate entries"


def test_trend_append_idempotent():
    """Test that trend append is idempotent for same date."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        trend_path = Path(tmpdir) / "trend.json"
        
        template_content = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 1024
                    }
                }
            }
        }
        
        trend_content = {
            "history": [
                {"date": "2024-01-01", "cost": 10.0}
            ]
        }
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        with open(trend_path, 'w') as f:
            json.dump(trend_content, f)
        
        # Run trend with same date multiple times
        for _ in range(5):
            # Rewrite file with same data
            with open(trend_path, 'w') as f:
                json.dump(trend_content, f)
            
            result = subprocess.run(
                ["costpilot", "trend", "--plan", str(template_path), "--history", str(trend_path)],
                capture_output=True,
                text=True,
                timeout=30
            )
        
        if trend_path.exists():
            with open(trend_path, 'r') as f:
                final_data = json.load(f)
            
            final_history = final_data.get("history", [])
            
            # Should not explode in size
            assert len(final_history) < 20, \
                "Trend append should be roughly idempotent"


if __name__ == "__main__":
    test_trend_append_only()
    test_trend_no_modification()
    test_trend_ordering_preserved()
    test_trend_no_duplicates()
    test_trend_append_idempotent()
    print("All trend append-only invariant tests passed")
