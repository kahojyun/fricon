# Capability Map

## Status

Draft pending product-analysis revalidation.

## Vision Alignment

The capability baseline should be revalidated against one initial adoption
goal: replace the simple LabRAD Data Vault/Grapher loop for new interactive
measurements and portable later analysis. Capabilities in the first adoption
slice should support that loop without importing old history, emulating LabRAD,
running user plotting code, generating reports, or pulling strategic follow-on
parameter, code-management, runner, device, or automation systems into the
first release path.

Strategic follow-on capability ordering is draft backlog context in
`product/future-concepts.md`. This map keeps stable IDs and compact scope
boundaries only.

Release versions are not product-horizon labels. Compatible strategic
follow-on capabilities may still ship on the same compatible release line; use
initial adoption, strategic follow-on, and ADR-gated labels for product
planning.

## Product Grouping

Use these planning groups when routing product work:

- Local adoption: CAP-001, CAP-002, CAP-012, CAP-013.
- Initial adoption measurement replacement: CAP-003, CAP-005, CAP-006,
  CAP-007, CAP-028, CAP-030.
- Context and provenance: CAP-004, CAP-014, CAP-015, CAP-017, CAP-027,
  CAP-029, CAP-033.
- Review and analysis: CAP-008, CAP-009, CAP-010, CAP-011, CAP-016, CAP-026.
- Strategic follow-on foundation: CAP-018 through CAP-025, CAP-031, CAP-032.

## Capability Definitions

CAP-001: Local data library.

- Promise: one normal lab computer can own a Fricon data library with durable
  identity, remembered location, and format version.
- Includes: create/open, recent-library memory, library identity, and clear
  handling for locked or unsupported libraries.
- Excludes: hosted service, shared-folder multi-writer database semantics, and
  multi-user administration.

CAP-002: Install and launch local Fricon.

- Promise: Desktop, CLI, Python SDK, and required local Fricon runtime
  components are delivered as one coherent local product.
- Includes: first-run setup, local runtime startup/connection diagnostics, and
  headless Python use when Desktop is closed. Product analysis should account
  for Windows lab PCs, locked-down/offline computers, and slow lab update
  cycles.
- Excludes: polished enterprise deployment, exact installer mechanics, and
  third-party protocol stability. Exact runtime/process packaging belongs to
  later architecture.

CAP-003: Python measurement recording.

- Promise: a Python script or notebook can explicitly create an unmanaged
  measurement with low ceremony and append measurement facts through public
  APIs.
- Includes: title, start/end lifecycle, optional context links, concurrent local
  writers, readable crash outcomes, and simple recording-section rewrites for
  Data Vault-style scripts.
- Excludes: managed runner, task queue, visual sweep builder, LabRAD
  compatibility layer, and device control.

CAP-004: Optional, visible, correctable sample/session context.

- Promise: sample and sample-session context can explain a measurement when it
  matters, without blocking quick exploratory work.
- Includes: optional active context, visible context on creation, and correction
  after a run.
- Excludes: requiring a complete sample registry before recording data.

CAP-005: Dataset artifact recording.

- Promise: one measurement can produce one or more first-class dataset
  artifacts with stable IDs and semantic metadata.
- Includes: measurement-scoped writers, lifecycle sharing by default, and
  storage/export hooks for internal streams when needed.
- Excludes: making internal streams the normal user-facing concept.

CAP-006: Dataset scan semantics.

- Promise: plotted datasets carry enough schema for slicing and plotting
  without relying on column-position guesses.
- Includes: variable roles, dependencies, axis structure, partial grids,
  irregular/adaptive points, logical step records, per-step parameters,
  selectable trace collections, fixed-shape traces, and variable-length traces.
- Excludes: requiring every advanced acquisition to fit a regular grid, or
  blocking writes only because optional units or labels are absent.

CAP-007: Nonblocking live inspection.

- Promise: Desktop can watch active data without slowing or breaking acquisition
  writes.
- Includes: live events, line/scatter plot, basic heatmap, selected output or
  trace views, simple trace inspection, stale/lag indicators, and product
  support for watching multiple relevant measurement or data views at once.
- Excludes: live consumers as required write acknowledgements, running user
  plotting code, and making full table browsing the primary live monitor
  surface.

CAP-008: Lifecycle and readable partial recovery.

- Promise: interrupted or failed work remains understandable and readable.
- Includes: lifecycle states, partial data reads, missing expected points where
  schema supports them, trash/recover, and failure/interruption visibility.
- Excludes: resuming unmanaged execution from the last scan point.

CAP-009: Notes, markers, favorites, pins, and optional tags.

- Promise: users can mark what matters during and after measurement work without
  turning Fricon into a full ELN.
- Includes: measurement notes, dataset notes, markers, favorites, pins, optional
  tags, and correction events.
- Excludes: publication notebook replacement and rich collaborative review.

CAP-010: Python reopen through stable IDs and public APIs.

