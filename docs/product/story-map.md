# Story Map

## Status

Draft rederived from current high-confidence inputs; older story and epic IDs
retired.

## Purpose

Keep the current initial-adoption route readable without preserving older draft
story or epic IDs. Fresh stable IDs can be added later when the rederived story
boundaries are accepted enough to need traceability. Do not use this file as
implementation requirements until the product baseline, capability map, and
validation posture are accepted.

## Current Evidence Basis

This map is based on:

- `vision.md`
- `personas.md`
- the VNA S21, IQ/readout, and generic irregular-step interview evidence in
  `product-analysis-progress.md`
- redacted sample-code pressure from legacy measurement workflows
- the scope challenge that narrowed first adoption to write, simple live
  inspection, checkpoint-safe reads, and reopen

Older numbered epics and user stories were draft derived material. They are no
longer part of the active product route and should not be reused. Future story
IDs should be allocated from the accepted rederived boundaries.

## Initial Adoption Backbone

```text
install and launch local Fricon
  -> create or open one local data library
  -> start ordinary Python measurement code
  -> declare enough dataset shape for plotting and reading
  -> append checkpointed measurement data
  -> inspect simple live views
  -> preserve already-written data after ordinary interruption cases
  -> reopen measurement or dataset by stable ID
  -> migrate the next simple script without making old workflows harder
```

Export, fuller viewer workflows, parameter systems, managed code, and device
communication are follow-on paths. They should not block the first usable
write/watch/reopen loop.

## Initial Adoption Story Slices

### Local Product Readiness

Users need Fricon Desktop, CLI, Python SDK, and required local runtime pieces to
behave like one coherent local product on a normal lab computer. Setup should
support constrained lab machines, including offline, locked-down, Windows, or
pinned-environment situations where practical.

Success means a measurement user can launch Desktop or use Python without
assembling mismatched pieces, and compatibility problems fail before mutating a
library.

### Local Data Library

Users need one normal lab-computer data library with durable identity, a
remembered location, and compatibility checks before writes. Shared-folder
multi-writer semantics, hosted service behavior, and account administration are
not part of first adoption.

### Unmanaged Python Measurement

Users keep ordinary scripts and notebooks in control. Fricon provides a
low-ceremony way to create a measurement and append data from unmanaged Python
without adopting a managed runner, task queue, device framework, or visual
sweep builder.

The first concrete case is a VNA S21 measurement that sweeps DC voltage and VNA
power while recording trace-valued outputs.

### Dataset Shape And Recording

Dataset artifacts must stay directly openable and searchable under a
measurement. The recording path should cover:

- regular one-, two-, and N-dimensional sweeps
- trace-valued records with explicit coordinate/value arrays or compact
  regular-coordinate descriptions
- multiple traces in one outer sweep record
- first-class complex values so magnitude/phase or I/Q views come from data
  semantics
- IQ averages, I/Q channels, single-shot arrays, or labels
- generic irregular or ragged step records

Minimizer or optimizer output should start as ordinary irregular/step data, not
as a special first-slice product model.

### Simple Live Inspection

Live inspection is a disposable consumer, not part of the write acknowledgement
path. The first live monitor should emphasize simple current views:

- recent one-dimensional line or scatter
- simple two-dimensional heatmap
- selected output or trace channel
- simple magnitude/phase or I/Q view when semantics support it
- IQ scatter for single-shot/readout work

The live monitor should not grow into a full analysis viewer. Row selection,
coarse/fine overlays, rich comparison, fitting, classifier tuning, and
publication plots can stay in the fuller viewer or user Python scripts.

### Checkpoint-Safe Readability

First adoption should protect already-written data from ordinary interruption
cases such as user interrupt or notebook-kernel failure. It should not promise
hard-crash or power-loss recovery beyond later durable-write architecture
decisions.

Partial or interrupted measurements should be visible as incomplete; Fricon
should not hide partial state to make data look complete.

### Reopen By Stable ID

The first analysis path is local reopen. Users should be able to copy a stable
measurement or dataset ID, preferably from a right-click menu and keyboard
shortcut, then use a predictable reader call from Python.

Reader design should start from the actual analysis tasks:

- plot several selected traces in one line plot
- build basic heatmaps from sweep-plus-trace data
- read IQ single-shot data as ndarray-like data when shape permits
- inspect generic irregular step records

Early readers can be generic enough for users to wrap in experiment-specific
helpers. Polished framework-specific views and portable export should follow
observed use.

### Direct Dataset Browser

Even with a measurement-first Desktop, dataset artifacts must remain
searchable and directly openable. A dataset view should show parent measurement
context, basic table or plot previews, and fast stable-ID copy.

### Notes, Attributes, And Attachments

Fricon should support lightweight notes, attributes, markers, favorites, pins,
and small attachments at measurement or dataset level. A plain attachment list
is enough for first adoption.

Physical setup, wiring, and lab-environment facts that Fricon does not
understand should remain user-supplied notes, attributes, or attachments.
Fricon should not imply that it can judge these facts as fresh, stale, trusted,
or ambiguous unless a later model owns the evidence.

### Honest Software-Visible Context

Initial adoption can record what Fricon can honestly know or what the user
explicitly supplies:

- unmanaged code label
- optional script or source-folder reference
- selected local configuration files, such as parameter or registry files
- demod/readout setting files or summaries when supplied
- environment labels or summaries where practical
- unmanaged procedure summaries

This is evidence, not a managed parameter system, code-deployment system, or
physical setup truth model.

### Migration Without Old-System Import

Users should be able to translate a simple Data Vault-style new measurement
script by rewriting the recording section, not by emulating LabRAD or importing
old history first.

Migration guidance should cover the hard shapes that appeared in the interview:
VNA traces, coarse/fine trace collections, first-class complex values, IQ-like
data, and generic irregular records. Old history can remain in old systems
while new runs move to Fricon.

## Deferred Backlog

These are real product pressures, but they should not be first-slice blockers:

- portable Fricon export packages and generic file exports
- read-only offline bundle viewer
- rich historical viewer, saved views, and comparison workflows
- polished Polars, pandas, NumPy, or xarray-specific reader views
- trace concatenation helpers and coarse/fine overlay polish
- analysis, fitting, classifier-tuning, and report artifact records
- parameter profiles, snapshots, diffs, and proposals
- managed code sources, approved updates, and managed execution
- calibration chains and reviewed automation
- structured setup/device state, communication, reconciliation, and apply
- rich sample maps and spatial visualization
- remote or LAN monitoring

## Next Product Step

Use `capability-map.md` and `product-analysis-progress.md` to challenge and
separate the first usable slice, follow-on backlog, ADR-gated directions, and
rejected scope.
