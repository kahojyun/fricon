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
3. `../docs-next/README.md` when working on new v0.2+ reset design
4. `maintenance-checklist.md`
5. `pr-preflight-checklist.md`
6. The topic-specific note for the area being changed

For narrow tasks, prefer the task-specific starting points below over reading
every canonical document.

## Agent Starting Points By Change Type

<!-- prettier-ignore -->
| Change type                                 | Start here                               | Then read                                                                                                                        | Usually avoid                                                          |
| ------------------------------------------- | ---------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------- |
| SQLite schema visible through Python        | `database-schema-changes.md`             | `maintenance-checklist.md`, `release-and-versioning.md`, Diesel skill                                                            | Dataset semantic proposal files unless semantics are changing          |
| Desktop UI action with Rust Tauri command   | `desktop-ui-feature-playbook.md`         | `architecture-guidelines.md`, `testing-strategy.md`, `release-and-versioning.md`, UI AGENTS files                                | React/shadcn skills unless touching those APIs                         |
| Workspace format or metadata compatibility  | `maintenance-checklist.md`               | `current-storage-notes.md`, `release-and-versioning.md`, `pr-preflight-checklist.md`                                             | Architecture background unless boundary design is unclear              |
| Dataset archive import/export compatibility | `maintenance-checklist.md`               | `current-storage-notes.md`, `release-and-versioning.md`, `pr-preflight-checklist.md`                                             | Public docs unless behavior is user-visible                            |
| Rust IPC/gRPC contract compatibility        | `maintenance-checklist.md`               | `pr-preflight-checklist.md`, `release-and-versioning.md`                                                                         | Public docs unless behavior is user-visible                            |
| Public dataset docs update                  | `docs/dataset.md` and `docs/concepts.md` | `current-storage-notes.md`, `current-dataset-semantics.md`                                                                       | Implementation plan details unless landed behavior is being documented |
| v0.2+ product or architecture reset         | `../docs-next/README.md`                 | `../docs-next/redesign-from-prototype.md`, `../docs-next/product/vision.md`, `../docs-next/domain/conceptual-model.md`, `../docs-next/architecture/system-overview.md`                         | Treating old v0.2 proposal content as current behavior or source of truth |
| Measurement-system foundation redesign      | `../docs-next/specs/SPEC-001-data-library-measurement-foundation/requirements.md` | `../docs-next/product/story-map.md`, `../docs-next/domain/lifecycle-model.md`, `../docs-next/architecture/module-boundaries.md` | Treating archived proposal content as active v0.2 guidance             |
| Parameter management product planning       | `../docs-next/product/future-concepts.md` | `../docs-next/domain/conceptual-model.md`, `../docs-next/product/capability-map.md`, `v0.2/future-concepts.md` as historical input only | Promoting parameter registry work before measurement foundation is accepted |
| Future concept capture                      | `../docs-next/product/future-concepts.md` | `../docs-next/product/capability-map.md`, `../docs-next/domain/conceptual-model.md`                                               | Creating implementation issues before the concept is narrowed          |
| Product route or issue planning             | `roadmap.md`                             | `project-intent.md`, proposal or implementation-plan files for affected areas                                                    | Treating roadmap future concepts as current implementation facts       |

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
- `current-dataset-semantics.md` - current dataset semantic model, writer
  metadata, reader interpretation, and chart projection behavior

## v0.2+ Design Baseline

New v0.2+ product, domain, architecture, ADR, traceability, and system-slice
planning starts in `../docs-next/`.

Use `dev-docs/v0.2/` and `dev-docs/v0.2/archive/` as source inputs and
historical rationale. They should not override accepted `docs-next/` decisions
or specs.

## Architecture Notes

These files are advisory background unless they explicitly point to a canonical
checklist, policy, or guideline.

- `architecture-ai-notes.md` - architecture discussion and AI-assisted
  maintainability guidance
- `adr/` - durable architecture decision records for settled cross-cutting
  decisions

## Proposals

These files describe proposed or historical direction. Do not treat them as
current implementation fact unless a current implementation note confirms that
the behavior has landed.

- `v0.2/` - historical planning directory for the proposed v0.2 reset. Its
  contents are source inputs for `../docs-next/`, not the new v0.2+ design
  baseline. Historical proposal inputs live under `v0.2/archive/` and should
  not override `docs-next/` decisions or specs.

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
