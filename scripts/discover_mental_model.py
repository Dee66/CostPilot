#!/usr/bin/env python3
"""
Mental Model Discovery Tool

Automatically scans codebase to discover raw, normalized facts.
Outputs facts for manual review during mental model creation.
"""

import os
import re
import subprocess
import json
import yaml
from pathlib import Path
from typing import Dict, List, Set, Tuple, Optional
from dataclasses import dataclass

@dataclass
class MentalModelFact:
    section: str
    fact: str
    evidence: str

class MentalModelDiscovery:
    def __init__(self, repo_root: Path):
        self.repo_root = repo_root
        self.mental_model_path = repo_root / "docs" / "mental_model.md"
        self.discovered_facts: List[MentalModelFact] = []

    def discover(self) -> List[MentalModelFact]:
        """Run all discovery checks"""
        print("🔬 Discovering mental model facts...")

        self._discover_project_identity()
        self._discover_execution_model()
        self._discover_security_boundary()
        self._discover_edition_model()
        self._discover_volatility()

        return self.discovered_facts

    def _discover_project_identity(self):
        """Discover project identity facts"""
        # Check Cargo.toml for project name and type
        cargo_path = self.repo_root / "Cargo.toml"
        if cargo_path.exists():
            with open(cargo_path) as f:
                cargo_data = f.read()

            # Extract binary targets
            bin_matches = re.findall(r'\[\[bin\]\]\s*name\s*=\s*"([^"]+)"', cargo_data)
            if bin_matches:
                self.discovered_facts.append(MentalModelFact(
                    section="1. Project Identity",
                    fact=f"Binary targets: {', '.join(bin_matches)}",
                    evidence=f"Cargo.toml contains binary targets: {bin_matches}"
                ))

    def _discover_execution_model(self):
        """Discover execution model facts"""
        # Check for IaC format support
        src_files = list(self.repo_root.glob("src/**/*.rs"))
        supported_formats = set()

        for src_file in src_files:
            content = src_file.read_text()

            # Look for format-specific code
            if 'terraform' in content.lower():
                supported_formats.add('Terraform')
            if 'cdk' in content.lower() or 'cloudformation' in content.lower():
                supported_formats.add('CloudFormation')
            if 'kubernetes' in content.lower() or 'k8s' in content.lower():
                supported_formats.add('Kubernetes')

        if supported_formats:
            self.discovered_facts.append(MentalModelFact(
                section="3. Execution Model",
                fact=f"Supported IaC formats: {', '.join(sorted(supported_formats))}",
                evidence=f"Code references found for: {supported_formats}"
            ))

        # Check for network access patterns
        network_indicators = []
        for src_file in src_files:
            content = src_file.read_text()
            if any(pattern in content for pattern in ['reqwest', 'hyper', 'http']):
                network_indicators.append(str(src_file))

        if network_indicators:
            self.discovered_facts.append(MentalModelFact(
                section="6. Security Boundary",
                fact=f"Network libraries found in: {network_indicators[:3]}",
                evidence=f"Network libraries found in: {network_indicators[:3]}"
            ))

    def _discover_security_boundary(self):
        """Discover security boundary facts"""
        # Look for ZeroCostGuard
        src_files = list(self.repo_root.glob("src/**/*.rs"))
        zerocostguard_found = False

        for src_file in src_files:
            content = src_file.read_text()
            if 'ZeroCostGuard' in content:
                zerocostguard_found = True
                break

        if zerocostguard_found:
            self.discovered_facts.append(MentalModelFact(
                section="6. Security Boundary",
                fact="ZeroCostGuard references found in source code",
                evidence="ZeroCostGuard references found in source code"
            ))

        # Check for WASM usage
        wasm_indicators = []
        for src_file in src_files:
            content = src_file.read_text()
            if 'wasm' in content.lower():
                wasm_indicators.append(str(src_file))

        if wasm_indicators:
            self.discovered_facts.append(MentalModelFact(
                section="6. Security Boundary",
                fact=f"WASM references found in: {wasm_indicators[:3]}",
                evidence=f"WASM references found in: {wasm_indicators[:3]}"
            ))

    def _discover_edition_model(self):
        """Discover edition/licensing facts"""
        src_files = list(self.repo_root.glob("src/**/*.rs"))
        edition_indicators = []

        for src_file in src_files:
            content = src_file.read_text()
            if 'edition' in content.lower() or 'premium' in content.lower():
                edition_indicators.append(str(src_file))

        if edition_indicators:
            self.discovered_facts.append(MentalModelFact(
                section="7. Edition & Licensing Model",
                fact=f"Edition references found in: {edition_indicators[:3]}",
                evidence=f"Edition references found in: {edition_indicators[:3]}"
            ))

    def _discover_volatility(self):
        """Discover volatile areas"""
        # Check command surface area
        main_rs = self.repo_root / "src" / "bin" / "costpilot.rs"
        if main_rs.exists():
            content = main_rs.read_text()

            # Count subcommands (rough approximation)
            subcommand_count = len(re.findall(r'Commands::', content))

            if subcommand_count > 10:
                self.discovered_facts.append(MentalModelFact(
                    section="8. Known Volatility",
                    fact=f"Found {subcommand_count} command variants in main.rs",
                    evidence=f"Found {subcommand_count} command variants in main.rs"
                ))

    def output_facts(self) -> str:
        """Output raw discovered facts"""
        if not self.discovered_facts:
            return "No facts discovered"

        fact_lines = ["# Discovered Raw Facts", ""]

        for fact in self.discovered_facts:
            fact_lines.extend([
                f"## Section: {fact.section}",
                f"**Fact:** {fact.fact}",
                f"**Evidence:** {fact.evidence}",
                ""
            ])

        return "\n".join(fact_lines)

def main():
    repo_root = Path(__file__).parent.parent
    discovery = MentalModelDiscovery(repo_root)

    facts = discovery.discover()
    output = discovery.output_facts()

    print(f"📊 Discovered {len(facts)} raw facts")
    print("\n" + "="*50)
    print(output)

    # Save to file for review
    output_path = repo_root / "discovered_facts.md"
    with open(output_path, 'w') as f:
        f.write(output)
    print(f"\n💾 Facts saved to {output_path}")

if __name__ == "__main__":
    main()
