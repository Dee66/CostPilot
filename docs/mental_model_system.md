# Mental Model System

This system maintains authoritative, accurate understanding of the CostPilot repository to prevent AI drift and ensure consistent development.

## Architecture: Four Strict Layers

### Layer 1 - Fact Extraction (Mechanical, Dumb, Exhaustive)
**What**: Pure extraction of provable facts from the codebase
**How**: No judgment, no exceptions, no interpretation
**Examples**:
- "This crate appears in Cargo.toml"
- "This symbol appears in this file"
- "This binary references this module"

### Layer 2 - Claim Declaration (Human / Curated)
**What**: Explicit claims in `docs/mental_model.md`
**How**: Factual statements only, following normative grammar in `docs/mental_model_claim_grammar.md`
**Examples**:
- "Runtime network access: Not permitted"
- "Non-deterministic behavior is a defect"

### Layer 3 - Claim ↔ Fact Comparison (Strict, Mechanical)
**What**: Detect contradictions between claims and facts
**How**: Pure comparison, no business logic, no allowances
**Tool**: `scripts/detect_mental_model_contradictions.py`

### Layer 4 - Human Resolution / Delta Proposal
**What**: Decide how to reconcile contradictions
**How**: Either fix claims, fix code, or narrow claims
**Tools**: Delta proposals, version history

## Tools

### Contradiction Detector (`scripts/detect_mental_model_contradictions.py`)
- **Purpose**: Layer 3 - Pure contradiction detection
- **Input**: Mental model claims + codebase facts
- **Output**: Structured JSON findings (contradictions, unverified claims, facts without claims)
- **Behavior**: Brutally stupid - no interpretation, no exceptions, no warnings

### Conflict Detector (`scripts/detect_mental_model_conflicts.py`)
- **Purpose**: Advisory conflict reporting
- **Input**: Mental model claims + codebase patterns
- **Output**: Neutral state information for human review
- **Behavior**: Non-blocking, no severity levels or automation

### Discovery (`scripts/discover_mental_model.py`)
- **Purpose**: Raw fact emission for mental model creation
- **Input**: Codebase scan
- **Output**: Normalized facts for manual review
- **Usage**: Before initial mental model creation; optionally during major repo inflection points (large refactors, ownership changes, audits)
- **Never**: Part of CI, pre-commit hooks, or steady-state enforcement

### Version History (`docs/mental_model_versions.md`)
- **Purpose**: Layer 4 - Track all changes with rationale
- **Input**: Applied deltas
- **Output**: Complete audit trail

## Critical Design Principles

### Authority Boundaries
- **Contradiction Detector**: Dumber than Copilot - never interprets intent
- **Copilot**: Follows strict rules, never infers unstated behavior
- **Humans**: Make all interpretive decisions in deltas

### No Silent Drift
- All contradictions are surfaced immediately
- No "allowances" or "exceptions" in code
- Every discrepancy requires explicit resolution

### Mechanical Operation
- Fact extraction: Exhaustive, pattern-based
- Claim parsing: Regex-based, factual only
- Comparison: Direct matching, no fuzzy logic

### Tooling Neutrality
- **Detectors Never**: Assign severity, suggest remediation, or infer intent
- **Policy External**: Block vs allow decisions made only in CI policy
- **State Only**: Tools report factual state for external interpretation
- **Boundary Lock**: Prevents automation from embedding governance assumptions

### Model Freeze
- **State Header**: `MODEL_STATE = mutable | frozen` in `docs/mental_model.md`
- **Detection**: Contradictions detected regardless of freeze state
- **Reporting**: Facts and conflicts reported regardless of freeze state
- **Application**: Deltas applied only when `mutable`; blocked when `frozen`
- **Purpose**: Prevent changes during critical periods (releases, audits)

## Workflow

### For AI Assistants (Copilot, etc.)

1. **Pre-Read**: Always read `docs/mental_model.md` first
2. **Sync Check**: Run contradiction detector
3. **Report Findings**: Surface contradictions exactly as found
4. **Propose Deltas**: Only for Layer 4 resolution

### For Human Developers

1. **Run Checks**: Execute contradiction detector regularly
2. **Review Findings**: Examine contradictions, unverified claims, facts without claims
3. **Resolve**: Either update mental model or fix code
4. **Document**: Record changes in version history

## Output Types

The contradiction detector emits structured JSON:

```json
{
  "contradictions": [
    {
      "claim": "Runtime network access: Not permitted",
      "evidence": {
        "file": "src/foo/bar.rs",
        "symbol": "reqwest"
      }
    }
  ],
  "unverified_claims": [
    "Execution scope: Pull-request boundary only"
  ],
  "facts_without_claims": [
    "Binary target exists: costpilot"
  ]
}
```

