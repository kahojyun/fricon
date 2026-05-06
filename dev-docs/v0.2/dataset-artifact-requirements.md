# Dataset Artifact Product Requirements

## Status

Focused product requirements note for the v0.2 reset.

This is not current behavior and not an implementation contract. Use this file
to align user stories, acceptance criteria, and product boundaries before
writing ADRs for storage layout, manifest format, service APIs, Python SDK
types, chart DTOs, live events, or portable exports.

Read `README.md`, `design.md`, `product-direction.md`, and
`measurement-requirements.md` first. This note narrows the dataset artifact
slice inside that broader measurement-centered model.

## Product Thesis

`DatasetArtifact` is the first concrete artifact type because replacing a
simple LabRAD Data Vault and Grapher workflow requires measured data that can be
written from Python, watched live, reopened later, sliced into useful plots, and
exported for analysis.

The product should feel natural to users migrating from LabRAD:

```text
create a measured dataset
append rows or batches while a measurement runs
watch it in a graph or table
reopen it by title, time, sample/session, or source alias
read it from Python without remembering storage paths
```

The product should not preserve LabRAD's limiting assumptions:

```text
dataset identity is a folder-local counter
all measured data is one scalar numeric table
metadata lives only in file attributes
read progress is hidden in a connection context
plot meaning is inferred from column order
```

## User Promise

For v0.2, a dataset artifact should answer these questions clearly:

- What measurement produced this data?
- Is the data still being written, finished, interrupted, or aborted?
- What variables exist, what are their labels and units, and which variables
  are axes, measured values, monitors, fixed values, or system fields?
- Which table or plot actions should the measurement console suggest by
  default?
- How do I slice a 1D or 2D plot without guessing column order?
- How do I reopen the same facts from Python or a portable export?
- Which metadata belongs to the dataset itself, and which context is inherited
  from the producing measurement or later provenance records?

For v0.3+ and v0.4+, the same artifact model should leave room for:

- remote read-only monitoring
- richer comparison and saved views
- sample-map and session-aware filtering
- analysis records that consume and produce artifacts
- calibration records and parameter proposals
- passive device/setup snapshots
- managed measurement templates and plot presets

## Concept Boundary

`DatasetArtifact` owns dataset-local facts:

- stable dataset artifact identity
- append state and completion state
- typed variables and field metadata
- append-only records or chunks
- dataset-local scan schema and lightweight display hints
- write/completion state and output-local exceptions
- dataset-local references to external data when a later ADR allows them
- direct Python/Desktop/export readability

`DatasetArtifact` does not own broader measurement context:

- sample or sample-session identity
- measurement intent, notes, or lifecycle history
- parameter snapshot ownership
- code provenance or environment summaries
- calibration decisions
- analysis conclusions
- global tags, favorites, or saved searches, except where a dataset-local
  exception is genuinely needed

Those facts belong to `Measurement`, `Sample`, `SampleSession`,
`ParameterSnapshot`, `CodeProvenanceSummary`, `Analysis`, `Calibration`, or
library-level view records. Dataset metadata may display inherited context, but
it should not become the owner of that context.

## v0.2 Requirements

### Measurement-Scoped Creation

The normal path is measurement-scoped:

```text
Measurement starts
  -> one or more DatasetArtifacts are declared
  -> writers append facts
  -> live consumers receive nonblocking updates
  -> Measurement finishes, interrupts, or aborts
  -> completed DatasetArtifacts become immutable facts
  -> corrections or derived results are recorded as events or new artifacts
```

Acceptance notes:

- A measurement may produce multiple datasets.
- Dataset artifacts should be one user-facing logical data table or trace
  collection in v0.2. If one measurement produces raw data indexed by
  `(x, y, z)` and an aggregate indexed by `(x, y)`, those should normally be
  separate dataset artifacts linked to the same measurement.
- Do not expose multiple named tables or streams inside one dataset artifact as
  a v0.2 product concept. A later storage/API ADR may still split payloads
  internally if Arrow files, chunking, or external assets make that useful.
- A dataset may be created without sample/session context when the measurement
  has none.
- A dataset may be lower-level or scratch, but the normal public examples
  should create datasets through a measurement.
- A crashed script leaves partial data visible and recoverable.
- Rerunning a script creates a new linked measurement by default; appending to a
  previous dataset requires explicit resume intent and compatibility checks.

### Data Shapes And Types

The v0.2 product must support table-shaped measurement data with enough type
coverage for LabRAD replacement and common physics-lab scans.

Required first-slice value kinds:

- numeric integer and float
- complex numeric values
- boolean
- text or categorical labels
- timestamp or time offset
- nullable values
- fixed-shape arrays or traces when declared up front
- variable-length traces when declared up front

Product direction for v0.2+:

- ragged arrays and variable-length traces should be supported through declared
  trace or array semantics, not through ad hoc JSON blobs
- arbitrary Python objects are not dataset values; users should serialize them
  as parameters, attachments, or external artifacts with clear type metadata
- large images, spectra, waveforms, row-linked external arrays, and other
  binary payload workflows are ADR-gated follow-up scope unless a concrete v0.2
  measurement requires them

