# CostPilot

Deterministic, audit-grade cost governance at the pull-request boundary.

CostPilot analyzes infrastructure-as-code changes **before they merge** and blocks only **irreversible cloud cost regressions**.  
When no meaningful risk exists, it stays silent.

---

## Overview

CostPilot is a PR-time cost governance engine for infrastructure as code.

Given a pull request and its associated plan, CostPilot:

- Detects cost-impacting changes
- Predicts monthly cost deltas
- Explains causality and propagation
- Enforces policy or SLO-based blocking **only when configured or required for safety**

All outputs are deterministic, reproducible, and suitable for CI enforcement.

---

## What CostPilot Does

- Operates strictly at the pull-request boundary
- Reasons from diffs and plans only
- Produces audit-grade outputs
- Supports advisory and blocking modes
- Remains silent for no-op or cosmetic changes

---

## What CostPilot Does Not Do

- No live infrastructure optimization
- No historical billing analysis
- No runtime agents or background collectors
- No cloud credentials or IAM access required
- No noisy or speculative blocking

Silence is a valid and expected outcome.

---

## Blocking Semantics

CostPilot blocks CI **only** in the following cases:

1. Explicit governance configuration (policy or SLO block mode)
2. Hard safety violations:
   - Invalid or inconsistent inputs
   - Determinism violations
   - Drift in protected artifacts
   - Internal execution errors

All other findings are advisory by default.

---

## Trust Model

CostPilot enforces a single trust chain:

**Detect → Predict → Explain**

If any stage is insufficient, CostPilot refuses to act.

---

## Canonical Demo

The authoritative reference for CostPilot behavior is the **CostPilot Demo** repository.

- Deterministic PR scenario
- Frozen, hash-stable outputs
- Zero-IAM, offline-safe execution
- CI-enforced invariants

Demo repository:  
https://github.com/Dee66/costpilotdemo

Live demo UI:  
https://dee66.github.io/costpilotdemo/

All screenshots, videos, and launch materials originate from this demo.

---

## Supported Scope (Launch)

- Terraform-based infrastructure diffs
- Pull-request–time analysis
- Deterministic detect / predict / explain outputs
- Policy-based governance
- Restrained autofix previews (explicitly scoped)

Additional IaC formats are intentionally deferred.

---

## Installation

CostPilot is distributed as a single CLI binary.

Installation methods and package managers are documented per-release.  
Correctness and determinism take precedence over distribution convenience.

---

## Documentation

- Demo walkthrough: see CostPilot Demo
- Architecture: `docs/architecture.md`
- Policy model: `docs/policies.md`
- Determinism & drift guarantees: `docs/determinism.md`

---

## License

MIT License.
