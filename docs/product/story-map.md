# Story Map

## Status

Draft pending product-analysis revalidation; first interview pass added.

## Purpose

Keep the product route readable for future AI sessions. This file is a compact
map from the product vision to initial adoption stories, not the place for full
acceptance details.

## Backbone

```text
Install and set up
  -> create/open data library
  -> optionally set sample/session context
  -> run unmanaged Python measurement
  -> declare scan, step, array, or trace shape
  -> record dataset artifacts incrementally
  -> inspect live
  -> finish/recover
  -> annotate or bind selected local context
  -> reopen in Python
  -> export
```

## Candidate Initial Adoption Journey

This journey is based on the first interview pass and redacted legacy sample
pressure. It is still draft; it should guide the next interview pass before
capabilities are rederived.

The concrete first-adoption case is a simple VNA S21 measurement. The user
keeps existing instrument calls, waveform generation, notebooks, local
parameter files, plotting utilities, and calibration helpers outside Fricon.
Fricon takes over the maintained measurement-data loop: record a measurement,
write dataset artifacts, show enough live inspection, preserve partial data,
reopen outputs from Python, and export a portable result for analysis.

The normal run shape is:

1. A user starts from an ordinary Python notebook or script and opens a local
   Fricon library.
2. The user optionally selects minimal context, such as a sample, cooldown,
   session, or human-readable note. Physical setup details are helpful
   evidence, but not required to start the run.
3. The script creates an unmanaged measurement and declares the expected data
   shape. For the VNA case, outer sweep axes may include DC voltage and VNA
   power, while each row carries a trace with its own frequency axis and S21
   values.
4. The script appends data durably as acquisition proceeds. First-adoption
   dataset shapes must cover regular grids, partial grids, trace-valued
   records, IQ averages or I/Q channels, single-shot arrays or labels, and
   irregular minimizer steps.
5. Desktop live inspection shows the latest useful view without becoming part
   of the write path. Essential views are recent 1D lines, basic 2D heatmaps,
   selected trace inspection, and IQ scatter plots for single-shot/readout
   work.
6. If the script stops, crashes, or is interrupted, data already written stays
   readable through the browser/viewer and public read APIs. Fricon should not
   hide partial state or silently discard written records.
7. The user can attach selected local context such as parameter files,
   registry files, wiring references, demod/readout settings, script labels, or
   notes. These are evidence for interpretation, not a guarantee that Fricon
   knows the physical setup or wiring is current.
8. Later, an analyst opens the measurement or dataset by stable ID, ideally by
   copying a reader snippet from the UI. The minimum useful read result is a
   table or table-like object containing sweep parameters and measured results,
   plus access to preserved attached files.
9. Export remains a handoff path for analysis on another computer. The first
   export promise is a portable Fricon package plus reader APIs, not broad
   report generation or legacy-system import.

The first success standard is reliability and easy reopen: Fricon is worth
continuing to use if it can keep running, record measurement identity and
produced data, preserve written partials, show the data in a browser/viewer,
and provide copyable Python reader snippets for measurements and datasets.

## Backbone Implications From Interview

- Dataset artifacts must cover trace-valued outputs, not only scalar dependent
  columns. VNA frequency is often an inner trace coordinate, while bias, power,
  or other settings may be outer sweep axes or run metadata.
- Complex-like values are common in readout and VNA workflows. First adoption
  may store them as explicit I/Q or magnitude/phase channels if that keeps
  reading and plotting clear, but the product language should not pretend they
  are ordinary scalar-only measurements.
- IQ scatter is a native inspection need, especially for single-shot data and
  readout classification work.
- Optimizer or minimizer traces should be easy to record as irregular step
  records. Legacy print-only optimizer evidence is a migration pain point.
- Setup, wiring, and mutable configuration are useful context but weak truth
  sources. The initial slice should preserve selected files and summaries,
  show ambiguity or staleness where possible, and avoid claiming device-control
  or reproducibility guarantees.
- The UI must treat copyable reader snippets as a first-adoption product
  affordance, not only a developer convenience.

