# Developer Docs

## Status

Directory index for developer documentation.

## Purpose

`dev-docs/` contains maintainer-facing documentation for implementation,
architecture, release, testing, and AI-assisted development. These files are
not public user docs. Public user-facing docs live in `docs/`.

## Reading Order

Start here when orienting a new human or AI contributor:

1. `project-intent.md`
2. `roadmap.md`
3. `v0.2/README.md` when working on the proposed v0.2 reset
4. `maintenance-checklist.md`
5. `pr-preflight-checklist.md`
6. The topic-specific note for the area being changed

For narrow tasks, prefer the task-specific starting points below over reading
every canonical document.

## Agent Starting Points By Change Type

| Change type                                 | Start here                               | Then read                                                                                         | Usually avoid                                                          |
| ------------------------------------------- | ---------------------------------------- | ------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------- |
| SQLite schema visible through Python        | `database-schema-changes.md`             | `maintenance-checklist.md`, `release-and-versioning.md`, Diesel skill                             | Dataset semantic proposal files unless semantics are changing          |
| Desktop UI action with Rust Tauri command   | `desktop-ui-feature-playbook.md`         | `architecture-guidelines.md`, `testing-strategy.md`, `release-and-versioning.md`, UI AGENTS files | React/shadcn skills unless touching those APIs                         |
| Workspace format or metadata compatibility  | `maintenance-checklist.md`               | `current-storage-notes.md`, `release-and-versioning.md`, `pr-preflight-checklist.md`              | Architecture background unless boundary design is unclear              |
| Dataset archive import/export compatibility | `maintenance-checklist.md`               | `current-storage-notes.md`, `release-and-versioning.md`, `pr-preflight-checklist.md`              | Public docs unless behavior is user-visible                            |
| Rust IPC/gRPC contract compatibility        | `maintenance-checklist.md`               | `pr-preflight-checklist.md`, `release-and-versioning.md`                                          | Public docs unless behavior is user-visible                            |
| Public dataset docs update                  | `docs/dataset.md` and `docs/concepts.md` | `current-storage-notes.md`, dataset semantic proposal status only                                 | Implementation plan details unless landed behavior is being documented |
| v0.2 product or architecture reset          | `v0.2/README.md`                         | `v0.2/product-direction.md`, `v0.2/technical-direction.md`, `v0.2/measurement-system-foundation-redesign.md`, `adr/README.md` | Treating v0.2 proposal content as current behavior                     |
| Measurement-system foundation redesign      | `v0.2/measurement-system-foundation-redesign.md` | `project-intent.md`, `roadmap.md`, `dataset-semantic-architecture-proposal.md`, `v0.2/experiment-run-and-runner-design.md`, `adr/README.md` | Treating redesign proposal content as current behavior                 |
| Parameter management product planning       | `v0.2/parameter-management-design.md`    | `v0.2/product-direction.md`, `v0.2/technical-direction.md`, `adr/README.md`                        | Treating proposal content as current behavior                          |
| Future concept capture                      | `v0.2/future-concepts.md`                | `v0.2/product-direction.md`, `v0.2/technical-direction.md`                                         | Creating implementation issues before the concept is narrowed          |
| Product route or issue planning             | `roadmap.md`                             | `project-intent.md`, proposal or implementation-plan files for affected areas                     | Treating roadmap future concepts as current implementation facts       |

## Canonical Guidance

These files are active project policy. Prefer them over duplicated guidance in
skills, prompts, or contributor notes.

- `project-intent.md` - product direction, target users, non-goals, and AI
  guardrails
- `roadmap.md` - AI-oriented product route, sequencing, relationship to current
  state, and decision pressure map
- `maintenance-checklist.md` - coordinated update checklist for cross-boundary
  changes
- `pr-preflight-checklist.md` - local and pre-PR validation matrix
- `release-and-versioning.md` - release model, changeset policy, and version
  bump guidance
- `architecture-guidelines.md` - current implementation architecture rules and
  boundary guidance
- `desktop-ui-feature-playbook.md` - low-context path for Rust Tauri plus
  frontend feature work
- `database-schema-changes.md` - low-context path for Diesel changes,
  especially DB-to-Python changes
- `testing-strategy.md` - test runner split and test placement rules
- `internal-doc-comments.md` - Rustdoc/comment style for maintainable internal
  code

## Current Implementation Notes

These files describe the current implementation. Update them when the
implementation changes.

- `current-storage-notes.md` - current workspace and dataset storage details

## Architecture Notes

These files are advisory background unless they explicitly point to a canonical
checklist, policy, or guideline.

- `architecture-ai-notes.md` - architecture discussion and AI-assisted
  maintainability guidance
- `adr/` - durable architecture decision records for settled cross-cutting
  decisions

## Proposals

This file describes proposed future direction. Do not treat it as current
implementation fact unless the implementation has already landed.

- `v0.2/` - canonical planning directory for the proposed v0.2 reset. It now
  owns the non-implemented proposal set, including the local data-library
  product direction, sample/session model, measurement-system foundation
  redesign, experiment/runner direction, parameter direction, future concepts,
  distribution surfaces, remote/auth boundary, and rewrite strategy.
- `dataset-semantic-architecture-proposal.md` - dataset-specific semantic
  proposal that has active implementation work. Reconcile it with the broader
  v0.2 direction before committing additional durable APIs or storage contracts.

## Maintenance Rules

- Keep checklist-style canonical guidance in `dev-docs/` and link to it from
  skills, `AGENTS.md`, and `CONTRIBUTING.md`.
- Keep `docs/` focused on user workflows and stable user-facing concepts.
- Keep internal storage, protocol, migration, architecture, and AI-agent
  guidance in `dev-docs/`.
- When a proposal becomes current behavior, update or split the proposal so
  current facts live in a current implementation note.
- Keep `roadmap.md` short and directional; move detailed execution steps into
  focused implementation plans.
- Add ADRs selectively for durable decisions, not for routine local
  implementation choices.
