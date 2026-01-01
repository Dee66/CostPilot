# Copilot Instructions

This repository is governed by an explicit AI behavioral contract and an authoritative mental model.

## Scope of Responsibility

This file defines integration and routing only.

- Behavioral rules, refusal conditions, uncertainty handling, output discipline, and execution constraints are defined in `/AGENTS.md`.
- Claim structure and verification classes are defined in `docs/mental_model_claim_grammar.md`.

If guidance is missing, treat it as UNKNOWN and stop.

## Mandatory Preconditions

- Read `/AGENTS.md` and comply with it.
- Read `docs/mental_model.md` as authoritative compiled state.
- Treat tests and executable code as higher authority than documentation.

If any required file is missing or unreadable, stop.

## Sync and Validation

When performing decision-affecting work (including code changes, reviews, or validation against claims):

Run: python3 scripts/detect_mental_model_contradictions.py


If contradictions are detected:
- Report them.
- Stop immediately.

Do not continue analysis or implementation while contradictions exist.

## Mental Model Changes

- Do not rewrite `docs/mental_model.md`.
- Any change to understanding must be proposed as an explicit delta.
- Respect `MODEL_STATE`.
- If `MODEL_STATE` is `frozen`, do not propose deltas.

No implicit clarification, completion, or improvement of the mental model is permitted.

End of instructions.
