# Agent Steering

## Status

Draft.

## Routing Rules

| Task | Start With |
| --- | --- |
| Product direction | `product/vision.md`, `product/capability-map.md`, `product/story-map.md` |
| Domain noun or ownership question | `domain/conceptual-model.md`, `domain/context-map.md` |
| Architecture question | `architecture/README.md`, then accepted product/domain docs |
| Storage/API/protocol decision | First confirm product/domain inputs; use `architecture/README.md` and ADRs, and do not create detailed architecture files until architecture design starts |
| Compatibility question | `architecture/compatibility-policy.md`, `decisions/ADR-001-v02-clean-reset-boundary.md` |
| Implementation planning | First confirm accepted product/domain/architecture/ADR inputs, then recreate or read derived `specs/` and `implementation-plans/`; they are sentinel-only right now |
| Current v0.1 behavior | `dev-docs/current-*.md`, public `docs/`, and code |

## When To Pause And Update Docs

Pause local implementation and update docs when:

- a new domain concept appears
- a module starts owning a concept outside its boundary
- storage/API compatibility behavior changes
- a lifecycle or recovery state is added
- an AI/calibration/automation feature would mutate data-library state
- a calibration or analysis workflow would update durable named parameter refs,
  setup refs, generated config, or devices without a reviewed proposal/audit
  path
- a calibration chain introduces working refs, health gates, retry semantics,
  or pause/review behavior
- a managed routine introduces desired-state setup/device reconciliation,
  parallel device apply, readback semantics, or partial-failure handling
- a spec or milestone draft introduces product scope, domain vocabulary, or
  architecture decisions that are not already owned by upstream docs or ADRs
- a spec contradicts an accepted ADR

## Review Questions Before Implementation

- Which capability IDs and story IDs does this work support?
- Which domain concepts does it touch?
- Which module owns each concept?
- What data-library, local runtime/API, Python SDK, Desktop, CLI, and export
  effects exist?
- What compatibility checks or migrations are needed?
- What validation proves the behavior?
