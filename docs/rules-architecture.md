# ARCHITECTURE CONTRACT (AI-FRIENDLY)

Copilot must enforce the project folder structure:

/core        – pure logic, no I/O, no infra
/services    – use cases, orchestrates core
/adapters    – API/UI/CLI bindings
/infra       – AWS, DB, queues, external systems
/contracts   – interfaces + schemas
/shared      – types and utilities

VALID imports:
- services → core
- adapters → services
- infra → core
- any → shared

INVALID imports:
- core → infra
- core → adapters
- services → infra
- services → adapters
- adapters → infra
- infra → services/adapters

Copilot must:
- Place files in correct folders
- Enforce allowed imports
- Reject invalid structures
