# Product Roadmap

## Status

AI-oriented product planning note. This file is intentionally lightweight and
should not replace issues, pull requests, implementation plans, or release
notes.

## Purpose

Use this roadmap to orient low-context human or AI contributors before choosing
implementation work. It summarizes project direction at a coarse level and
links to the documents that contain detailed policy, plans, or current
implementation facts.

Keep this file short. If an item needs step-by-step execution details, put those
details in a focused implementation plan and link it from here.

## Product Route

The current route is dataset-first, Python-led, and local-first.

Fricon should first become reliable for recording, organizing, and inspecting
scientific measurement datasets. Experiment, parameter, and device concepts
should build on that foundation instead of forcing the dataset layer to absorb
higher-level workflow meaning implicitly.

Initial experiment support should lean on Python scripts as the execution
entry point. The desktop UI should inspect, browse, and eventually assist those
workflows, but it should not become the primary experiment execution engine
before the Python-led model is clear.

Device management remains a later foundation. Near-term work may preserve space
for device identity and configuration, but should not build a broad driver or
hardware orchestration framework.

## Product Pillars

- Workspace and dataset management
- Dataset semantics and durable interpretation
- Desktop dataset browsing, inspection, and charting
- Python scripting API for data recording and automation
- Python-led experiment and parameter workflows
- Reproducibility support for experiment code and environments
- Later device identity and configuration foundations

## Now

Current work should favor:

- stabilizing the local-first workspace and dataset experience
- landing the minimal durable dataset semantic model
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

Near-term direction should clarify and land the dataset semantic model, then
make the desktop UI consume resolved dataset interpretation instead of
rediscovering chart meaning from row heuristics.

Relevant notes:

- `dataset-semantic-architecture-proposal.md`
- `dataset-semantic-implementation-plan.md`
- `database-schema-changes.md`
- `release-and-versioning.md`

## After Dataset Semantics

After the dataset foundation is durable, product work should move toward:

- richer desktop inspection and charting workflows
- dataset read snippets that help users reopen or reproduce analysis from
  Python
- Python-led experiment run records
- parameter capture and display for recorded runs
- a minimal model for connecting runs to datasets

This phase should avoid turning experiment support into a desktop-first
workflow engine too early. Python scripts should remain the first-class way to
run scientific measurement code.

## Future Product Concepts

These ideas are promising, but not current implementation commitments. Convert
them into focused design notes, issues, or ADRs before implementation if the
details affect storage, API contracts, or user workflows.

Parameter management may grow beyond static run metadata into:

- parameter history views
- plotting parameter values across runs or time
- parameter versioning for experiment configurations
- links between parameter versions, runs, and generated datasets

Experiment code management may support local reproducibility features such as:

- automatic history tracking for experiment code
- Git-backed storage, possibly using a bare repository managed inside the
  workspace
- automatic environment capture or setup using tools such as `uv` or `pixi`
- clear links between a run, the code version, the environment, parameters, and
  produced datasets

Dataset usability may include:

- generated Python read snippets for each dataset
- copyable examples for loading selected datasets by workspace-local ID or UID
- snippets that match the public Python API instead of exposing internal
  storage layout

## Later

Longer-term product direction includes explicit concepts for:

- UI-assisted experiment execution
- richer parameter schemas and sweep definitions
- parameter history and version comparison workflows
- experiment code and environment history management
- device identity and configuration
- device integration points once real workflows justify them

Keep these ideas aligned with `project-intent.md`. Do not let future remote,
multi-user, or SaaS possibilities drive the current local-first architecture.

## Issue Planning Guidance

When generating GitHub issues from this roadmap:

- split work by product pillar and owning code area
- keep each issue small enough for one focused pull request
- mark whether the issue is backend, Python API, desktop UI, docs, or
  cross-boundary
- list likely touched files or modules when known
- list required validation commands from `pr-preflight-checklist.md`
- create an ADR task before implementation when a durable cross-cutting
  decision is still unresolved

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
- experiment code history and environment management foundations

Use `dev-docs/adr/README.md` for ADR rules and format.