- Promise: Python can reopen measurements, dataset artifacts, and exports
  without depending on storage paths.
- Includes: stable IDs, typed read APIs, schema-aware reads, partial-data
  semantics, Desktop-visible snippets, and analysis-friendly reads into common
  Python objects such as NumPy, pandas, or Polars where appropriate.
- Excludes: path-based storage contracts and private file layout coupling.

CAP-011: Measurement-centered export.

- Promise: a completed or interrupted measurement can become a portable bundle
  for offline analysis.
- Includes: produced datasets, semantic metadata, selected provenance,
  integrity metadata, simple manifest or index preview, direct Python reading,
  privacy-aware provenance selection, and a lightweight Python reader that does
  not require running the acquisition-time local runtime or Desktop.
- Excludes: importing old history, first-slice report generation, a fully
  polished offline Desktop viewer before the write/reopen loop is proven, and
  promising generic CSV, Parquet, or NumPy exports before real demand is
  validated.

CAP-012: Backup, restore, and migration checkpoints.

- Promise: format-changing operations have a recovery posture before user data
  is mutated.
- Includes: backup/restore guidance, repair or migration checkpoints, and
  ordinary recoverable cleanup paths.
- Excludes: full legacy-system migration and distributed backup service.

CAP-013: Compatibility diagnostics and fail-before-write checks.

- Promise: incompatible client, local runtime, or library combinations fail
  before mutation with actionable diagnostics.
- Includes: version negotiation for Desktop, local runtime components, CLI,
  Python SDK, and data library format; diagnostics for stopped runtime
  components, wrong library, locked library, old SDK, incompatible components,
  migration-required state, and unsafe update timing.
- Excludes: accepting best-effort writes from unknown or stale clients.

CAP-014: Honest code provenance summary.

- Promise: Fricon records what it can honestly know about unmanaged Python code
  without pretending it owns execution.
- Includes: unmanaged labels, optional script path, copied-folder or
  source-root label, user-supplied summary, selected local configuration
  snapshots, optional Git summary where meaningful, and privacy-aware export
  handling.
- Excludes: automatic notebook state capture and managed code snapshots.

CAP-015: Light parameter context summary without a registry UI.

- Promise: a measurement can keep optional parameter context before Fricon has
  parameter profiles, effective snapshots, or calibration workflows.
- Includes: user-supplied structured or semi-structured parameter summaries,
  units where supplied, and links to measurement context.
- Excludes: global parameter registry, immutable profile binding, override
  semantics, proposal workflow, and calibration promotion.

CAP-016: Light measurement attachments.

- Promise: a measurement can reference small supporting files needed to
  understand or analyze the run.
- Includes: lightweight attachments, labels, source metadata, and export
  selection.
- Excludes: large detector-file management and general media asset library.

CAP-017: Lightweight operator profile and audit actor.

- Promise: notes, corrections, lifecycle events, and exports can name the local
  operator or actor responsible.
- Includes: local operator label, audit actor on events, and correction history.
- Excludes: accounts, permissions, teams, and identity-provider integration.

CAP-018: Read-only remote monitoring.

- Intent: allow trusted users on the lab network to watch measurements without
  editing the data library.
- Boundary: the initial adoption slice may keep APIs observable, but should not
  promise remote access or shared editing.

CAP-019: Rich sample maps and saved views.

- Intent: support user-defined spatial sample maps, richer sample metadata, and
  saved comparison views after the local data-library loop works.
- Boundary: initial adoption only needs optional sample/session context and
  correction. Strategic follow-on map details belong in a dedicated spec or
  ADR.

CAP-020: Measurement-code source setup and approved code update flows.

- Intent: help labs manage code source locations and reviewed updates for
  measurement scripts, replacing copied working folders as the normal
  explanation for where run, analysis, and calibration code came from.
- Includes later product pressure from local folders, Git/Gitea repositories,
  mirrored repositories, packages, importable Python entry points, approved
  releases, setup profiles, environment lock files, scan-schema helpers,
  measurement templates, plot presets, export recipes, and maintainer handoff
  for local changes.
- Boundary: initial adoption records honest provenance but does not own code
  deployment. Scheduler or managed-execution behavior belongs to later
  capabilities.

CAP-021: Parameter profiles and proposals.

- Intent: promote repeated parameter snapshots into reusable profiles and
  reviewed proposals, with diffs and source evidence that make calibration
  changes inspectable instead of anonymous config-file edits.
- Boundary: initial adoption keeps only light parameter context summaries,
  without a registry UI or effective-configuration model.

CAP-022: Analysis, interpretation, and calibration records.

- Intent: model downstream analysis, fit attempts, interpretation decisions,
  anomaly review, calibration, and derived-result activity as first-class
  records that can cite input measurements, code context, parameter snapshots,
  generated artifacts, fitted values, and affected parameter paths.
