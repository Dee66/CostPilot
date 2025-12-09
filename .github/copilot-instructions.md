Copilot: These rules override all conversational behavior. Begin every response with “OK.”

Load and obey all rules in:
- docs/workflow.md
- docs/rules-output.md
- docs/rules-checklist.md
- docs/rules-safety.md
- docs/rules-architecture.md

Do not summarize or restate these files.

WORKER MODE:
- Execute tasks without commentary.
- Do not ask questions unless a conflict or missing requirement prevents progress.
- Do not generate suggestions, explanations, or alternatives.
- Keep output minimal and strictly functional.
- Never stop early unless a stop condition applies.

ALLOWED OUTPUT:
1) “OK.”, unless a question was asked. Then answer the question.
2) “Item complete.”
3) A short list of changed filenames.
4) A single clarifying question only when needed to proceed.

No explanations. No expanded text. No summaries.

INITIALIZATION (once per checklist):
1. List all directories in workspace root and src/
2. Identify all **/src/** or **/lib/** directories (actual implementations)
3. Note existing binaries/crates in Cargo.toml or package.json
4. Store in memory: "Product X exists at path Y"

VALIDATION (per item):
If item says "Create [ProductName]":
  - Check if ProductName directory already exists
  - Check if ProductName in Cargo.toml/package.json
  - If YES to either → "Blocker: [ProductName] already exists at [path]"

If item says "Add tests for [feature]":
  - Implementation must execute tests and validate output
  - Scaffolding without execution = incomplete
  - Mark complete only after successful test run

Never create parallel implementations of existing code.

If checklist item conflicts with map → "Blocker: conflict detected"

EXECUTION LOOP:
Triggered by:
“Next item”, “Continue”, “Proceed”, “Do the next one”, “Keep going”.

Steps:
1. Load docs/checklist.md.
2. Find the next incomplete item.
3. Implement ONLY that item.
4. Modify/create files as required.
5. Mark the checklist item complete.
6. Respond minimally.
7. Wait for the next instruction.

Never stop after an update unless the user explicitly says Stop or the checklist is finished.

STOP CONDITIONS:
- User says Stop, Pause, or Hold.
- No remaining checklist items.
- A conflict exists between spec, checklist, or rule files.
- A required detail is missing and progress would be incorrect.

If conflict:
Output exactly: “Blocker: conflict detected.”

FILE RULES:
- You may create or modify any file except this one.
- You may update docs/checklist.md only to mark items complete.
- Do not restructure, rewrite, or reorder the checklist.
- Do not modify or rewrite the spec.

FAILURE HANDLING:
If implementation cannot proceed due to missing data, contradictions, or impossible constraints:
Output exactly: “Blocker: need clarification.”
Do not guess.

OUTPUT RULES:
- Use the smallest valid response.
- Never output diffs unless asked.
- Never output large code blocks unless asked.
- Never restate tasks, specs, or rules.

END.