## Initial Adoption Product Epics

The initial adoption route is the LabRAD Data Vault/Grapher replacement loop
for new interactive measurements: write from Python, watch live monitor plots,
recover partial data, reopen, and export for analysis on another machine. The
epics below divide that route without making legacy import, LabRAD emulation,
report generation, user plotting code, or managed automation part of the first
adoption slice.

- EPIC-001: Local setup and data-library adoption.
- EPIC-002: New measurement logging replacement.
- EPIC-003: Measurement console and inspection.
- EPIC-004: Recovery, annotation, reopen, and export.
- EPIC-005: Migration ergonomics and future lab scaling.

Detailed epic notes live under `product/epics/`.

## Initial Adoption Story Index

Ownership rules:

- Epics own outcomes. A story may affect another epic, but it should have only
  one primary owner.
- Story persona wording follows `product/personas.md`; use the role that owns
  the outcome instead of a broad lab identity.
- Keep this index compact. Story-level success criteria live under
  `product/user-stories/`.
- Each expanded story file names its primary epic.
- Put implementation acceptance detail in specs, not in this map.

Foundation:

- US-001: Install and launch Fricon on a lab computer.
- US-002: Create or open one local data library.
- US-010: Update without corrupting measurement work.

Measurement loop:

- US-003: Run an interactive unmanaged measurement from Python.
- US-004: Produce dataset artifacts from a measurement.
- US-005: Watch and inspect live data.
- US-015: Declare common scan shapes quickly.

Context and provenance:

- US-007: Annotate at the right level.
- US-011: Record code provenance honestly.
- US-012: Register a sample and sample session when useful.
- US-014: Record passive setup context.
- US-016: Record passive procedure context.
- US-019: Capture run-bound local configuration.

Recovery and analysis:

- US-006: Recover a partial measurement.
- US-008: Reopen measurement outputs from Python.
- US-009: Export a measurement for offline analysis.
- US-013: Find and open a dataset artifact directly.

Migration:

- US-017: Migrate a Data Vault-style script to Fricon writers.
- US-018: Start new work in Fricon while old history stays in the old system.

Migration ownership:

- US-017 is owned by EPIC-005. It pressures EPIC-002 writer ergonomics but does
  not make EPIC-002 responsible for old-system import.
- US-018 is owned by EPIC-005. EPIC-004 owns reopen/export for Fricon data, not
  the product migration posture.

Each initial adoption story has an expanded file under `product/user-stories/`.
Keep those files concise: they own story-level success criteria, not
implementation tasks.

## Sequencing

The earliest implementation slice should prove enough of US-001 through US-008
and US-015 to record, inspect, recover, and reopen a Python measurement while
preserving first-class dataset discovery. The milestone plan should be derived
after the product baseline and required architecture or ADR inputs are accepted.

The full initial adoption slice also needs US-009 for export, US-017/US-018 to
validate the incremental adoption posture, and US-019 to make copied local
configuration visible. Users should be able to translate new Data Vault-style
scripts, keep old history in the old system, and bind the run-relevant files or
summaries that old scripts currently leave in folders and operator memory. This
means a small explicit rewrite of the recording section, not a built-in LabRAD
compatibility layer.

Python SDK UX is part of the product story, not only implementation detail.
The product-level SDK usage guideline lives in `product/python-sdk-ux.md`;
detailed API signatures and capture mechanics belong in later ADRs/specs.

Strategic follow-on work should turn initial adoption facts into local
experiment memory and reviewed action. Those priorities are outside the initial
adoption story index and live as draft backlog context in
`product/future-concepts.md`.

Export remains an initial adoption product promise. A portable Fricon package
with a lightweight Python reader is the current baseline; reader APIs should
load data into NumPy, pandas, Polars, or similar analysis objects where the
dataset shape supports it. CSV, Parquet, or other generic file exports should
be added when real user demand justifies them. Detailed export/offline-analysis
requirements should be derived in a later spec after the product, architecture,
and ADR boundaries are accepted.
