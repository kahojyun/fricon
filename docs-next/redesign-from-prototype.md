# Redesign From Prototype Workflow

## Status

Draft workflow tracker.

## Purpose

Track the reset from the v0.1 prototype to the v0.2+ design baseline.

## Workflow State

| Phase | Status | Output |
| --- | --- | --- |
| Phase 0: Inventory and postmortem | Draft complete | `postmortems/v0-lessons.md`, `architecture/compatibility-policy.md`, ADR-001 |
| Phase 1: Product and capability baseline | Draft complete | `product/vision.md`, `product/personas.md`, `product/capability-map.md`, `product/story-map.md`, `product/glossary.md` |
| Phase 2: Domain baseline | Draft complete | `domain/conceptual-model.md`, `domain/context-map.md`, `domain/lifecycle-model.md`, `domain/invariants.md` |
| Phase 3: Architecture baseline | Draft complete | `architecture/system-overview.md`, `architecture/module-boundaries.md`, `architecture/data-flow.md`, `architecture/api-boundaries.md`, `architecture/storage-model.md`, `architecture/compatibility-policy.md`, `architecture/risks.md` |
| Phase 4: Traceability baseline | Draft complete | `product/story-module-matrix.md`, `specs/SPEC-001-data-library-measurement-foundation/traceability.md` |
| Phase 5: System-slice specs | Draft started | `specs/SPEC-001-data-library-measurement-foundation/` |
| Phase 6: Implementation and doc sync | Not started | Requires accepted SPEC-001 and storage/API ADRs |

## Baseline Readiness Gate

Gate A currently passes as a draft baseline:

- Vision and capability map exist.
- Core domain concepts are named.
- Module boundaries are explicit.
- Compatibility policy exists.
- ADRs record the reset boundary and documentation governance.
- Agent steering exists.

Before implementation, move the relevant docs or spec from Draft to Accepted,
or record explicit user authorization to implement from Draft.

## Immediate Next Decisions

Create focused ADRs before durable v0.2 implementation:

1. Data-library storage layout and pre-v0.2 migration/import stance.
2. Local service API contract and compatibility negotiation.
3. Dataset artifact storage and scan schema representation.
4. Measurement lifecycle and event/audit record model.
5. Export bundle format and privacy preview.
