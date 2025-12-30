# CostPilot — Canonical Mental Model

## MODEL_STATE
mutable

This document is intentionally incomplete.

It records only stable, verified, non-derivable facts about the system.
Absence of information indicates UNKNOWN, not absence in the system.

This file is not a design document, roadmap, or product description.

## 0. Authority & Precedence

Order of truth:

1. Tests
2. Executable code
3. This file (`mental_model.md`)
4. Other documentation

If conflicts exist, they must be reported.
Conflicts must not be resolved implicitly.

## 1. Project Identity (Factual)

- Name: CostPilot
- Artifact type: Local CLI binary
- Execution scope: Pull-request boundary only
- Runtime network access: Not permitted (except pattern matching for violation detection)

## 2. Non-Goals (Hard Constraints)

CostPilot does NOT:

- Call live cloud provider APIs
- Perform runtime telemetry or analytics
- Run as a background service or daemon
- Maintain mutable external state
- Operate against live infrastructure

## 3. Execution Model

- Inputs: Infrastructure-as-code plans and diffs
  - Terraform is supported
  - Other IaC dialects may exist (see volatility)

- Processing:
  - Offline
  - Deterministic
  - Local-only

- Outputs:
  - Structured findings
  - Output MAY be empty

Silence is a valid and correct outcome.

## 4. Blocking Semantics

Blocking occurs ONLY when one of the following is true:

1. Explicit governance configuration enables blocking, OR
2. Safety or integrity invariants are violated

The following MUST NOT cause blocking by default:

- Cost magnitude
- Cost trend direction
- Confidence level
- Feature tier
- Advisory findings

Default behavior is advisory.

## 5. Determinism Contract

Given identical inputs:

- Outputs must be byte-identical
- Ordering must be stable
- Hashes must match across runs and platforms

Non-deterministic behavior is a defect, except:

- Cryptographic key generation (license issuer)
- Unique identifier generation (UUIDs for escrow receipts, metering events)
- Timestamp recording (SystemTime for audit trails, not for core logic)
- Deterministic pseudo-random sequences (Monte Carlo with fixed seeds)

## 6. Security Boundary (Factual)

- ZeroCostGuard executes before command execution
- Runtime network access is prohibited
- WASM execution, when used, is sandboxed
- WASM modules have no direct host system access

## 7. Edition & Licensing Model (Factual)

- Multiple editions exist
- Feature access is edition-gated
- Licensing is:
  - Offline
  - Signature-verified
  - Client-side only

No server-side license validation occurs at runtime.

## 8. Known Volatility

The following areas are expected to change over time:

- Supported IaC dialects
- Cost heuristic coverage
- Command surface area
- Edition feature boundaries

Changes in these areas must be reflected via deltas.
