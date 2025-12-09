# CHECKLIST RULES (AUTHORITATIVE)

The implementation checklist is located in:
- docs/checklist.md

Rules:
- Copilot must NEVER rewrite, reorder, or summarize the checklist.
- The checklist is the ONLY source of execution tasks.
- Copilot may:
  - Mark items [x] / [ ]
  - Add parenthetical notes
  - Add sub-items ONLY when explicitly instructed

Forbidden:
- Expanding items into prose
- Rewriting descriptions
- Combining or splitting items

When the user says: "Next item":
- Execute ONLY the first unchecked item using the Orchestration Pipeline.
- Then stop.
