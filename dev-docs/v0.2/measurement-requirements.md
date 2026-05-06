# Measurement Product Requirements

## Status

Focused product requirements note for the v0.2 reset.

This is not current behavior and not an implementation contract. Use this file
to align the user-visible measurement workflow before writing ADRs for storage
layout, lifecycle records, service APIs, Python SDK types, Desktop views,
events, export bundles, or compatibility policy.

Read `README.md`, `design.md`, and `product-direction.md` first. This note
narrows the central v0.2 workflow. `dataset-artifact-requirements.md` is a
supporting note for the first concrete artifact type produced by measurements.

## Product Thesis

`Measurement` is the first public acquisition noun in v0.2. A measurement is a
data-taking attempt that may produce one or more dataset artifacts and may carry
sample/session context, notes, lifecycle state, parameter snapshots, and code
provenance summaries.

The v0.2 product goal is modest:

```text
run Python measurement code
record a measurement record
produce dataset artifacts
watch table/plot views while writing
recover partial data
reopen and export the measurement later
```

This replaces the everyday LabRAD Data Vault and Grapher logging loop for new
measurements. It does not make Fricon a visual sweep builder, a LabRAD
compatibility layer, a full parameter registry, or a device-control framework.

## User Promise

For v0.2, a measurement should answer these questions clearly:

- What was this data-taking attempt called, and when did it run?
- Was a sample or sample session selected, missing, or corrected later?
- Which dataset artifacts did it produce?
- Is it active, finished, interrupted, failed, aborted, trashed, or recovered?
- What notes or markers explain what happened?
- What optional parameter snapshot and code provenance summary were recorded?
- How do I watch it live, reopen it from Python, or export it for analysis?

For v0.3+ and v0.4+, the same measurement model should leave room for:

- read-only remote monitoring
- richer sample/session comparison and saved views
- passive setup/device snapshots
- analysis records that consume measurement outputs
- calibration records and parameter proposals
- managed measurement templates and managed script runs

## Concept Boundary

`Measurement` owns user-facing data-taking context:

- stable measurement identity
- human title or name
- start/end timestamps and basic timing metadata
- active, finished, interrupted, failed, aborted, trash/recover state
- optional sample/session link and later corrections
- produced artifact links, especially dataset artifacts
- measurement notes, markers, favorites or pins, and optional tags
- optional parameter snapshot link
- optional code provenance summary link
- optional setup/method labels
- export/reopen entry points

`Measurement` does not own lower-level or future-specific facts:

- dataset payload facts, scan schema, variable roles, or chart-readiness details
- sample field schema, inventory behavior, or sample maps
- global parameter profile management, calibration promotion, or validation
- managed code snapshots, execution worktrees, or task queues
- device communication, driver state, or readback enforcement
- analysis conclusions, stitching, resampling, or derived result semantics
- full export bundle format, checksums, or offline viewer implementation

Those facts belong to `DatasetArtifact`, `Sample`, `SampleSession`,
`ParameterSnapshot`, `CodeProvenanceSummary`, future `Analysis` and
`Calibration` records, device/setup records, or focused export requirements.

## v0.2 Requirements

### Measurement Creation

Measurement creation should be explicit but short in Python.

Acceptance notes:

- The normal SDK path creates a measurement before dataset writes.
- The library handle should be cheap to reuse across notebook cells.
- Fricon should not create hidden measurements behind every low-level dataset
  write in the normal examples.
- A measurement may run without sample/session context.
- Multiple local measurements may be active concurrently through the service.
- Incompatible clients must fail before writes when practical.

### Optional Sample And Session Context

Sample/session context is useful but optional.

Acceptance notes:

- Active context may come from Desktop, CLI, or Python prelude code.
- The measurement records resolved sample/session IDs when context is used.
- Missing context is visible and fixable later.
- Wrong context can be corrected with history.
- Dataset artifacts inherit context for display through the measurement; they
  should not become owners of sample/session identity.

### Produced Artifacts

A measurement may produce one or more dataset artifacts and light attachments.

Acceptance notes:

- Dataset writers share the measurement lifecycle in the normal path.
- Different index models or different logical tables should usually be
  separate dataset artifacts linked to the same measurement.
- A measurement can expose produced datasets in the console with status,
  table/plot actions, and export actions.