## Quality Assurance

### Automated Checks
- Pre-commit hooks run contradiction detector
- CI/CD blocks on contradictions
- Tests verify mental model claims

### Manual Reviews
- Human review required for contradiction resolution
- Delta proposals need approval
- Version history maintains accountability

## Success Metrics

- **Zero Contradictions**: No claim vs fact conflicts
- **Complete Coverage**: All important facts claimed
- **Fast Resolution**: Contradictions resolved within 24 hours
- **Clean Automation**: All checks pass without exceptions

## Emergency Procedures

If contradictions prevent development:

1. **Isolate**: Identify specific claim-evidence pairs
2. **Evidence**: Verify facts are correct
3. **Choose**: Claim wrong, code wrong, or claim needs narrowing
4. **Resolve**: Apply minimal fix
5. **Document**: Record in version history

## Why This Architecture Works

- **Authority**: Clear separation prevents drift
- **Scalability**: Same tools work across repos
- **Reliability**: Mechanical operation prevents bugs
- **Control**: Humans keep all interpretive power
- **Auditability**: Complete history of decisions

## Workflow

### For AI Assistants (Copilot, etc.)

1. **Pre-Read**: Always read `docs/mental_model.md` first
2. **Sync Check**: Run validation tools to ensure sync
3. **Conflict Report**: Report any detected conflicts
4. **Propose Deltas**: Manually craft deltas based on contradiction findings

### For Human Developers

1. **Change Code**: Make code/test changes normally
2. **Run Validation**: Execute contradiction detector
3. **Resolve Conflicts**: Fix any reported issues
4. **Propose Deltas**: Craft explicit deltas for needed updates
5. **Update Model**: Apply approved deltas (check MODEL_STATE)
6. **Version**: Record changes in version history

## Delta Proposal Process

When mental model updates are needed:

1. **Run Detection**: Execute contradiction detector to identify gaps
2. **Review Findings**: Examine contradictions and unverified claims
3. **Craft Deltas**: Manually write explicit delta proposals
4. **Validation**: Verify evidence against code/tests
5. **Approval**: Get human review for high-impact changes
6. **Apply**: Update `docs/mental_model.md` (requires ACTIVE MODEL_STATE)
7. **Version**: Record in `docs/mental_model_versions.md`

## Initial Mental Model Creation

For new projects or major repo inflection points (large refactors, ownership changes, audits):

1. **Discovery**: Run `python3 scripts/discover_mental_model.py` episodically
2. **Review Facts**: Examine `discovered_facts.md` for raw facts
3. **Craft Claims**: Manually create claims following `docs/mental_model_claim_grammar.md`
4. **Validate**: Run contradiction detector to verify claims
5. **Iterate**: Refine claims until no contradictions

## Quality Assurance

### Automated Checks
- Pre-commit hooks run contradiction detector (blocking)
- CI/CD runs contradiction detector (blocking)
- Conflict detector provides advisory reports (non-blocking)
- Tests verify mental model claims

### Manual Reviews
- Human review required for contradiction findings
- Delta proposals need approval and MODEL_STATE check
- Version history maintains accountability

## Benefits

- **Zero Drift**: Authoritative understanding prevents assumptions
- **Consistency**: All AI assistants work from same facts
- **Accuracy**: Claims validated against executable code
- **Evolution**: Structured process for mental model updates
- **Auditability**: Complete history of understanding changes

## Usage Examples

```bash
# Detect contradictions (CI-blocking)
python3 scripts/detect_mental_model_contradictions.py

# Report conflicts (advisory, non-blocking)
python3 scripts/detect_mental_model_conflicts.py

# Discover raw facts (episodic, pre-creation or major inflection points)
python3 scripts/discover_mental_model.py

# Check version history
cat docs/mental_model_versions.md
```

## Emergency Procedures

If mental model conflicts prevent development:

1. **Isolate**: Determine which claims are incorrect
2. **Evidence**: Gather code/test evidence for corrections
3. **Update**: Apply minimal fixes to mental model
4. **Validate**: Re-run checks to confirm resolution
5. **Document**: Record changes in version history

## Success Metrics

- **Zero Critical Conflicts**: No CRITICAL severity conflicts
- **Low Conflict Rate**: < 5 total conflicts at any time
- **High Coverage**: > 90% of important facts documented
- **Fast Resolution**: Conflicts resolved within 24 hours
- **Clean Validation**: All automated checks pass
