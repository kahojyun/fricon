# Agent Steering

## Status

Draft.

## Routing Rules

| Task | Start With |
| --- | --- |
| Product direction | `product/vision.md`, `product/capability-map.md`, `product/story-map.md` |
| Domain noun or ownership question | `product/glossary.md`, `product/capability-map.md`, then `domain/README.md` for rebuild rules |
| Architecture question | `architecture/README.md`, then accepted product docs and `domain/README.md` |
| Storage/API/protocol decision | First confirm product inputs and the domain placeholder status; use `architecture/README.md` and ADRs, and do not create detailed architecture files until architecture design starts |
| Compatibility question | `architecture/compatibility-policy.md`, `decisions/ADR-001-v02-clean-reset-boundary.md` |
| Implementation planning | First confirm accepted product, rebuilt domain, architecture, and ADR inputs, then recreate or read derived `specs/` and `implementation-plans/`; they are sentinel-only right now |

## When To Pause And Update Docs

Pause local implementation and update docs when:

- a new domain concept appears and is not already product-owned
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
- Which product terms, capability IDs, and future concepts does it touch?
- Is a rebuilt domain concept needed before implementation?
- Which future module or boundary would own each concept?
- What data-library, local runtime/API, Python SDK, Desktop, CLI, and export
  effects exist?
- What compatibility checks or migrations are needed?
- What validation proves the behavior?
