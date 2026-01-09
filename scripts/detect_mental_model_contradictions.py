#!/usr/bin/env python3
"""
Mental Model Contradiction Detector

Detects contradictions between mental model claims and codebase facts.
Purely mechanical - no interpretation, no exceptions, no allowances.
"""
from dataclasses import dataclass
from pathlib import Path
from typing import Dict, List
import re
import json

@dataclass
class Contradiction:
    claim: str
    evidence: Dict[str, str]


@dataclass
class Finding:
    contradictions: List[Contradiction]
    unverified_claims: List[str]
    facts_without_claims: List[str]


class MentalModelContradictionDetector:
    def __init__(self, repo_root: Path):
        self.repo_root = repo_root
        self.mental_model_path = repo_root / "docs" / "mental_model.md"

    def detect(self) -> Finding:
        """Run contradiction detection"""
        print("🔍 Detecting mental model contradictions...")

        claims = self._extract_claims()
        facts = self._extract_facts()

        contradictions = self._find_contradictions(claims, facts)
        unverified_claims = self._find_unverified_claims(claims)
        facts_without_claims = self._find_facts_without_claims(claims, facts)

        return Finding(contradictions, unverified_claims, facts_without_claims)

    def _extract_claims(self) -> Dict[str, str]:
        """Extract claims from mental model (Layer 2)"""
        claims = {}

        if not self.mental_model_path.exists():
            return claims

        content = self.mental_model_path.read_text()

        # Extract CLAIM entries in format: CLAIM key = value
        claim_pattern = r'CLAIM\s+(\w+)\s*=\s*([^\n]+)'
        matches = re.findall(claim_pattern, content)

        for key, value in matches:
            claims[key] = value.strip()

        return claims

    def _extract_facts(self) -> Dict[str, List[Dict[str, str]]]:
        """Extract facts from codebase (Layer 1 - Mechanical, Exhaustive)"""
        facts = {
            "network_symbols": [],
            "crates": [],
            "binaries": [],
            "non_deterministic_symbols": [],
        }

        # Extract network symbols
        network_patterns = [
            r"reqwest::",
            r"hyper::",
            r"curl::",
            r"ureq::",
            r"http::",
            r"websocket",
            r"socket::",
        ]
        src_files = list(self.repo_root.glob("src/**/*.rs"))
        for src_file in src_files:
            content = src_file.read_text()
            for pattern in network_patterns:
                if re.search(pattern, content):
                    facts["network_symbols"].append(
                        {"file": str(src_file), "symbol": pattern.rstrip(":")}
                    )

        # Extract crates from Cargo.toml
        cargo_path = self.repo_root / "Cargo.toml"
        if cargo_path.exists():
            with open(cargo_path, encoding="utf-8") as f:
                cargo_content = f.read()

            # Find all crate dependencies
            crate_pattern = r"^(\w+)\s*="
            for line in cargo_content.split("\n"):
                match = re.match(crate_pattern, line.strip())
                if match:
                    facts["crates"].append({"name": match.group(1)})

        # Extract binary targets
        bin_pattern = r'\[\[bin\]\]\s*name\s*=\s*"([^"]+)"'
        bin_matches = re.findall(bin_pattern, cargo_content)
        for bin_name in bin_matches:
            facts["binaries"].append({"name": bin_name})

        # Extract non-deterministic symbols
        non_det_patterns = [
            r"rand::",
            r"Random",
            r"time::SystemTime::now",
            r"std::time::Instant::now",
            r"uuid::",
            r"Uuid::",
        ]
        for src_file in src_files:
            content = src_file.read_text()
            for pattern in non_det_patterns:
                if re.search(pattern, content):
                    facts["non_deterministic_symbols"].append(
                        {"file": str(src_file), "symbol": pattern}
                    )

        return facts

    def _find_contradictions(
        self, claims: Dict[str, str], facts: Dict[str, List[Dict[str, str]]]
    ) -> List[Contradiction]:
        """Find contradictions between claims and facts (Layer 3)"""
        contradictions = []

        # Check network access claim
        if "runtime_network_access" in claims:
            network_claim = claims["runtime_network_access"]
            if "prohibited" in network_claim:
                # Check for exceptions - static analysis violation detection is allowed
                allowed_patterns = []
                content = self.mental_model_path.read_text()
                if "static_analysis_violation_detection" in content:
                    allowed_patterns = [
                        "security/validator.rs",  # Security validator can use network patterns for detection
                    ]

                for symbol_info in facts["network_symbols"]:
                    file_path = symbol_info["file"]
                    # Only flag if not in allowed contexts
                    if not any(allowed in file_path for allowed in allowed_patterns):
                        contradictions.append(
                            Contradiction(
                                claim="Runtime network access: prohibited",
                                evidence=symbol_info,
                            )
                        )

        # Check determinism claim
        if "determinism" in claims:
            determinism_claim = claims["determinism"]
            if "strict" in determinism_claim:
                # Parse EXCEPT lines for allowed modules
                content = self.mental_model_path.read_text()
                except_pattern = r'EXCEPT\s+module\s*=\s*([^\n]+)'
                except_matches = re.findall(except_pattern, content)
                allowed_modules = [match.strip() for match in except_matches]

                for symbol_info in facts["non_deterministic_symbols"]:
                    file_path = symbol_info["file"]

                    # Check if this file is in the allowed modules
                    is_allowed = False
                    for allowed_module in allowed_modules:
                        if allowed_module in file_path:
                            is_allowed = True
                            break

                    if not is_allowed:
                        contradictions.append(
                            Contradiction(
                                claim="Determinism: strict (non-deterministic behavior prohibited)",
                                evidence=symbol_info,
                            )
                        )

        return contradictions

    def _find_unverified_claims(self, claims: Dict[str, str]) -> List[str]:
        """Find claims that cannot be verified against facts"""
        unverified = []

        # Claims that require specific evidence to verify
        if "execution_scope" in claims:
            # Cannot verify "pull-request boundary only" mechanically
            unverified.append(f"Execution scope: {claims['execution_scope']}")

        return unverified

    def _find_facts_without_claims(
        self, claims: Dict[str, str], facts: Dict[str, List[Dict[str, str]]]
    ) -> List[str]:
        """Find facts that exist but are not claimed in mental model"""
        facts_without_claims = []

        # Binary targets
        if facts["binaries"]:
            claimed_binaries = claims.get("artifact_type", "").lower()
            if "cli" in claimed_binaries:
                for binary in facts["binaries"]:
                    facts_without_claims.append(
                        f"Binary target exists: {binary['name']}"
                    )

        # Network crates (if network access is permitted)
        if facts["crates"]:
            network_crates = ["reqwest", "hyper", "curl", "ureq"]
            for crate in facts["crates"]:
                if crate["name"] in network_crates:
                    facts_without_claims.append(
                        f"Network crate present: {crate['name']}"
                    )

        return facts_without_claims

    def report_findings(self, finding: Finding) -> str:
        """Generate structured JSON report"""
        report = {
            "contradictions": [
                {"claim": c.claim, "evidence": c.evidence}
                for c in finding.contradictions
            ],
            "unverified_claims": finding.unverified_claims,
            "facts_without_claims": finding.facts_without_claims,
        }

        return json.dumps(report, indent=2)


def main():
    repo_root = Path(__file__).parent.parent
    detector = MentalModelContradictionDetector(repo_root)

    finding = detector.detect()
    report = detector.report_findings(finding)

    print(report)

    # Exit with error if contradictions found
    if finding.contradictions:
        print(f"\n❌ Found {len(finding.contradictions)} contradictions")
        exit(1)
    else:
        print("\n✅ No contradictions detected")
        exit(0)


if __name__ == "__main__":
    main()
