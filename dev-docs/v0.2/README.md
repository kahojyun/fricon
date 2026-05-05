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

This includes the multi-computer lab case. Fricon should help researchers stop
copying measurement code folders between acquisition PCs, but it should do so
through explicit code-source provenance and future setup/update tooling, not by
making one central Fricon server or shared data folder responsible for
everything.

The full product model that v0.2 should leave room for is:

```text
one Fricon data library
  -> samples and sample sessions
  -> measurement, analysis, simulation, import, and calibration activity records
  -> artifacts: datasets first, then results, reports, logs, attachments,
     device snapshots, and parameter proposals
  -> parameter snapshots, code provenance summaries, favorites, optional
     notes/tags, and lifecycle flags
  -> measurement-centered portable exports for offline analysis
```

The first shipped v0.2 slice is deliberately narrower:

```text
one local data library
  -> optional sample/session context
  -> explicit measurements
  -> table-shaped dataset artifacts with scan schema for plotted data
  -> notes/events, lifecycle flags, attachments, and honest code provenance
  -> backup/restore, trash/recover, diagnostics, and portable exports
```

Analysis records, simulation/import activity records, calibration records,
parameter proposals, managed code snapshots, managed script execution, device
communication, and declarative managed measurement are reserved future layers
unless a focused ADR narrows a minimal placeholder.

For normal users, the first mental model should stay simple:

```text
I selected an active sample/session when it mattered.
I ran a measurement.
It produced datasets.
Fricon helps me inspect, annotate, recover, reopen, and export them.
```

Focused product requirements for this loop live in
`measurement-requirements.md`. Use that note to narrow user-visible
measurement behavior before writing durable storage, API, Desktop, export, or
lifecycle ADRs.

## v0.x Milestone Language

Use v0.x milestone names in v0.2 planning docs. Avoid separate pre-1.0 release
aliases that make the roadmap look like it has a second versioning scheme.

- v0.2: first usable LabRAD Grapher/Data Vault replacement slice for new
  measurements. It should record measurement metadata and produced datasets,
  keep datasets directly inspectable and reopenable, support optional
  sample/session context, preserve partial/interrupted data, support basic
  backup/restore and trash/recover, require scan schema for datasets intended
  for plotting, keep live viewing nonblocking, and keep the user model minimal.
  It should record honest code provenance levels so runs can be traced across
  lab computers without false reproducibility claims. Non-managed user-run
  Python may be `unmanaged`; future managed runs can require immutable code
  snapshots. v0.2 should not yet manage full code sync or environment setup.
  It is local-only: Fricon Desktop and the Python SDK operate through one local
  service and one primary local data library; the bundled CLI is mainly for
  setup, diagnostics, service control, and developer workflows.
- v0.3: candidate product-assistance slice after the v0.2 measurement loop is
  usable. The leading candidate is read-only LAN viewing from another computer.
  Other likely areas include richer sample fields and 2D sample maps,
  comparison views, saved views, portable export viewer polish, attach/correct
  context UX, passive setup/device snapshots, rerun-from-artifact, locked-down
  Windows installer polish, guided new-computer setup, shared
  measurement-code source update flows, and better code-provenance/environment
  summaries.
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
3. `measurement-requirements.md` - focused product requirements for the
   central v0.2 measurement workflow
4. `dataset-artifact-requirements.md` - focused product requirements and
   scenario checks for the first concrete artifact type
5. `technical-direction.md` - supporting engineering posture and API/runtime
   direction
6. `future-concepts.md` - active ledger for lower-confidence or less narrowed
   ideas

Use current implementation notes only to understand what must be replaced or
adapted:

- `../current-storage-notes.md`
- `../current-dataset-semantics.md`

Use supporting and process notes for design or implementation context:

- `../database-schema-changes.md`
- `../maintenance-checklist.md`

Use archived proposal inputs only for historical rationale or edge-case context:

- `archive/README.md`
- `archive/dataset-semantic-architecture-proposal.md`
- `archive/measurement-system-foundation-redesign.md`
- `archive/experiment-run-and-runner-design.md`
- `archive/parameter-management-design.md`

The archived proposals are not active v0.2 guidance. They may preserve older
sequencing, terminology, or boundaries that were superseded by `design.md`,
`product-direction.md`, and `technical-direction.md`.

## Relationship To Older Docs

The older project route was dataset-first, Python-led, and local-first. v0.2
keeps the Python-led and local-first parts, but broadens the foundation from a
dataset catalog to a measurement record system:

- data library instead of user-facing workspace
- optional active sample and session identity before measurement history gets
  fragmented
- measurements as the default data-taking work record
- artifacts as outputs, with datasets as the primary table-shaped case
- parameter and code provenance as explicit recorded context
- analysis, simulation, import, and calibration as reserved future activity
  records that consume and produce artifacts
- code provenance summaries that record whether acquisition code was unmanaged,
  user-supplied, or resolved from a managed code snapshot without making the
  data library a shared code repository

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
- multi-computer setup assumptions that rely on copying code folders or running
  active measurement code from a shared editable network directory

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
- live preview/export/analysis consumers that can block acquisition writes
- LabRAD Data Vault/Grapher compatibility layer for old scripts
- direct built-in import of old LabRAD Data Vault storage
- automatic calibration, managed analysis, or managed device communication in
  the first replacement slice
- driver marketplace or regulated-lab compliance UX in the first replacement
  slice
- remote mode or browser/PWA distribution as shipped v0.2 features
- remote annotations or remote acquisition writes
- Labber-like visual sweep builder as a product goal
- full Git client, Git hosting, or automatic source-control workflow for
  ordinary experimenters
- central Fricon server required only to distribute measurement code
- shared editable network folder as the primary measurement-code workflow

Remote access, authentication boundaries, and actor/audit records should be
planned early, but the first product should remain local-first and single-owner.
