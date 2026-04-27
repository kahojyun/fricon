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
2. `maintenance-checklist.md`
3. `pr-preflight-checklist.md`
4. The topic-specific note for the area being changed

## Canonical Guidance

These files are active project policy. Prefer them over duplicated guidance in
skills, prompts, or contributor notes.

- `project-intent.md` - product direction, target users, non-goals, and AI
  guardrails
- `maintenance-checklist.md` - coordinated update checklist for cross-boundary
  changes
- `pr-preflight-checklist.md` - local and pre-PR validation matrix
- `release-and-versioning.md` - release model, changeset policy, and version
  bump guidance
- `architecture-guidelines.md` - current implementation architecture rules and
  boundary guidance
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

## Proposals And Plans

These files describe proposed future direction. Do not treat them as current
implementation facts unless the implementation has already landed.

- `dataset-semantic-architecture-proposal.md`
- `dataset-semantic-implementation-plan.md`

## Maintenance Rules

- Keep checklist-style canonical guidance in `dev-docs/` and link to it from
  skills, `AGENTS.md`, and `CONTRIBUTING.md`.
- Keep `docs/` focused on user workflows and stable user-facing concepts.
- Keep internal storage, protocol, migration, architecture, and AI-agent
  guidance in `dev-docs/`.
- When a proposal becomes current behavior, update or split the proposal so
  current facts live in a current implementation note.
