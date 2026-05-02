# Fricon v0.2 Reset

## Status

Canonical planning entry point for the proposed v0.2 reset.

This directory describes the intended v0.2 product and architecture direction.
It is not current behavior. Do not document v0.2 behavior in public user docs
or implement durable storage/API contracts from these notes until the relevant
ADR, schema, IPC, Python SDK, Fricon Desktop, migration, and release decisions
have landed.

## v0.2 Thesis

Fricon v0.2 should stop being framed primarily as a workspace dataset browser.
It should become a local lab data library and automation foundation for
scientific measurement work.

The main product goal is to prevent the common failure mode where experimental
history, sample context, copied code folders, JSON parameters, calibration
scripts, and measured data quietly diverge until they are hard to trust or
maintain.

The v0.2 product model is:

```text
one Fricon data library
  -> samples and sample sessions
  -> measurement, analysis, simulation, import, and calibration activity records
  -> artifacts: datasets first, then results, reports, logs, attachments,
     device snapshots, and parameter proposals
  -> parameter snapshots, code summaries, favorites, optional notes/tags, and
     lifecycle flags
  -> measurement-centered portable exports for offline analysis
```

The first v0.2 shipped slice should be narrower than the full target model:
record new measurements and table-shaped datasets, preserve context and safety
paths, require explicit scan schema for plotted datasets, and leave analysis,
automatic calibration, managed device communication, and declarative managed
measurement as later layers.

For normal users, the first mental model should stay simple:

```text
I selected an active sample/session when it mattered.
I ran a measurement.
It produced datasets.
Fricon helps me inspect, annotate, recover, reopen, and export them.
```

## v0.x Milestone Language

Use v0.x milestone names in v0.2 planning docs. Avoid separate pre-1.0 release
aliases that make the roadmap look like it has a second versioning scheme.

- v0.2: first usable LabRAD Grapher/Data Vault replacement slice for new
  measurements. It should record measurement metadata and produced datasets,
  keep datasets directly inspectable and reopenable, support optional
  sample/session context, preserve partial/interrupted data, support basic
  backup/restore and trash/recover, require scan schema for datasets intended
  for plotting, and keep the user model minimal. It is local-only: Fricon
  Desktop and the Python SDK operate through one local service and one primary
  local data library; the bundled CLI is mainly for setup, diagnostics, service
  control, and developer workflows.
- v0.3: candidate product-assistance slice after the v0.2 measurement loop is
  usable. The leading candidate is read-only LAN viewing from another computer.
  Other likely areas include richer sample fields and 2D sample maps,
  comparison views, saved views, portable export viewer polish, attach/correct
  context UX, and better passive code/environment summaries.
- v0.4: candidate automation-foundation slice after measurement history is
  trustworthy. Likely areas include parameter snapshots and proposals, analysis
  provenance UI, calibration workflow history, managed measurement plans, and
  early device-adapter boundaries.

The exact v0.3/v0.4 contents should be decided after v0.2 proves the core
measurement flow. Treat them as sequencing hints, not current implementation
commitments.

## Reading Order

Start here for v0.2 planning:

1. `design.md` - canonical v0.2 product and architecture synthesis
2. `product-direction.md` - supporting product stories and user pain
3. `technical-direction.md` - supporting engineering posture and API/runtime
   direction
4. `measurement-system-foundation-redesign.md` - background redesign proposal
   for dataset artifacts and run-like provenance
5. `experiment-run-and-runner-design.md` - older detailed runner proposal;
   reconcile its `ExperimentRun` terminology with the v0.2 `Measurement`
   naming policy before implementation
6. `parameter-management-design.md` - detailed future parameter proposal
7. `future-concepts.md` - lower-confidence or less narrowed ideas

Use current implementation notes only to understand what must be replaced or
adapted:

- `../current-storage-notes.md`
- `../dataset-semantic-architecture-proposal.md`
- `../database-schema-changes.md`
- `../maintenance-checklist.md`

## Relationship To Older Docs

The older project route was dataset-first, Python-led, and local-first. v0.2
keeps the Python-led and local-first parts, but broadens the foundation from a
dataset catalog to a measurement record system:

- data library instead of user-facing workspace
- optional active sample and session identity before measurement history gets
  fragmented
- measurements as the default data-taking work record
- artifacts as outputs, with datasets as the primary table-shaped case
- parameter and code history as first-class provenance
- analysis, simulation, import, and calibration as activity records that
  consume and produce artifacts

Older dataset semantics work remains useful, especially append-only facts,
record IDs, manifests, and resolved interpretation. But v0.2 implementation
should reconcile that work with the broader data-library, sample/session, and
provenance model before committing storage or API contracts.

## Development Posture

Fricon is still pre-adoption. v0.2 may make breaking changes when they produce a
cleaner long-term foundation.

Prefer rewriting or replacing current domain boundaries when the existing shape
would preserve the wrong user model. Preserve reusable infrastructure where it
does not lock the product into workspace/dataset-first assumptions.

Settled v0.2 decision: prefer a clean model over compatibility with old
workspace/dataset-first APIs and storage assumptions. Keep useful
infrastructure, but do not preserve compatibility just to avoid breaking
pre-adoption local test workspaces.

Once v0.2 records real lab data, preserve data durability and explicit
migration/recovery paths even if v0.x SDK, CLI, UI, or service API shapes keep
changing. The compatibility promise is fail-before-write diagnostics and
recoverable data, not strict pre-1.0 API stability.

The intended breaking scope is broad at the product boundary, but not a mandate
to rewrite every line of code. v0.2 may intentionally break:

- public workspace and dataset-first Python APIs
- workspace/storage layout and compatibility assumptions
- desktop navigation and IPC DTOs that assume datasets are the only organizing
  object
- archive, import, and export formats that cannot carry data-library,
  measurement, sample/session, and provenance context
- distribution and client/server compatibility assumptions that make the
  Fricon Desktop, CLI, Python SDK, or local service drift without an explicit
  compatibility envelope
- protocol assumptions that keep Python SDK on a separate gRPC public contract
  while Fricon Desktop moves to HTTP/WebSocket service APIs
- installation/update assumptions that require a separate server install,
  update an active service, or silently migrate a data library on launch
- GUI/runtime assumptions that make Fricon Desktop own the data backend or make
  remote clients depend on shared-folder access to a database-backed data
  library
- metadata ownership rules that put measurement, sample, parameter, or code
  meaning inside dataset-local metadata

Compatibility for existing local test workspaces should be an explicit
storage/migration ADR decision. Until that ADR exists, do not optimize the v0.2
model around preserving pre-v0.2 workspace behavior.

Good candidates to keep or adapt:

- Rust, Python binding, and frontend build infrastructure
- Arrow payload IO concepts
- append-only dataset facts and record IDs
- chart rendering work after DTO/model changes
- Tauri packaging and web frontend stack
- testing, release, and documentation infrastructure

Good candidates to redesign:

- public workspace mental model
- dataset catalog schema and metadata ownership
- Python dataset creation API
- desktop dataset-first navigation
- IPC/protobuf contracts and public gRPC assumptions
- HTTP/WebSocket service API, event stream, and binary dataset payload contract
- service sidecar packaging, startup, and update lifecycle
- archive/import/export format
- run, sample, parameter, code, and provenance boundaries

The reset should not make the user model bigger than necessary. Keep most
internal provenance objects invisible unless the user is debugging automation,
recovering an interrupted run, or configuring managed execution.

## v0.2 Non-Goals

Do not turn v0.2 into a complex lab administration system.

Non-goals:

- hosted SaaS
- full multi-user roles and permissions
- account/team administration
- distributed database semantics
- broad device-driver framework
- generic workflow DAG engine in the initial v0.2 implementation
- automatic notebook state capture
- mandatory parameter schema design before exploratory measurement
- post-hoc scan guessing as the primary path for plotted measurement data
- LabRAD Data Vault/Grapher compatibility layer for old scripts
- automatic calibration, managed analysis, or managed device communication in
  the first replacement slice
- remote mode or browser/PWA distribution as shipped v0.2 features
- remote annotations or remote acquisition writes
- Labber-like visual sweep builder as a product goal

Remote access, authentication boundaries, and actor/audit records should be
planned early, but the first product should remain local-first and single-owner.