- Reports, derived results, device snapshots, and parameter proposals are
  future artifact kinds, not v0.2 measurement requirements.

### Lifecycle And Partial Recovery

Partial data must stay visible and recoverable.

Acceptance notes:

- Active, finished, interrupted, failed, and aborted states are visible.
- If a script crashes, the measurement remains visible with completed dataset
  facts preserved.
- Rerun creates a new linked measurement by default.
- Appending to an older measurement requires explicit resume intent and
  compatibility checks.
- Ordinary cleanup uses trash/recover, not hard delete.
- Broad manual good/suspect classification should remain optional notes or tags
  unless real workflows justify a first-class field.

### Notes And Annotation

Users should annotate at the measurement level by default.

Acceptance notes:

- Favorites or pins are the primary manual signal for important measurements.
- Notes and markers can be added during or after a measurement.
- Notes and system events appear in a measurement timeline.
- Tags remain optional and secondary.
- Dataset-level notes are reserved for output-local exceptions.
- A lightweight operator profile may label mutating actions, but users should
  not be prompted for an operator on every measurement.

### Parameter Snapshot And Code Provenance

v0.2 should record minimal optional context without making parameter or code
systems first-class workflow surfaces.

Acceptance notes:

- A measurement may link an optional flexible parameter snapshot.
- v0.2 does not require a global parameter registry or profile UI.
- A measurement may record a code provenance level such as unmanaged,
  user-supplied summary, or future managed snapshot.
- Non-managed user-run code should not be presented as reproducible.
- Setup/method labels may be recorded as human-readable context.
- Full code-source management, managed snapshots, and managed execution are
  v0.3+ or v0.4+ product work.

### Live Inspection

Live views are operational monitors, not publication plotting tools.

Acceptance notes:

- The Desktop first screen should be a measurement console, not a generic
  dataset browser.
- The console shows active and recent measurements, active sample/session
  context, produced datasets, and live status.
- Table, line/scatter, basic heatmap, and simple trace inspection are enough
  for v0.2.
- Live plotting, previews, export preparation, and future analysis hooks are
  noncritical consumers and must not slow or fail acquisition writes.
- Users may detach measurement, table, or plot windows to watch multiple active
  measurements.
- Newly started Python measurements should be highlighted without stealing
  focus or auto-opening windows.

### Python Reopen

Users should reopen measurement outputs without storage paths.

Acceptance notes:

- Python snippets use stable measurement or dataset IDs.
- Reads go through the public SDK and service or direct export reader, not
  internal storage paths.
- The SDK should expose semantic dataset reads through measurement outputs.
- Service discovery should find or start the local service where practical and
  fail with guided diagnostics otherwise.

### Measurement Export

Measurement-centered export is the default portability workflow.

Acceptance notes:

- Export starts from a measurement by default.
- Export includes produced dataset artifacts and enough metadata for offline
  analysis.
- Export should preview sensitive provenance or local paths before including
  them.
- Python should open an export bundle directly.
- Desktop read-only export viewing is useful but can be polished after the
  core write/reopen loop.
- Detailed bundle format, checksums, and offline viewer behavior belong in a
  focused export requirements note or ADR before implementation.

### Migration From LabRAD-Style Scripts

Fricon should help users migrate new measurement work away from LabRAD without
depending on old Data Vault internals.

Acceptance notes:

- New Data Vault-style dataset declarations should map naturally to
  measurement-scoped Fricon dataset writers.
- Independent/dependent variable semantics belong to dataset artifact
  requirements.
- Fricon should provide generic APIs that user-written import scripts can call.
- Fricon should not ship a Data Vault storage parser or old Grapher/Data Vault
  compatibility layer in v0.2.

## Interaction With Other Modules

### Dataset Artifacts

Dataset artifacts own typed data facts, scan schema, variable roles, and
chart-readiness. Measurements own why the data was taken, what context was
selected, and which artifacts were produced.

### Measurement Console

The console owns the primary v0.2 Desktop workflow. It should present
measurements first and datasets as outputs inside measurements.

### Python SDK

The Python SDK owns measurement creation ergonomics, writer lifecycle, service
discovery diagnostics, and stable reopen snippets.

### Sample And Sample Session

