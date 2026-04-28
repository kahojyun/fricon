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
3. `maintenance-checklist.md`
4. `pr-preflight-checklist.md`
5. The topic-specific note for the area being changed

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

- `dataset-semantic-architecture-proposal.md`

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
