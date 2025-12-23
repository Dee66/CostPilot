# WORKFLOW RULES (v2.3)

This file defines the execution workflow.
Copilot must load and obey all rules in this file.
Do not summarize or restate this file.

----------------------------------------
EXECUTION TRIGGERS
----------------------------------------
The workflow begins when the user issues:
- "Next item"
- "Continue"
- "Proceed"
- "Do the next one"
- "Keep going"

----------------------------------------
PRIMARY WORKFLOW LOOP
----------------------------------------
On trigger:
1. Load docs/checklist.md.
2. Identify the next incomplete item.
3. Implement ONLY that item.
4. Modify or create code files as required.
5. Update docs/checklist.md by marking the item complete.

----------------------------------------
CONSTRAINTS
----------------------------------------
- Do not skip items.
- Do not reorder checklist items.
- Do not generate explanations, summaries, or commentary.
- Do not modify the spec.
- Do not modify rule files.
- Do not exit early unless a stop condition applies.

----------------------------------------
STOP CONDITIONS
----------------------------------------
Stop only if:
- User says: "Stop", "Pause", or "Hold".
- No incomplete checklist items remain.
- A conflict exists between spec, checklist, or rule files.
- A required detail is missing and progress cannot be correct.

If conflict: output "Blocker: conflict detected."
If missing detail: output "Blocker: need clarification."

----------------------------------------
VALIDATION STEP
----------------------------------------
Before implementing:
- Check for conflicts between the checklist item, the spec, and rule files.
- If conflict → stop and report.
- If not enough information to implement safely → stop and ask.

----------------------------------------
ALLOWED OUTPUT
----------------------------------------
Allowed responses:
1. "OK."
2. "Item complete."
3. A short list of changed filenames.
4. A single clarifying question (only if progress is impossible without it).

No other output is permitted.

----------------------------------------
FILE HANDLING RULES
----------------------------------------
- You may create or modify any file except copilot-instructions.md.
- Modify docs/checklist.md ONLY to mark items complete.
- Do not regenerate, reorder, or rewrite the checklist.
- Do not modify the spec under any circumstance.

----------------------------------------
WORKER MODE BEHAVIOR
----------------------------------------
- Deterministic execution.
- No suggestions.
- No alternative implementations.
- No reasoning unless explicitly requested.
- No verbose code unless the user asks for it.

----------------------------------------
RECOVERY RULES
----------------------------------------
If previous state becomes ambiguous:
- Reload spec.
- Reload checklist.
- Reload all rule files.
- Resume from the next incomplete item.

----------------------------------------
END OF WORKFLOW RULES
----------------------------------------
