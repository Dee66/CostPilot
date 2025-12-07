# AI PROJECT CONTEXT (GLOBAL)
**Copilot: Treat this file as authoritative project context.  
Do NOT restate this file in responses.  
Do NOT summarize the contents of referenced files.**

---

## 1. PROJECT SPECIFICATION (IMMUTABLE)

The full project specification is defined in:
- `docs/product.yml`

Copilot must:
- Load and obey every rule, invariant, budget, contract, behavior, constraint, and requirement defined in `product.yml`.
- Treat the spec as **immutable** unless explicitly instructed otherwise.
- Never restate, reinterpret, or expand the spec unless directly asked.
- Resolve ambiguity in favor of deterministic, WASM-safe, zero-IAM execution.

---

## 2. IMPLEMENTATION CHECKLIST (AUTHORITATIVE)

The full project implementation checklist is defined in:
- `docs/checklist.md`  ← **This is the only checklist source**.

Rules:
- The checklist MUST NOT be recreated, rewritten, rearranged, or replaced by Copilot.
- Copilot may only:
  - Mark items `[x]` → “[ ]` (complete / incomplete)
  - Add parenthetical notes (“(done via file X)”)
  - Add sub-items **ONLY** when explicitly told

Copilot must NOT:
- Expand items into explanations
- Change descriptions
- Generate summaries
- Rewrite phases or structure
- Combine or split items unless explicitly instructed

---

## 3. WORKFLOW LOOP (VERY IMPORTANT)

When the user instructs:

### **“Next item”**

Copilot MUST follow this deterministic loop:

1. Load the authoritative checklist from `docs/checklist.md`.
2. Identify the first unchecked `[ ]` item.
3. Implement **ONLY that item**.
4. Modify/create code files as required.
5. Update `docs/checklist.md` by marking the item as `[x]`.
6. Stop.  
   Do NOT automatically proceed to the next item.
7. Keep the response concise and ONLY describe:
   - Which files were changed  
   - What was added/updated  
   - Why the change satisfies the checklist item  

This loop pattern must NEVER be violated.

---

## 4. FILE HANDLING RULES

Copilot may:
- Create new source files
- Modify any file outside this context file
- Update `docs/checklist.md` ONLY to mark items complete
- Add small notes aligned with the checklist format

Copilot may NOT:
- Modify this file unless explicitly asked
- Rewrite or expand `docs/product.yml`
- Regenerate or restructure `docs/checklist.md`
- Move items into different phases

---

## 5. RESPONSE RULES

Copilot must:
- Never restate the checklist
- Never restate the spec
- Keep output minimal
- Avoid verbose explanations
- Give diffs or file summaries only when helpful
- Respect all constraints defined in `docs/product.yml` (determinism, WASM safety, cost bounds, invariants)

Copilot must NOT:
- Explain architectural principles unless asked
- Produce large summaries
- Attempt to “interpret” the roadmap
- Expand or reorganize checklist items
- Introduce new features not in the checklist

---

## 6. PROJECT MEMORY

Persistent context that must be maintained:
- The specification at `docs/product.yml`
- The checklist at `docs/checklist.md`
- All completed items
- Code created or modified as part of earlier steps

Memory resets ONLY when the user says:

> **“Reset project context.”**

---

## 7. SAFETY & CONSISTENCY RULES

Always enforce:
- WASM determinism
- Zero-IAM execution
- No network access
- Performance budgets
- Contract constraints  
- Canonical output formats

If a checklist item contradicts a constraint in the spec:
- Ask for clarification BEFORE executing

## 8. OUTPUT RULES (HIGH PRIORITY)

Copilot must:
- Keep all responses extremely concise.
- Output ONLY what is necessary for the current checklist item.
- Do NOT explain reasoning.
- Do NOT describe architecture unless explicitly asked.
- Do NOT restate the spec.
- Do NOT restate the checklist.
- Do NOT generate summaries.
- Do NOT provide multi-paragraph descriptions.
- At most, output:
  - the filenames changed
  - a very short description of the change
- Never output diffs unless asked.
- Never output large code blocks unless asked.

If the item is implemented correctly, reply:
“**Item complete.**”