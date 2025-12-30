# Copilot Operating Instructions

This repository uses an explicit, external mental model.
Copilot must treat this model as authoritative compiled state.

## Mandatory Pre-Read

Before responding to any request, you MUST:

1. Read `docs/mental_model.md` as authoritative project state
2. Treat tests and executable code as higher authority than documentation
3. Treat absence of information as UNKNOWN, not FALSE
4. Prefer silence over speculation

If you have not read the mental model, you are not synced.

## Authority & Precedence

Order of truth:

1. Tests
2. Executable code
3. `docs/mental_model.md`
4. Other documentation and comments

If any conflict exists between these layers, you MUST report it.
Do not resolve conflicts implicitly.

## Mental Model Change Rules

- New information MUST be proposed as a delta
- Existing mental-model content MUST NOT be rewritten unless factually incorrect
- Deltas MUST be explicit, minimal, and additive
- Deltas MUST NOT be auto-applied or merged

The mental model is not to be “improved”, “completed”, or “clarified”
unless explicitly instructed.

## Discovery Rules

When interacting with the repository:

- If repository facts contradict the mental model, REPORT the contradiction
- If new verifiable facts are discovered, PROPOSE them as a delta
- If evidence is insufficient, say so explicitly and stop

Do NOT infer missing structure, intent, or behavior.

## Prohibited Behaviors

You MUST NOT:

- Infer intent beyond documented facts
- Fill in missing sections for completeness
- Assume architectural patterns not explicitly stated
- Redesign or refactor unless explicitly asked
- Treat documentation as aspirational truth
- Collapse uncertainty into assumptions

## Output Discipline

- Use factual, bounded language
- Avoid “likely”, “probably”, “implied”, or “appears to”
- State confidence explicitly when required
- Silence is a valid and correct outcome

## Sync Failure Condition

If you cannot comply with these rules due to missing information,
state what is missing and stop.

Proceeding unsynced is incorrect behavior.
