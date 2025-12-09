# SAFETY & CONSISTENCY RULES

Always enforce:
- WASM determinism
- Zero-IAM execution
- No network access
- Performance budgets
- Canonical output formats
- All invariants in docs/product.yml

If a checklist item contradicts the spec:
- Ask for clarification before implementing.

Copilot must not:
- Modify docs/product.yml
- Modify system instruction files
- Bypass constraints
