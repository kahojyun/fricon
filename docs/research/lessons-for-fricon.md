# Lessons For Fricon

## Status

Draft research synthesis.

## Review Date

2026-05-06.

## Sources

Primary/project-maintained sources reviewed:

- QCoDeS measurement/dataset docs:
  https://microsoft.github.io/Qcodes/examples/DataSet/Performing-measurements-using-qcodes-parameters-and-dataset.html
- QCoDeS datasaver builder:
  https://microsoft.github.io/Qcodes/examples/DataSet/Datasaver_Builder.html
- QCoDeS export docs:
  https://microsoft.github.io/Qcodes/examples/DataSet/Exporting-data-to-other-file-formats.html
- QCoDeS Pandas/XArray docs:
  https://microsoft.github.io/Qcodes/examples/DataSet/Working-With-Pandas-and-XArray.html
- LabRAD Data Vault/Grapher quick start:
  https://sourceforge.net/p/labrad/wiki/QuickStartDataVaultAndGrapher/
- Bluesky Event Model:
  https://blueskyproject.io/event-model/main/explanations/data-model.html
- Ophyd device/hints docs:
  https://blueskyproject.io/ophyd/device-overview.html
- Bluesky callbacks and interruption/state-machine docs:
  https://blueskyproject.io/bluesky/v1.13/callbacks.html
  https://blueskyproject.io/bluesky/v1.14.0/state-machine.html
- Databroker and Tiled docs:
  https://blueskyproject.io/databroker/
  https://blueskyproject.io/tiled/getting-started/what-is-tiled.html
- Bluesky Tiled Plugins layout:
  https://blueskyproject.io/bluesky-tiled-plugins/explanations/layout.html
- Suitcase usage:
  https://blueskyproject.io/suitcase/usage.html
- Keysight Labber product docs:
  https://www.keysight.com/us/en/products/all-instrument-software/labber-software.html
- ARTIQ data interfaces and core language docs:
  https://m-labs.hk/artiq/manual/using_data_interfaces.html
  https://m-labs.hk/artiq/manual/core_language_reference.html
- ARTIQ environment and management docs:
  https://m-labs.hk/artiq/manual/environment.html
  https://m-labs.hk/artiq/manual/management_system.html
- NIST ARTIQ scan framework:
  https://pages.nist.gov/artiq_scan_framework/scans/core_features.html

## Synthesis

### Measurement-first is supported, but datasets stay first-class

Existing systems support a run or measurement record with metadata and
lifecycle, but QCoDeS and LabRAD also make datasets easy to find and reopen.

Fricon should make Desktop measurement-first while keeping `DatasetArtifact` a
stable, searchable, directly openable artifact. This is especially important
for future analysis, import, simulation, calibration, and export workflows.

### Scan schema needs shape modes

Explicit scan semantics are strongly supported, but a simple
independent/dependent pair is not enough.

Fricon v0.2 should model:

- regular grids
- partial grids with missing expected points
- irregular or adaptive points
- repeated points
- fixed-shape arrays/traces
- variable-length traces with per-trace coordinates and settings

Display hints should stay separate from durable semantic facts.

The public API also needs ergonomic Python-native scan plans or helpers for
common scan/trace shapes. Raw schema should remain available for advanced
cases, but raw schema alone is too much ceremony for routine 1D/2D/N-D
measurements.

### Internal streams should not become the normal user concept

Bluesky/Tiled style systems show that stream-like grouping is useful for
primary, baseline, external arrays, and export/read layout. Fricon should allow
internal stream or subpayload representation where storage/export/read APIs
need it, while keeping `DatasetArtifact` as the normal user-facing concept in
v0.2.

### Readable partials come before resumable execution

All reviewed systems care about partial/interrupted data. Some also support
pause/resume, but resumability requires scan-point checkpoint semantics and
runner cooperation.

Fricon v0.2 should guarantee readable partial data and explicit missing/partial
shape semantics. Resumable execution should wait for a managed-runner design.

### Live views must be disposable consumers

Live plotting and dashboards should not be on the acquisition write path.
Durable writes come first; live events, previews, and charts are bounded,
coalesced, throttled, or opt-in consumers.

### Passive setup provenance is useful before device control

Labber and ARTIQ show that device/setup information matters. Fricon should not
build a broad device framework in v0.2, but measurements should have room for a
passive setup/device/environment summary that describes context without
claiming control or full reproducibility.

Measurements should also allow passive procedure summaries: unmanaged script,
external runner, or declared plan context. This records what was intended or
invoked without claiming managed execution or resume semantics.

### Export must support bundles and common analysis workflows

Portable export should preserve measurement metadata, dataset semantics,
checksums, and source identity. Common analysis formats such as CSV, Parquet,
NetCDF, XArray-oriented views, or similar paths should be considered in a
focused export spec rather than deferred indefinitely.
