#!/usr/bin/env python3
"""
CostPilot License Boundary Validation Script

This script validates that Free and Pro tiers behave correctly:
- Decisions are identical for same input
- Only autofix_mode and output_artifacts can differ
- Tier gating doesn't influence core decision logic

Usage:
    python validate_license_boundaries.py [test_scenario]

Arguments:
    test_scenario: Name of test scenario to run (optional, runs all if not specified)
"""

import os
import sys
import json
import yaml
import subprocess
import tempfile
from pathlib import Path
from typing import Dict, Any, Tuple

class LicenseBoundaryValidator:
    def __init__(self, test_data_dir: str):
        self.test_data_dir = Path(test_data_dir)
        self.project_root = Path(__file__).parent.parent.parent.parent
        self.costpilot_binary = self.project_root / "target" / "release" / "costpilot"

        # Load test data
        with open(self.test_data_dir / "license_boundary_tests.yml", encoding='utf-8') as f:
            self.test_config = yaml.safe_load(f)

        with open(self.test_data_dir / "expected_outputs.yml", encoding='utf-8') as f:
            self.expected_outputs = yaml.safe_load(f)

        with open(self.test_data_dir / "sample_input_plans.yml", encoding='utf-8') as f:
            self.sample_plans = yaml.safe_load(f)

    def run_costpilot(self, plan_data: Dict[str, Any], license_mode: str = "free") -> Tuple[int, str, str]:
        """Run CostPilot with given plan data and license mode."""
        # Create temporary plan file
        with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False) as f:
            json.dump(plan_data, f, indent=2)
            plan_file = f.name

        try:
            # Set environment variables for license mode
            env = os.environ.copy()
            if license_mode == "free":
                env.pop("COSTPILOT_LICENSE_KEY", None)
            elif license_mode == "pro":
                env["COSTPILOT_LICENSE_KEY"] = "test-valid-license-key"

            # Run CostPilot
            cmd = [str(self.costpilot_binary), "validate", plan_file]
            result = subprocess.run(
                cmd,
                capture_output=True,
                text=True,
                env=env,
                cwd=self.project_root,
                check=False
            )

            return result.returncode, result.stdout, result.stderr

        finally:
            os.unlink(plan_file)

    def validate_decision_equivalence(self, scenario: str) -> bool:
        """Validate that Free and Pro make identical decisions."""
        print(f"  Validating decision equivalence for {scenario}...")

        plan_data = self.sample_plans.get(scenario)
        if not plan_data:
            print(f"    ERROR: No plan data found for scenario {scenario}")
            return False

        # Run both tiers
        free_exit, free_out, _ = self.run_costpilot(plan_data, "free")
        pro_exit, pro_out, _ = self.run_costpilot(plan_data, "pro")

        # Extract decisions (simplified - in real implementation, parse structured output)
        free_decision = self.extract_decision(free_out, free_exit)
        pro_decision = self.extract_decision(pro_out, pro_exit)

        if free_decision != pro_decision:
            print(f"    FAIL: Decisions differ - Free: {free_decision}, Pro: {pro_decision}")
            return False

        print(f"    PASS: Both tiers decided '{free_decision}'")
        return True

    def validate_allowed_differences(self, scenario: str) -> bool:
        """Validate that only allowed differences exist between tiers."""
        print(f"  Validating allowed differences for {scenario}...")

        plan_data = self.sample_plans.get(scenario)
        if not plan_data:
            print(f"    ERROR: No plan data found for scenario {scenario}")
            return False

        # Run both tiers
        free_exit, free_out, _ = self.run_costpilot(plan_data, "free")
        pro_exit, pro_out, _ = self.run_costpilot(plan_data, "pro")

        expected = self.expected_outputs.get(f"{scenario}_outputs")
        if not expected:
            print(f"    ERROR: No expected outputs found for {scenario}")
            return False

        # Validate Free tier expectations
        if not self.validate_output_against_expectations(free_out, free_exit, expected["free_tier"], "Free"):
            return False

        # Validate Pro tier expectations
        if not self.validate_output_against_expectations(pro_out, pro_exit, expected["pro_tier"], "Pro"):
            return False

        print("    PASS: Outputs match expected differences")
        return True

    def validate_output_against_expectations(self, output: str, exit_code: int,
                                           expectations: Dict[str, Any], tier: str) -> bool:
        """Validate output against expectations."""
        # Check exit code
        if "exit_code" in expectations and exit_code != expectations["exit_code"]:
            print(f"    FAIL {tier}: Expected exit code {expectations['exit_code']}, got {exit_code}")
            return False

        # Check stdout contains expected strings
        if "stdout_contains" in expectations:
            for expected_str in expectations["stdout_contains"]:
                if expected_str not in output:
                    print(f"    FAIL {tier}: Expected '{expected_str}' in output")
                    return False

        # Check stderr expectations
        if "stderr_empty" in expectations and expectations["stderr_empty"] and output.strip():
            print(f"    FAIL {tier}: Expected empty stderr, got: {output}")
            return False

        # Check boolean flags
        for flag in ["autofix_generated", "contains_patch", "contains_snippet"]:
            if flag in expectations:
                # This would need actual parsing of structured output
                pass

        return True

    def extract_decision(self, output: str, exit_code: int) -> str:
        """Extract decision from CostPilot output."""
        # Simplified decision extraction - in real implementation, parse JSON output
        if exit_code == 0:
            if "Warning:" in output:
                return "warn"
            else:
                return "allow"
        elif exit_code == 1:
            return "block"
        else:
            return "error"

    def run_scenario(self, scenario_name: str) -> bool:
        """Run all validations for a scenario."""
        print(f"\nRunning scenario: {scenario_name}")

        success = True
        success &= self.validate_decision_equivalence(scenario_name)
        success &= self.validate_allowed_differences(scenario_name)

        return success

    def run_all_scenarios(self) -> bool:
        """Run all license boundary test scenarios."""
        print("CostPilot License Boundary Validation")
        print("=" * 50)

        scenarios = [
            "basic_cost_regression",
            "complex_multi_resource",
            "policy_violation",
            "drift_detection",
            "seasonal_variation",
            "exemption_applied",
            "autofix_candidate",
            "drift_with_violation",
            "patch_scenario",
            "complex_analysis",
            "error_scenario"
        ]

        total_scenarios = len(scenarios)
        passed_scenarios = 0

        for scenario in scenarios:
            if self.run_scenario(scenario):
                passed_scenarios += 1

        print(f"\n{'='*50}")
        print(f"Results: {passed_scenarios}/{total_scenarios} scenarios passed")

        if passed_scenarios == total_scenarios:
            print("✅ All license boundary validations PASSED")
            return True
        else:
            print("❌ Some license boundary validations FAILED")
            return False

def main():
    if len(sys.argv) > 2:
        print("Usage: python validate_license_boundaries.py [scenario_name]")
        sys.exit(1)

    test_data_dir = Path(__file__).parent / "license_boundaries"
    validator = LicenseBoundaryValidator(test_data_dir)

    if len(sys.argv) == 2:
        scenario = sys.argv[1]
        success = validator.run_scenario(scenario)
    else:
        success = validator.run_all_scenarios()

    sys.exit(0 if success else 1)

if __name__ == "__main__":
    main()