- Boundary: initial adoption may export analysis-ready data but does not manage
  calibration promotion. Detailed calibration workflow semantics belong in the
  future backlog.

CAP-023: Managed code snapshots and execution.

- Intent: run selected measurement code under Fricon control with stronger code
  provenance snapshots and lifecycle supervision.
- Boundary: managed code starts from selected importable Python entry points or
  SDK-integrated wrappers, not a general scheduler, shell-command runner, or
  workflow DAG.
- Boundary: initial adoption records unmanaged execution context only.

CAP-024: Device boundary and managed device communication.

- Intent: introduce explicit device identity, configuration, and communication
  boundaries when Fricon begins controlling instruments, including later
  desired-state planning, observed-state readback, reconciliation diffs, and
  reviewed apply plans.
- Boundary: initial adoption may record passive setup/device summaries but does
  not talk to instruments. Device apply is ADR-gated.

CAP-025: AI-assisted reviewed automation.

- Intent: let AI propose actions or analysis steps that are reviewed before
  mutating the data library.
- Boundary: initial adoption should preserve auditability, not implement
  mutating AI automation. AI-created durable conclusions should record
  provenance, and mutating AI actions should enter through the same reviewed
  proposal path as non-AI automation.

CAP-026: Dataset artifact discovery and direct open.

- Promise: dataset artifacts remain searchable and directly openable even when
  the Desktop home is measurement-first.
- Includes: stable artifact IDs, measurement context, search/open entry points,
  and direct navigation to selector, plot, or trace views.
- Excludes: returning to a dataset-first product model.

CAP-027: Passive setup and environment summary.

- Promise: measurements can record setup, device, method, software, or
  environment context without controlling the lab setup.
- Includes: optional labels, freeform summaries, external references, and
  privacy-aware export selection.
- Excludes: device identity registry, calibration records, and communication
  with instruments.

CAP-028: Python-native scan plans plus raw schema escape hatch.

- Promise: common scan shapes are easy to declare in ordinary Python while
  uncommon schemas remain possible.
- Includes: low-ceremony scan/schema authoring for common 1D, 2D, N-D,
  irregular step-record, and trace cases; dict/literal-friendly authoring where
  useful; optional convenience helpers; and raw schema construction for
  advanced users.
- Excludes: a fixed framework-specific helper name, a framework-specific
  parameter-object model, treating first-draft helper syntax as accepted
  without usage feedback, a visual sweep builder, or a managed execution plan.

CAP-029: Passive procedure summary.

- Promise: users can describe the procedure that produced data even when Fricon
  did not execute that procedure.
- Includes: unmanaged script summary, external-runner reference, declared plan
  summary, and operator correction.
- Excludes: managed measurement plans, resumable execution, and scan-point
  checkpoints.

CAP-030: Migration ergonomics for Data Vault-style new measurement scripts.

- Promise: users can translate new Data Vault-style scripts to Fricon writers
  without a full experiment-stack rewrite.
- Includes: natural mapping for independent/dependent variables, labels, units,
  legends, old path aliases, numbered legacy titles as metadata, and migration
  guide examples for non-obvious shapes such as N-D sweeps, coarse/fine trace
  collections, and irregular optimizer records.
- Excludes: a LabRAD compatibility server, LabRAD-dependent helper module,
  built-in Data Vault parser, or old history browser.

CAP-031: Run manifest and failure investigation.

- Intent: give managed or high-provenance runs a compact manifest linking
  parameter snapshots, target keys, code/environment summary, lifecycle, logs,
  artifacts, operator, timestamps, diagnostics, previous-good baselines,
  calibration evidence, generated sidecars, review decisions, and handoff
  state.
- Boundary: a run manifest is a composite view over available facts, not a
  duplicate owner of parameter, code, setup, analysis, or artifact records.
  Initial adoption should record facts that later support compare, handoff, and
  anomaly investigation without accepting a run-manifest model yet.

CAP-032: Routine recipes and reviewed replay.

- Intent: capture repeated compare, run, and analyze routines as previewable
  recipes so tedious lab work can be replayed without hidden mutation,
  especially when calibration chains need working refs, health gates, and
  reviewed promotion to durable parameter refs, or when imperative loop bodies
  would repeatedly issue avoidable device writes.
- Boundary: read-only batch compare or triage can arrive before
  mutation-capable automation; durable mutation requires the reviewed proposal
  and audit model described by future specs and ADRs.

CAP-033: Run-bound local configuration snapshot.

- Promise: a measurement can bind selected local configuration files,
  references, hashes, or summaries so later analysis can identify the effective
  lab-local state without reading copied folders by hand.
- Includes: user-selected parameter files, registry files, wiring references,
  line/chip information, demod/readout settings, runner labels, source aliases,
  privacy-aware export selection, and correction history.
- Excludes: automatic tracing of every file read, global parameter profiles,
  calibration promotion, device control, or claiming unmanaged execution was
  fully reproducible.