Variable-length trace recording is a v0.2 product requirement. A common
measurement may tune instrument settings such as VNA bandwidth, sweep range,
and point count during one scan, then write several `(frequency, S21)` traces
with different lengths. Fricon should record those traces without forcing a
fake shared frequency grid or padding with null-heavy columns. Stitching,
resampling, and display policy can be chart or analysis work, but the recorded
facts must preserve each trace's own coordinate values, measured values, and
per-trace acquisition settings.

The product requirement is about preserved meaning and user workflows, not the
physical representation. Choices such as Arrow struct/list columns, sidecar
trace chunks, or external asset thresholds belong in the dataset storage and
API ADRs.

### Variables, Roles, And Dependencies

Datasets intended for live or historical plotting must carry explicit scan
schema at creation time.

Each public variable should have:

- stable variable ID or name
- human label
- optional unit
- value type and shape
- role such as axis/independent, measured/dependent, fixed/config,
  monitor/readback, or system
- per-dependent axis/dependency links
- optional legend or handle for traces
- optional display hints

Acceptance notes:

- Do not infer important plot meaning from field order.
- Do not require every dependent value to depend on every sweep axis.
- Standalone monitor or baseline values should remain first-class.
- Duplicate or repeated points need a declared display policy, such as latest
  by record ID, show all, or require user choice.
- Uncertainty columns, rich source metadata, and publication display roles are
  future or measurement-specific extensions unless a concrete v0.2 workflow
  proves they are needed.
- The UI may guess a schema for scratch or imported unplotted data, but guessed
  schema is not the normal path for measurement data that users expect to plot.

### Live Monitoring

Live viewing is a noncritical consumer of acquisition.

Acceptance notes:

- Dataset writes must not wait for chart rendering, preview transforms, export
  preparation, or analysis hooks.
- Live readers should tail explicit append positions, not hidden connection
  cursors.
- A live monitor can attach after a dataset starts and request the current
  summary plus later updates.
- Update delivery may coalesce or drop preview updates under load, but it must
  not drop committed data.
- The user should be able to watch multiple active measurements through the
  measurement console and detachable data or plot windows.
- Live status should distinguish active, draining, finished, interrupted,
  failed, and aborted writes.

### Chart Slicing And Display

The chart system should consume dataset semantics instead of raw storage
columns.

v0.2 chart scope:

- table view
- line and scatter plots
- basic 2D heatmap or image view from tabular axes
- simple trace inspection for declared fixed-shape or variable-length traces

Acceptance notes:

- Default x/y/color choices come from variable roles and dependencies.
- Axis labels combine variable label and unit.
- Plot titles should come from measurement title, dataset title, sample/session
  context when available, and lightweight display hints.
- Multi-axis datasets need explicit slice controls for fixed axes.
- The default slice should be predictable, such as latest value for unspecified
  axes, first value, or a declared preferred value.
- Users should be able to switch dependent variables without rebuilding the
  entire measurement context.
- Stitching, averaging, resampling, best-trace selection, saved plot presets,
  and rich saved views are analysis, chart, or future product work. v0.2 should
  preserve enough facts and hints for those workflows without implementing them
  as dataset requirements.

### LabRAD Migration Feel

Fricon v0.2 should make new measurement scripts easy to migrate from Data Vault
without building a Data Vault compatibility layer or promising direct import of
old Data Vault storage.

Acceptance notes:

- Preserve the mental model of independent variables, dependent variables,
  labels, legends, and units.
- Provide generic source-alias and source-metadata fields so user-written
  migration scripts can preserve original paths, numbered titles, and source
  IDs without making them primary identity.
- Let user-written migration scripts map LabRAD-style parameters, comments,
  timestamps, and tags into Fricon records without Fricon depending on Data
  Vault implementation details.
- Provide a low-boilerplate append API that resembles `new` plus `add`, while
  recording richer Fricon semantics.
- Replace LabRAD's implicit `get` cursor with explicit reader positions and
  subscriptions.

### Direct Python Readability

Users should reopen dataset artifacts from Python without knowing storage
paths.

Acceptance notes:

- Python read APIs should expose semantic reads, not only raw Arrow files.
- Users should be able to request a table, a dependent-with-axes view, or a
  grid-like view when the schema supports it.
- Rectangular gridded data should have enough labels, units, and axis metadata
  for later xarray/NetCDF-style interoperability.
- Irregular or adaptive scans should remain readable as row or per-dependent
  data without forcing a fake grid.
- CSV export is useful but lossy; product UX should pair it with a manifest or
  explain what metadata is lost.

### Portable Export

Measurement-centered export is the default.

Acceptance notes:

- Exporting a measurement includes its dataset artifacts, semantics, relevant
  metadata, notes/events, sample/session labels when safe, and provenance
  summaries selected for export.
- Exported datasets open read-only from Python or Desktop without importing
  into a data library.
- Exports should include checksums and stable IDs.
- Common tabular files such as CSV or Parquet may be included for convenience,
  but the Fricon manifest is the source of meaning.
- Sensitive provenance and local paths should be previewed or opt-in.

