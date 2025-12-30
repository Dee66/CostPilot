# Mental Model Claim Grammar

This document defines the normative grammar for claims in the mental model. Claims must follow these structures to be valid.

## Claim Types

### Factual Claims
Statements of verifiable truth about the codebase or system behavior.

**Structure:**
- Declarative sentences
- No aspirational language
- Verifiable against code, tests, or documentation

**Examples:**
- "Name: CostPilot"
- "Binary targets: costpilot, license-issuer"
- "Runtime network access: Not permitted"

### Constraint Claims
Hard limits or prohibitions on system behavior.

**Structure:**
- Clear boundaries or restrictions
- Absolute terms (permitted/not permitted, required/forbidden)
- No exceptions or qualifications

**Examples:**
- "Non-deterministic behavior is a defect"
- "External network calls: Forbidden"
- "File system writes: Restricted to temp directory"

### Scope Claims
Boundaries of operation or applicability.

**Structure:**
- Defines what the system does/doesn't do
- Clear inclusion/exclusion criteria
- No ambiguity in scope

**Examples:**
- "Execution scope: Pull-request boundary only"
- "Supported platforms: Linux, macOS, Windows"
- "Data persistence: Ephemeral only"

## Grammar Rules

### Allowed Syntax
- Claims use colon-separated key-value format: `Key: Value`
- Keys are noun phrases describing the claim domain
- Values are specific, unambiguous statements
- No conditional language ("may", "might", "could")
- No speculative language ("likely", "probably", "appears")

### Prohibited Patterns
- Aspirational statements ("Should support...", "Will include...")
- Vague quantifiers ("Some", "Many", "Most")
- Temporal qualifiers ("Eventually", "Later", "Future")
- Comparative language ("Better than", "Faster than")

### Validation
Claims are validated by:
- Mechanical verification against codebase patterns
- Human review for logical consistency
- Testing against actual system behavior

This grammar ensures claims remain factual, verifiable, and enforceable.
