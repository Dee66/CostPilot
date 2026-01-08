# CostPilot

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Status](https://img.shields.io/badge/status-stable-green.svg)
![Version](https://img.shields.io/badge/status-stable-green.svg)

**Deterministic, audit-grade cost governance at the pull-request boundary.**

CostPilot analyzes infrastructure-as-code changes **before they merge** and blocks only **irreversible cloud cost regressions**.
When no meaningful risk exists, it stays silent.

Silence is a valid outcome.

---

## Overview

CostPilot is a **local, offline CLI tool** for PR-time cost governance.

It exists for one reason:
to surface *real*, *irreversible* cost risk **at review time**, not after deployment.

Given a pull request and its associated infrastructure plan, CostPilot:

- Detects cost-impacting changes
- Predicts monthly cost deltas using deterministic heuristics
- Explains causality and propagation
- Enforces policy or SLO-based blocking **only when explicitly configured or required for safety**

All outputs are deterministic, reproducible, and suitable for CI enforcement.

---

## What CostPilot Does

- Operates strictly at the pull-request boundary
- Reasons from diffs and plans only
- Produces audit-grade, reproducible outputs
- Supports advisory and blocking modes
- Remains silent for no-op or cosmetic changes

CostPilot does not reward noise.
It acts only when evidence is sufficient.

---

## What CostPilot Does *Not* Do

- No live infrastructure optimization
- No historical billing analysis
- No runtime agents or background collectors
- No cloud credentials or IAM access required
- No external service dependencies at runtime
- No speculative or noisy blocking

Everything runs locally.
Everything is inspectable.
Nothing phones home.

---

## Blocking Semantics

CostPilot blocks CI **only** in the following cases:

1. Explicit governance configuration (policy or SLO in block mode)
2. Hard safety violations:
   - Invalid or inconsistent inputs
   - Determinism violations
   - Drift in protected artifacts
   - Internal execution errors

All other findings are advisory by default.

This is intentional.
Blocking is a last resort, not a feature.

---

## Trust Model

CostPilot enforces a single trust chain:

**Detect → Predict → Explain**

If any stage lacks sufficient evidence, CostPilot refuses to act.

Confidence is earned, not assumed.

---

## Canonical Demo

The authoritative reference for CostPilot behavior is the **CostPilot Demo** repository.

It demonstrates a complete, frozen scenario:

- Deterministic PR inputs
- Hash-stable outputs
- Zero-IAM, offline-safe execution
- CI-enforced invariants

Demo repository:
https://github.com/Dee66/costpilotdemo

Live demo UI:
https://dee66.github.io/costpilotdemo

All screenshots, videos, and launch materials originate from this demo.

---

## Supported Scope (Launch)

- Terraform-based infrastructure diffs
- Pull-request–time analysis
- Deterministic detect / predict / explain outputs
- Static, file-based policy governance
- Explicitly scoped autofix previews

Additional IaC formats and features are intentionally deferred.

---

## Quick Start

### Install

CostPilot is distributed as a single native binary.

**Download pre-built binary:**

1. Visit [GitHub Releases](https://github.com/Dee66/costpilot/releases)
2. Download the appropriate archive for your platform (`.tar.gz` or `.zip`)
3. Extract and place the `costpilot` binary on your PATH

**Or build from source:**

```bash
git clone https://github.com/Dee66/CostPilot.git
cd CostPilot
cargo build --release
sudo cp target/release/costpilot /usr/local/bin/
```

Verify installation:

```bash
costpilot --version
