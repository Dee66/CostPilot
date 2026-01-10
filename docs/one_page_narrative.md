CostPilot - One-Page Launch Narrative (Internal)
What CostPilot Is

CostPilot provides deterministic, audit-grade cost governance at the pull-request boundary.
It evaluates infrastructure changes before merge, identifies irreversible cloud cost commitments, and blocks only when necessary.
When no material risk exists, CostPilot stays silent.

CostPilot is not a monitoring tool, an optimizer, or a FinOps dashboard.
It is a governance engine for pre-deployment decisions.

The Problem It Solves

Modern infrastructure changes often create recurring financial commitments that:

Appear reasonable in isolation

Pass human code review

Cannot be safely reversed after merge

Only become visible after deployment or billing cycles

Existing tools either:

Speak after the fact

Produce noisy, advisory output

Optimize costs without governance context

Require credentials, billing access, or runtime data

This creates a decision gap at the PR boundary:
engineers are asked to approve irreversible cost risk without reliable, pre-merge evidence.

CostPilot’s Core Claim

CostPilot closes that gap by answering one question with certainty:

“Does this PR introduce an irreversible, material cloud cost commitment?”

If the answer is yes, CostPilot blocks the merge (when configured to do so).
If the answer is no, CostPilot emits deterministic silence.

How CostPilot Works (Trust Chain)

CostPilot operates through a single, inseparable trust chain:

Detect
Identifies cost-impacting changes introduced by a PR diff - including emergent, cross-service effects that are invisible to linting or static checks.

Predict
Calculates the expected recurring cost delta using deterministic heuristics and cold-start assumptions. No billing data, no credentials, no runtime signals.

Explain
Produces a structured explanation showing:

Why the reviewer likely missed this

How the cost propagates through infrastructure

Why the commitment is irreversible post-merge

No step is valid in isolation.
CostPilot refuses to act if evidence is insufficient.

Blocking Philosophy

CostPilot blocks only in two cases:

Explicit governance configuration (policy-as-code, SLOs)

Hard safety violations, including:

Invalid or non-deterministic inputs

Heuristic corruption or drift

Sandbox or execution integrity failures

Cost magnitude, severity, confidence, trends, or feature tier do not block by default.

Blocking precedence is strictly enforced:

Safety → Governance Block → Warn → Advisory

What CostPilot Is Not

CostPilot explicitly does not:

Optimize live infrastructure

Perform billing reconciliation

Use historical cost data

Require cloud credentials or IAM

Replace human judgment

Speak when no material risk exists

If a request falls outside scope, CostPilot refuses deterministically.

Proof Strategy

All public claims must be provable from a single source:

The CostPilot Demo Repository

The demo is adversarial by design and demonstrates:

A realistic PR that passes human review

Deterministic Detect → Predict → Explain outputs

A noop PR that produces provable silence

Reproducible artifacts, snapshots, and CI enforcement

If the demo and product diverge, the release is invalid.

Intended Audience

CostPilot is built for:

Senior platform engineers

Infrastructure reviewers

DevOps and SRE leads

Technical founders

It assumes familiarity with:

Terraform

CI pipelines

PR-based workflows

Emergent cloud pricing behavior

It is not designed for beginners or cost-only optimization users.

Why CostPilot Exists

CostPilot exists to prevent the wrong first merge.

Not to maximize savings.
Not to produce more alerts.
Not to optimize dashboards.

Its value is restraint, correctness, and authority - exercised only when justified.

Non-Negotiable Invariants

Determinism over intelligence

Silence over noise

Refusal over overreach

Evidence over opinion

Governance over optimization

If any of these are violated, CostPilot is behaving incorrectly.
