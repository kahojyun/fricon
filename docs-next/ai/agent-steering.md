# Agent Steering

## Status

Draft.

## Routing Rules

| Task | Start With |
| --- | --- |
| Product direction | `product/vision.md`, `product/capability-map.md`, `product/story-map.md` |
| Domain noun or ownership question | `domain/conceptual-model.md`, `domain/context-map.md` |
| Storage/API/protocol decision | `architecture/storage-model.md`, `architecture/api-boundaries.md`, ADRs |
| Compatibility question | `architecture/compatibility-policy.md`, `decisions/ADR-001-v02-clean-reset-boundary.md` |
| Implementation planning | Relevant `specs/SPEC-###-*` and `implementation-plans/` |
| Current v0.1 behavior | `dev-docs/current-*.md`, public `docs/`, and code |

## When To Pause And Update Docs

Pause local implementation and update docs when:

- a new domain concept appears
- a module starts owning a concept outside its boundary
- storage/API compatibility behavior changes
- a lifecycle or recovery state is added
- an AI/calibration/automation feature would mutate data-library state
- a spec contradicts an accepted ADR

## Review Questions Before Implementation

- Which capability IDs and story IDs does this work support?
- Which domain concepts does it touch?
- Which module owns each concept?
- What data-library, service API, Python SDK, Desktop, CLI, and export effects
  exist?
- What compatibility checks or migrations are needed?
- What validation proves the behavior?
