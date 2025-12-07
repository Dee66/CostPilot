# AI PROJECT CONTEXT (GLOBAL)
**Copilot: Treat this file as authoritative project context.  
Do NOT restate this file in responses.  
Do NOT summarize the contents of referenced files.  
Follow all rules in this file with highest priority.**

---

## 1. PROJECT SPECIFICATION (IMMUTABLE)

The full project specification is defined in:
- `/docs/product.yml`

Copilot must:
- Load and obey every rule, invariant, budget, contract, behavior, constraint, and requirement defined in `product.yml`.
- Treat the spec as **immutable** unless explicitly instructed otherwise.
- Never restate, reinterpret, or expand the spec unless directly asked.
- Resolve ambiguity in favor of deterministic, WASM-safe, zero-IAM execution.

---

## 2. IMPLEMENTATION CHECKLIST (AUTHORITATIVE)

The full implementation checklist is defined in:
- `/docs/checklist.md` ← **This is the only checklist source.**

Rules:
- The checklist MUST NOT be recreated, rewritten, rearranged, or replaced.
- Copilot may only:
  - Mark items `[x]` / `[ ]`
  - Add parenthetical notes (“(done in file X)”)
  - Add sub-items only when explicitly instructed

Copilot must NOT:
- Expand items into explanations
- Change descriptions
- Generate summaries
- Rewrite phases or structure
- Combine or split items unless explicitly instructed

---

## 3. WORKFLOW LOOP (CORE BEHAVIOR)

When the user instructs **“Next item”**, Copilot MUST:

1. Load `/docs/checklist.md`.
2. Identify the first unchecked `[ ]` item.
3. Execute the **AI Orchestration Pipeline** (Section 8) to complete ONLY that item.
4. Modify/create code files as required.
5. Update `/docs/checklist.md` by marking the item as `[x]`.
6. Stop. Do NOT continue to the next item automatically.
7. Respond concisely per Output Rules.

This loop pattern must NEVER be violated.

---

## 4. FILE HANDLING RULES

Copilot may:
- Create new source files as needed.
- Modify any file except `ai-context.md`.
- Update `/docs/checklist.md` ONLY to mark items complete.

Copilot may NOT:
- Modify this file unless explicitly asked.
- Rewrite or expand `/docs/product.yml`.
- Restructure, regenerate, or reorder `/docs/checklist.md`.
- Move items into different phases.

---

## 5. RESPONSE RULES (OUTPUT MINIMIZATION)

Copilot must:
- Keep responses extremely concise.
- Output ONLY what is necessary for the current checklist item.
- Never restate the spec or the checklist.
- Never explain reasoning unless explicitly asked.
- Never generate summaries or multi-paragraph descriptions.

Allowed output:
- Filenames changed
- A very short description of the change  
- Or simply: **“Item complete.”**

Copilot must NOT:
- Provide diffs unless asked.
- Output large code blocks unless asked.

---

## 6. PROJECT MEMORY

Persistent context that must be maintained:
- Specification in `/docs/product.yml`
- Checklist in `/docs/checklist.md`
- Completed items
- Previously created or modified code

Memory resets ONLY when explicitly instructed with:
**“Reset project context.”**

---

## 7. SAFETY & CONSISTENCY RULES

Always enforce:
- WASM determinism  
- Zero-IAM execution  
- No network access  
- Performance budgets  
- Contract constraints  
- Canonical output formats  

If a checklist item contradicts any constraint in the spec:
- Ask for clarification BEFORE implementing.

---

## 8. AI ORCHESTRATION PIPELINE (MANDATORY)

For ANY implementation task—including but not limited to checklist items—Copilot MUST execute the following pipeline **in order**:

### **1. PLAN**
- Briefly outline the steps required to fulfill the task.  
- No explanations or restatements of context.

### **2. VALIDATE**
- Compare the plan against:
  - the checklist item  
  - the specification (`/docs/product.yml`)  
  - all rules in this file  
- If any contradiction appears, ask for clarification immediately.

### **3. IMPLEMENT**
- Modify or create code files exactly as required.
- Follow SOLID, architectural structure, and all constraints.
- Keep implementations minimal, correct, and deterministic.

### **4. VERIFY**
- Confirm internally that the implementation satisfies:
  - the checklist item  
  - the spec requirements  
  - all constraints  
- Output only: “validated” or “failed”.

### **5. UPDATE**
- Mark the checklist item `[x]` in `/docs/checklist.md`.
- Add a brief parenthetical note if useful.
- Stop and wait for the next instruction.

This pipeline must be followed **for every task without exception**.