#!/usr/bin/env python3
"""
Mental Model Conflict Detector

Reports discrepancies between mental model claims and codebase state.
Provides neutral state information for external policy decisions.
"""

import os
import re
import subprocess
import hashlib
from pathlib import Path
from typing import Dict, List, Set, Tuple, Optional
from dataclasses import dataclass

@dataclass
class Conflict:
    description: str
    mental_model_claim: str
    codebase_evidence: str

class ConflictDetector:
    def __init__(self, repo_root: Path):
        self.repo_root = repo_root
        self.mental_model_path = repo_root / "docs" / "mental_model.md"
        self.conflicts: List[Conflict] = []

    def detect_conflicts(self) -> List[Conflict]:
        """Run all conflict detection checks"""
        print("🔍 Detecting mental model conflicts...")

        if not self.mental_model_path.exists():
            self.conflicts.append(Conflict(
                description="Mental model file missing",
                mental_model_claim="Mental model exists at docs/mental_model.md",
                codebase_evidence="File not found"
            ))
            return self.conflicts

        mental_model_content = self.mental_model_path.read_text()

        self._check_execution_scope_conflicts(mental_model_content)
        self._check_network_access_conflicts(mental_model_content)
        self._check_determinism_conflicts(mental_model_content)
        self._check_security_boundary_conflicts(mental_model_content)
        self._check_edition_model_conflicts(mental_model_content)

        return self.conflicts

    def _check_execution_scope_conflicts(self, mental_model: str):
        """Check execution scope claims"""
        # Mental model claims "pull-request boundary only"
        if "pull-request boundary only" in mental_model:
            # Check if code can operate outside PR context
            src_files = list(self.repo_root.glob("src/**/*.rs"))
            for src_file in src_files:
                content = src_file.read_text()
                # Look for operations that might work outside PR context
                if any(pattern in content for pattern in [
                    'std::env::args', 'clap::', 'structopt::'
                ]):
                    # This is expected for CLI - not a conflict
                    continue

                # Look for file system operations that might be global
                if 'std::fs::' in content and 'git' not in content.lower():
                    self.conflicts.append(Conflict(
                        description="Potential global file system operations detected",
                        mental_model_claim="Execution scope: Pull-request boundary only",
                        codebase_evidence=f"File system operations in {src_file}"
                    ))

    def _check_network_access_conflicts(self, mental_model: str):
        """Check network access claims"""
        if "Runtime network access: Not permitted" in mental_model:
            # Check for network libraries in Cargo.toml
            cargo_path = self.repo_root / "Cargo.toml"
            if cargo_path.exists():
                with open(cargo_path) as f:
                    cargo_content = f.read()

                network_crates = [
                    'reqwest', 'hyper', 'curl', 'ureq', 'tokio-tungstenite'
                ]

                for crate in network_crates:
                    if crate in cargo_content:
                        self.conflicts.append(Conflict(
                            description=f"Network crate '{crate}' found in dependencies",
                            mental_model_claim="Runtime network access: Not permitted",
                            codebase_evidence=f"Crate '{crate}' in Cargo.toml"
                        ))

    def _check_determinism_conflicts(self, mental_model: str):
        """Check determinism claims"""
        if "Non-deterministic behavior is a defect" in mental_model:
            src_files = list(self.repo_root.glob("src/**/*.rs"))
            for src_file in src_files:
                content = src_file.read_text()

                # Skip test files
                if 'test' in str(src_file).lower():
                    continue

                non_det_patterns = [
                    (r'rand::', 'Random number generation'),
                    (r'time::SystemTime::now', 'System time access'),
                    (r'std::time::Instant::now', 'Instant time access'),
                    (r'uuid::', 'UUID generation'),
                ]

                for pattern, description in non_det_patterns:
                    if re.search(pattern, content):
                        self.conflicts.append(Conflict(
                            description=f"Non-deterministic {description} in production code",
                            mental_model_claim="Non-deterministic behavior is a defect",
                            codebase_evidence=f"Pattern '{pattern}' found in {src_file}"
                        ))

    def _check_security_boundary_conflicts(self, mental_model: str):
        """Check security boundary claims"""
        if "ZeroCostGuard executes before command execution" in mental_model:
            main_rs = self.repo_root / "src" / "bin" / "costpilot.rs"
            if main_rs.exists():
                content = main_rs.read_text()
                if 'ZeroCostGuard' not in content:
                    self.conflicts.append(Conflict(
                        description="ZeroCostGuard not found in main execution path",
                        mental_model_claim="ZeroCostGuard executes before command execution",
                        codebase_evidence="ZeroCostGuard not referenced in main.rs"
                    ))

    def _check_edition_model_conflicts(self, mental_model: str):
        """Check edition/licensing model claims"""
        if "No server-side license validation occurs at runtime" in mental_model:
            src_files = list(self.repo_root.glob("src/**/*.rs"))
            for src_file in src_files:
                content = src_file.read_text()

                # Look for server communication patterns
                server_patterns = [
                    'http.*post', 'http.*get', 'api\.', 'server',
                    'license.*server', 'validation.*server'
                ]

                for pattern in server_patterns:
                    if re.search(pattern, content, re.IGNORECASE):
                        self.conflicts.append(Conflict(
                            description="Potential server-side validation detected",
                            mental_model_claim="No server-side license validation occurs at runtime",
                            codebase_evidence=f"Server pattern '{pattern}' in {src_file}"
                        ))

    def report_conflicts(self) -> str:
        """Generate conflict report"""
        if not self.conflicts:
            return "✅ No conflicts detected"

        report_lines = ["# Mental Model Conflicts Report", ""]

        report_lines.append(f"## Summary: {len(self.conflicts)} conflicts found")
        report_lines.append("")

        for i, conflict in enumerate(self.conflicts, 1):
            report_lines.extend([
                f"## Conflict {i}",
                f"**Description:** {conflict.description}",
                f"**Mental Model Claim:** {conflict.mental_model_claim}",
                f"**Codebase Evidence:** {conflict.codebase_evidence}",
                ""
            ])

        return "\n".join(report_lines)

def main():
    repo_root = Path(__file__).parent.parent
    detector = ConflictDetector(repo_root)

    conflicts = detector.detect_conflicts()
    report = detector.report_conflicts()

    print(report)

    # Save report if conflicts found
    if conflicts:
        output_path = repo_root / "mental_model_conflicts.md"
        with open(output_path, 'w') as f:
            f.write(report)
        print(f"\n💾 Conflict report saved to {output_path}")

if __name__ == "__main__":
    main()
