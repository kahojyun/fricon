# Lightweight Roadmap

## Status

AI-oriented planning note. This file is intentionally lightweight and should not
replace issues, pull requests, implementation plans, or release notes.

## Purpose

Use this roadmap to orient low-context human or AI contributors before choosing
implementation work. It summarizes project direction at a coarse level and
links to the documents that contain detailed policy, plans, or current
implementation facts.

Keep this file short. If an item needs step-by-step execution details, put those
details in a focused implementation plan and link it from here.

## Now

Current work should favor:

- stabilizing the local-first workspace and dataset experience
- keeping Python API and desktop UI behavior coherent as user-facing contracts
- preserving clear vertical slice boundaries across Rust, Python bindings, and
  the frontend
- keeping current implementation notes accurate when storage, IPC, or workspace
  behavior changes

Start with:

- `project-intent.md`
- `architecture-guidelines.md`
- `current-storage-notes.md`
- `maintenance-checklist.md`
- `pr-preflight-checklist.md`

## Next

Near-term direction should clarify and land the dataset semantic model without
turning proposals into assumed implementation facts before the code catches up.

Relevant notes:

- `dataset-semantic-architecture-proposal.md`
- `dataset-semantic-implementation-plan.md`
- `database-schema-changes.md`
- `release-and-versioning.md`

## Later

Longer-term product direction includes explicit concepts for:

- experiment execution
- parameter management
- device management
- richer desktop inspection workflows
- Python scripting and automation around those concepts

Keep these ideas aligned with `project-intent.md`. Do not let future remote,
multi-user, or SaaS possibilities drive the current local-first architecture.

## Explicit Non-Goals For Current Work

Do not treat these as active roadmap items unless the project direction changes
explicitly:

- multi-user collaboration
- hosted SaaS operation
- account, team, role, or permission systems
- distributed database semantics
- web-first deployment as the primary experience

## Decision Pressure

Create or update an ADR when a decision is durable, cross-cutting, and likely to
be rediscovered or relitigated later. Good ADR candidates include:

- workspace and storage format compatibility
- dataset semantic model commitments
- IPC or gRPC compatibility policy
- Python API contract decisions
- desktop/runtime architecture decisions
- experiment, parameter, or device model foundations

Use `dev-docs/adr/README.md` for ADR rules and format.
