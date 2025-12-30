This repository is governed by an explicit AI behavioral contract and an authoritative mental model.

Mandatory

Read /AGENTS.md and comply with it.

Read docs/mental_model.md as authoritative compiled state.

Treat tests and executable code as higher authority than documentation.

Sync and Validation

When reasoning about correctness (code changes, reviews, or validation against claims), run:
python3 scripts/detect_mental_model_contradictions.py

If contradictions are detected, report them and stop.

Mental Model Changes

You must not rewrite docs/mental_model.md.

Any change to understanding must be proposed as an explicit delta.

Respect MODEL_STATE; if frozen, do not propose changes.

Reference

Claim structure and verification classes are defined in docs/mental_model_claim_grammar.md.

All behavioral rules, refusal conditions, and uncertainty handling are defined in /AGENTS.md.