## Interaction With Other Modules

### Measurement Console

The console should show active and recent measurements first. Datasets appear
as outputs inside each measurement, with status, row/chunk counts, default plot
actions, table actions, and export actions.

### Live Monitor

The live monitor subscribes to measurement and dataset events. It should not
open files directly or depend on polling the final artifact. It should use the
same semantics that historical chart views use.

### Charts

Charts consume resolved dataset interpretation:

- variable roles
- dependency graph
- axis labels and units
- duplicate policy
- slice options
- lightweight display hints
- live append positions or historical read ranges

Charts should not own dataset semantics. User-selected chart presets, saved
views, publication styling, stitching, averaging, and resampling should be chart
or analysis outputs, not hidden mutations of dataset facts.

### Sample And Sample Session

Dataset views may display sample/session context inherited from the producing
measurement. Dataset records should not copy sample/session identity as their
own owner unless the dataset is imported or unassigned and needs correction.

### Parameters And Code Provenance

Dataset views may show relevant parameter snapshots or code provenance
summaries through the producing measurement. Dataset-local metadata is for
dataset-local facts, not for whole-run reproducibility claims.

### Analysis And Calibration

Analysis and calibration are v0.3+ or v0.4+ product layers. They should consume
dataset artifacts through stable IDs and semantic views, then produce derived
artifacts, result summaries, or parameter proposals. They should not mutate
completed dataset facts silently.

### Import And Legacy Data

Imported datasets should record source identity, conversion notes, original
paths or IDs, checksums when available, and any inferred or user-corrected
schema. Fricon should provide generic APIs that let users write their own
legacy import scripts, including scripts for LabRAD Data Vault data when they
understand their lab's old storage. Fricon should not depend on LabRAD
implementation details, ship a Data Vault parser, or promise direct legacy
browsing as part of the v0.2 product.

## Dataset Scenario Checks

These scenarios are not separate product workflows. They are concrete checks
that the dataset artifact model can support the measurement workflows owned by
`measurement-requirements.md`.

### 1D Scalar Or Trace Scan

As an experimentalist migrating from Data Vault, I want to create a measurement
dataset with one independent variable and one or more dependent traces so that I
can watch a live line plot while the script appends points.

Acceptance notes:

- The Python setup is short enough to replace common `new` and `add` usage.
- The plot opens with the expected x label, y label, unit text, and trace
  legend.
- If the script crashes, the partial line remains visible and marked partial.

### 2D Scan

As an experimentalist, I want to declare two scan axes and one measured value so
that Fricon can show a live heatmap and let me inspect line cuts without
guessing which column is which.

Acceptance notes:

- The dataset declares both axes and their units.
- The heatmap can update during acquisition.
- Repeated points use a visible duplicate policy.
- The user can switch between heatmap and x/y line cuts.

### Mixed Dependencies

As an experimentalist, I want one measurement to produce multiple values with
different dependencies so that I can record a primary signal, monitor channels,
fit estimates, and trace data without forcing all of them into one global grid.

Acceptance notes:

- Each dependent variable declares its own axes.
- Monitor values can be shown in a table or small status plot.
- Trace rows can be inspected without breaking the scalar plot.

### Adaptive VNA Traces

As an experimentalist measuring a cavity, I want to write multiple `(frequency,
S21)` traces with different sweep ranges, bandwidths, and point counts so that
I can optimize measurement precision and runtime without losing the original
trace facts.

Acceptance notes:

- Each trace can have a different length.
- Each trace records its own frequency coordinate values and S21 values.
- Per-trace settings such as bandwidth, range, point count, power, averaging,
  and timestamp can be recorded beside the trace.
- Fricon does not require padding, resampling, or a shared grid at write time.
- Later stitching, resampling, or best-trace selection creates analysis output
  or display interpretation, not silent mutation of the recorded trace facts.

## v0.2 Non-Goals

- Full LabRAD Data Vault compatibility server.
- Full legacy LabRAD history browser.
- Direct built-in import of old LabRAD Data Vault storage.
- Publication-quality plotting or figure layout.
- Generic dashboard builder.
- Full HDF5/NeXus/Labber compatibility layer.
- User-facing multiple table or named stream containers inside one dataset
  artifact.
- Row-linked external asset management for large binary values unless a v0.2
  ADR proves a narrow need.
- Saved plot preset, rich saved view, stitching, resampling, or aggregation
  systems.
- Visual sweep builder as the primary acquisition model.
- Broad hardware driver framework.
- Automatic notebook state capture.
- Remote write access or multi-user permission model.

## Product Decisions To Confirm

These decisions should be confirmed before the dataset storage/API ADRs:

1. What product constraints should guide the technical representation for
   fixed-shape arrays and variable-length traces, such as expected trace size,
   live-read latency, Python ergonomics, export fidelity, and external asset
   thresholds?
2. Should lower-level standalone datasets remain a public Python happy path, or
   should they be an advanced API beneath measurement-scoped examples?
3. What is the minimum artifact-level attachment or external-reference support
   needed for v0.2, if any, before deferring row-linked external assets?