Sample/session context is an optional link on the measurement. Attach-later and
correction workflows belong to measurement history, not dataset metadata.

### Parameter And Provenance

Measurements link optional parameter snapshots and code provenance summaries.
They do not own parameter profile mutation, calibration acceptance, code-source
updates, or managed execution.

### Analysis And Calibration

Analysis and calibration consume completed measurement outputs and produce
derived artifacts, result summaries, or parameter proposals. They should not
silently mutate measurement or dataset facts.

### Export

Export is measurement-centered but cross-cutting. A separate requirements note
or ADR should own bundle format, checksums, offline viewer behavior, privacy
preview, and import/read policy when implementation starts.

## User Stories

### Run An Exploratory Measurement

As an experimentalist, I want to run a measurement from Python with minimal
boilerplate so that Fricon records the run, produced datasets, optional
sample/session context, and basic provenance.

Acceptance notes:

- The Python SDK uses an explicit but short measurement-creation call.
- Measurement-scoped dataset writers finalize with the measurement lifecycle.
- Optional parameter snapshots can be attached without a global parameter
  registry.
- Missing sample/session context is visible and fixable later.

### Watch And Inspect Data

As an experimentalist, I want local live and historical table/chart views so
that I can decide whether a measurement is working.

Acceptance notes:

- The console centers active and recent measurements.
- Live views are noncritical consumers.
- Interrupted or partial measurements remain visible and recoverable.
- Users can detach data or plot windows to watch multiple active measurements.

### Recover A Partial Measurement

As an experimentalist, I want interrupted data to stay visible and readable so
that I can decide whether to keep, rerun, export, or invalidate the run.

Acceptance notes:

- Completed dataset facts remain readable.
- The measurement status explains the interruption.
- Rerun creates a new linked measurement by default.
- Notes or invalidation events can explain what happened.

### Annotate Once At The Right Level

As an experimentalist, I want to favorite important measurements and add notes
or optional tags at the measurement or sample/session level so that I do not
have to annotate every dataset.

Acceptance notes:

- Favorites or pins are quick to apply.
- Notes and markers appear in the measurement timeline.
- Dataset-level notes are used only for output-local exceptions.

### Reopen Data From Python

As an analyst, I want stable IDs and read snippets so that I can reopen
measurement outputs without knowing storage paths.

Acceptance notes:

- Snippets use the public Python SDK.
- Measurement outputs expose semantic dataset reads.
- Export bundles can be opened directly for offline analysis.

### Export A Measurement For Offline Analysis

As an experimentalist, I want to export a complete measurement bundle so that I
can analyze it on another computer without setting up a Fricon data library.

Acceptance notes:

- Export starts from a measurement by default.
- Produced datasets and relevant metadata are included.
- Sensitive provenance is previewed or opt-in.
- Importing into another data library is optional, not required for analysis.

### Migrate A LabRAD-Style Measurement Script

As a lab user, I want to translate new Data Vault-style measurement code into
Fricon with minimal conceptual change so that I can stop using Data Vault for
new measurements without rewriting the whole experiment stack.

Acceptance notes:

- The migration path uses the Fricon SDK, not a Data Vault compatibility layer.
- Dataset-specific independent/dependent mappings are covered by
  `dataset-artifact-requirements.md`.
- User-written import scripts can call generic Fricon APIs when old data needs
  to be brought forward.

## v0.2 Non-Goals

- Full LabRAD Data Vault or Grapher compatibility.
- Built-in import or browsing of old Data Vault storage.
- Visual sweep builder as the primary acquisition model.
- Full parameter profile or calibration workflow.
- Managed code-source updates or managed execution.
- Device communication or readback enforcement.
- Publication plotting or dashboard building.
- Multi-user roles, remote writes, or hosted collaboration.
- Full export viewer polish before the write/reopen loop works.

## Product Decisions To Confirm

These decisions should be confirmed before durable measurement storage/API
contracts:

1. What is the minimal lifecycle state vocabulary for v0.2 measurements?
2. Which measurement fields are immutable facts, and which are corrected by
   event history?
3. What minimal parameter snapshot shape is useful without a parameter
   registry?
4. What code provenance levels should v0.2 expose by default?
5. Which export requirements are part of v0.2, and which belong to a later
   focused export requirements note?
