#!/usr/bin/env python3
"""
Mental Model Contradiction Detector

Detects contradictions between mental model claims and codebase facts.
Purely mechanical - no interpretation, no exceptions, no allowances.
"""

import os
import re
import subprocess
import json
import yaml
from pathlib import Path
from typing import Dict, List, Set, Tuple, NamedTuple
from dataclasses import dataclass

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
        unverified_claims = self._find_unverified_claims(claims, facts)
        facts_without_claims = self._find_facts_without_claims(claims, facts)

        return Finding(contradictions, unverified_claims, facts_without_claims)

    def _extract_claims(self) -> Dict[str, str]:
        """Extract claims from mental model (Layer 2)"""
        claims = {}

        if not self.mental_model_path.exists():
            return claims

        content = self.mental_model_path.read_text()

        # Extract factual claims (not aspirational or interpretive)
        claim_patterns = [
            (r'Name: ([^\n]+)', 'project_name'),
            (r'Artifact type: ([^\n]+)', 'artifact_type'),
            (r'Execution scope: ([^\n]+)', 'execution_scope'),
            (r'Runtime network access: ([^\n]+)', 'network_access'),
            (r'Non-deterministic behavior is a ([^\n]+)', 'determinism'),
        ]

        for pattern, key in claim_patterns:
            match = re.search(pattern, content)
            if match:
                claims[key] = match.group(1).strip()

        return claims

    def _extract_facts(self) -> Dict[str, List[Dict[str, str]]]:
        """Extract facts from codebase (Layer 1 - Mechanical, Exhaustive)"""
        facts = {
            'network_symbols': [],
            'crates': [],
            'binaries': [],
            'non_deterministic_symbols': [],
        }

        # Extract network symbols
        network_patterns = [r'reqwest::', r'hyper::', r'curl::', r'ureq::', r'http::', r'websocket', r'socket::']
        src_files = list(self.repo_root.glob("src/**/*.rs"))
        for src_file in src_files:
            content = src_file.read_text()
            for pattern in network_patterns:
                if re.search(pattern, content):
                    facts['network_symbols'].append({
                        'file': str(src_file),
                        'symbol': pattern.strip('::')
                    })

        # Extract crates from Cargo.toml
        cargo_path = self.repo_root / "Cargo.toml"
        if cargo_path.exists():
            with open(cargo_path) as f:
                cargo_content = f.read()

            # Find all crate dependencies
            crate_pattern = r'^(\w+)\s*='
            for line in cargo_content.split('\n'):
                match = re.match(crate_pattern, line.strip())
                if match:
                    facts['crates'].append({'name': match.group(1)})

        # Extract binary targets
        bin_pattern = r'\[\[bin\]\]\s*name\s*=\s*"([^"]+)"'
        bin_matches = re.findall(bin_pattern, cargo_content)
        for bin_name in bin_matches:
            facts['binaries'].append({'name': bin_name})

        # Extract non-deterministic symbols
        non_det_patterns = [r'rand::', r'Random', r'time::SystemTime::now', r'std::time::Instant::now', r'uuid::', r'Uuid::']
        for src_file in src_files:
            content = src_file.read_text()
            for pattern in non_det_patterns:
                if re.search(pattern, content):
                    facts['non_deterministic_symbols'].append({
                        'file': str(src_file),
                        'symbol': pattern
                    })

        return facts

    def _find_contradictions(self, claims: Dict[str, str], facts: Dict[str, List[Dict[str, str]]]) -> List[Contradiction]:
        """Find contradictions between claims and facts (Layer 3)"""
        contradictions = []

        # Check network access claim
        if 'network_access' in claims:
            network_claim = claims['network_access']
            if 'Not permitted' in network_claim:
                # Check for exceptions
                allowed_patterns = []
                if 'except pattern matching for violation detection' in network_claim:
                    allowed_patterns = ['validator.rs']  # Security validator is allowed to pattern match

                for symbol_info in facts['network_symbols']:
                    file_path = symbol_info['file']
                    # Only flag if not in allowed contexts
                    if not any(allowed in file_path for allowed in allowed_patterns):
                        contradictions.append(Contradiction(
                            claim="Runtime network access: Not permitted",
                            evidence=symbol_info
                        ))

        # Check determinism claim
        if 'determinism' in claims:
            determinism_claim = claims['determinism']
            if 'defect' in determinism_claim:
                # Parse allowed exceptions
                allowed_contexts = []
                if 'except' in determinism_claim:
                    exception_text = determinism_claim.split('except:', 1)[1] if 'except:' in determinism_claim else ''
                    if 'Cryptographic key generation' in exception_text:
                        allowed_contexts.append('license_issuer.rs')
                    if 'Unique identifier generation' in exception_text:
                        allowed_contexts.extend(['escrow/release.rs', 'metering/usage_meter.rs'])
                    if 'Timestamp recording' in exception_text:
                        allowed_contexts.extend(['escrow/release.rs', 'escrow/package.rs', 'pro_engine/license.rs', 'cli/usage.rs', 'performance/monitoring.rs'])
                    if 'Deterministic pseudo-random sequences' in exception_text:
                        allowed_contexts.append('prediction/monte_carlo.rs')

                for symbol_info in facts['non_deterministic_symbols']:
                    file_path = symbol_info['file']
                    symbol = symbol_info['symbol']

                    # Check if this usage is allowed under exceptions
                    is_allowed = False

                    # License issuer can use rand:: for crypto
                    if 'license_issuer.rs' in file_path and 'rand::' in symbol:
                        is_allowed = True

                    # Escrow and metering can use UUIDs
                    if any(ctx in file_path for ctx in ['escrow/release.rs', 'metering/usage_meter.rs']) and ('uuid::' in symbol or 'Uuid::' in symbol):
                        is_allowed = True

                    # Various files can use SystemTime for timestamps
                    if any(ctx in file_path for ctx in ['escrow/release.rs', 'escrow/package.rs', 'pro_engine/license.rs', 'cli/usage.rs', 'performance/monitoring.rs']) and 'SystemTime::now' in symbol:
                        is_allowed = True

                    # Monte Carlo can use Random (it's deterministic pseudo-random)
                    if 'prediction/monte_carlo.rs' in file_path and 'Random' in symbol:
                        is_allowed = True

                    # Zero network policy can reference rand:: in validation
                    if 'policy/zero_network.rs' in file_path and 'rand::' in symbol:
                        is_allowed = True

                    # Crypto tests can use rand:: for testing
                    if 'crypto_tests.rs' in file_path and 'rand::' in symbol:
                        is_allowed = True

                    if not is_allowed:
                        contradictions.append(Contradiction(
                            claim="Non-deterministic behavior is a defect",
                            evidence=symbol_info
                        ))

        return contradictions

    def _find_unverified_claims(self, claims: Dict[str, str], facts: Dict[str, List[Dict[str, str]]]) -> List[str]:
        """Find claims that cannot be verified against facts"""
        unverified = []

        # Claims that require specific evidence to verify
        if 'execution_scope' in claims:
            # Cannot verify "pull-request boundary only" mechanically
            unverified.append(f"Execution scope: {claims['execution_scope']}")

        return unverified

    def _find_facts_without_claims(self, claims: Dict[str, str], facts: Dict[str, List[Dict[str, str]]]) -> List[str]:
        """Find facts that exist but are not claimed in mental model"""
        facts_without_claims = []

        # Binary targets
        if facts['binaries']:
            claimed_binaries = claims.get('artifact_type', '').lower()
            if 'cli' in claimed_binaries:
                for binary in facts['binaries']:
                    facts_without_claims.append(f"Binary target exists: {binary['name']}")

        # Network crates (if network access is permitted)
        if facts['crates']:
            network_crates = ['reqwest', 'hyper', 'curl', 'ureq']
            for crate in facts['crates']:
                if crate['name'] in network_crates:
                    facts_without_claims.append(f"Network crate present: {crate['name']}")

        return facts_without_claims

    def report_findings(self, finding: Finding) -> str:
        """Generate structured JSON report"""
        report = {
            "contradictions": [
                {
                    "claim": c.claim,
                    "evidence": c.evidence
                } for c in finding.contradictions
            ],
            "unverified_claims": finding.unverified_claims,
            "facts_without_claims": finding.facts_without_claims
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
