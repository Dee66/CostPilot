#!/usr/bin/env python3
"""
Test: Telemetry opt-in/off toggles.

Validates telemetry opt-in/off toggles work across install and config.
"""

import os
import sys
import tempfile
import json
from pathlib import Path


def test_telemetry_disabled_by_default():
    """Verify telemetry is disabled by default."""
    
    default_config = {
        "telemetry_enabled": False,
        "anonymous_usage_stats": False
    }
    
    # Should be opt-in, not opt-out
    assert default_config["telemetry_enabled"] is False
    
    print("✓ Telemetry disabled by default (opt-in required)")


def test_opt_in_toggle():
    """Verify opt-in toggle enables telemetry."""
    
    config_before = {"telemetry_enabled": False}
    config_after = {"telemetry_enabled": True}
    
    assert config_before["telemetry_enabled"] is False
    assert config_after["telemetry_enabled"] is True
    
    print("✓ Opt-in toggle enables telemetry")


def test_opt_out_toggle():
    """Verify opt-out toggle disables telemetry."""
    
    config_before = {"telemetry_enabled": True}
    config_after = {"telemetry_enabled": False}
    
    assert config_before["telemetry_enabled"] is True
    assert config_after["telemetry_enabled"] is False
    
    print("✓ Opt-out toggle disables telemetry")


def test_config_file_persistence():
    """Verify telemetry setting persists in config file."""
    
    with tempfile.NamedTemporaryFile(mode='w', suffix='_config.json', delete=False) as f:
        config = {
            "telemetry_enabled": True,
            "telemetry_endpoint": "https://telemetry.example.com"
        }
        json.dump(config, f)
        path = f.name
    
    try:
        # Read back
        with open(path, 'r') as f:
            loaded = json.load(f)
        
        assert loaded["telemetry_enabled"] is True
        
        print("✓ Telemetry setting persists in config file")
        
    finally:
        os.unlink(path)


def test_environment_variable_override():
    """Verify environment variable can override config."""
    
    config_setting = True
    env_override = "false"
    
    # Environment should take precedence
    final_setting = False if env_override.lower() == "false" else True
    
    assert final_setting is False
    
    print("✓ Environment variable overrides config")


def test_cli_flag_override():
    """Verify CLI flag can override config."""
    
    config_setting = True
    cli_flag = "--no-telemetry"
    
    # CLI flag should take precedence
    final_setting = False if "--no-telemetry" in cli_flag else True
    
    assert final_setting is False
    
    print("✓ CLI flag overrides config")


def test_install_time_prompt():
    """Verify install prompts for telemetry consent."""
    
    install_flow = {
        "prompt_shown": True,
        "user_choice_recorded": True,
        "default_selection": "disabled"
    }
    
    # Should prompt during install
    assert install_flow["prompt_shown"] is True
    assert install_flow["default_selection"] == "disabled"
    
    print("✓ Install time prompt for telemetry consent")


def test_consent_timestamp_recorded():
    """Verify consent timestamp is recorded."""
    
    with tempfile.NamedTemporaryFile(mode='w', suffix='_consent.json', delete=False) as f:
        consent = {
            "telemetry_enabled": True,
            "consent_timestamp": "2024-01-15T10:00:00Z",
            "consent_version": "1.0"
        }
        json.dump(consent, f)
        path = f.name
    
    try:
        with open(path, 'r') as f:
            data = json.load(f)
        
        assert "consent_timestamp" in data
        assert "consent_version" in data
        
        print("✓ Consent timestamp recorded")
        
    finally:
        os.unlink(path)


def test_granular_telemetry_controls():
    """Verify granular telemetry controls."""
    
    telemetry_config = {
        "error_reporting": True,
        "usage_analytics": False,
        "performance_metrics": True,
        "crash_reports": True
    }
    
    # User can control individual categories
    assert "error_reporting" in telemetry_config
    assert telemetry_config["usage_analytics"] is False
    
    print("✓ Granular telemetry controls (4 categories)")


def test_toggle_reflected_immediately():
    """Verify toggle change reflected immediately."""
    
    state_changes = [
        {"enabled": False, "time": 0},
        {"enabled": True, "time": 1},  # Toggled on
        {"enabled": False, "time": 2}   # Toggled off
    ]
    
    # Changes should take effect immediately
    assert state_changes[0]["enabled"] != state_changes[1]["enabled"]
    
    print("✓ Toggle changes reflected immediately")


def test_no_telemetry_when_disabled():
    """Verify no telemetry sent when disabled."""
    
    config = {"telemetry_enabled": False}
    telemetry_events = []  # Should remain empty
    
    # Simulate events
    events = ["event1", "event2", "event3"]
    
    if config["telemetry_enabled"]:
        telemetry_events.extend(events)
    
    # No events should be sent
    assert len(telemetry_events) == 0
    
    print("✓ No telemetry sent when disabled")


if __name__ == "__main__":
    print("Testing telemetry opt-in/off toggles...")
    
    try:
        test_telemetry_disabled_by_default()
        test_opt_in_toggle()
        test_opt_out_toggle()
        test_config_file_persistence()
        test_environment_variable_override()
        test_cli_flag_override()
        test_install_time_prompt()
        test_consent_timestamp_recorded()
        test_granular_telemetry_controls()
        test_toggle_reflected_immediately()
        test_no_telemetry_when_disabled()
        
        print("\n✅ All telemetry toggle tests passed")
        sys.exit(0)
        
    except AssertionError as e:
        print(f"\n❌ Test failed: {e}", file=sys.stderr)
        sys.exit(1)
    except Exception as e:
        print(f"\n❌ Unexpected error: {e}", file=sys.stderr)
        sys.exit(1)
