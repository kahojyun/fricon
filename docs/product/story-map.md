# Story Map

## Status

Draft pending product-analysis revalidation.

## Purpose

Keep the product route readable for future AI sessions. This file is a compact
map from the product vision to initial adoption stories, not the place for full
acceptance details.

## Backbone

```text
Install and set up
  -> create/open data library
  -> select optional sample/session context
  -> run Python measurement
  -> record dataset artifacts
  -> inspect live
  -> finish/recover
  -> annotate
  -> reopen in Python
  -> export
```

## Initial Adoption Product Epics

The initial adoption route is the LabRAD Data Vault/Grapher replacement loop
for new measurements: write from Python, watch live, recover partial data,
reopen, and export. The epics below divide that route without making legacy
import or managed automation part of the first adoption slice.

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
summaries that old scripts currently leave in folders and operator memory.

Python SDK UX is part of the product story, not only implementation detail.
The product-level SDK usage guideline lives in `product/python-sdk-ux.md`;
detailed API signatures and capture mechanics belong in later ADRs/specs.

Strategic follow-on work should turn initial adoption facts into local
experiment memory and reviewed action. Those priorities are outside the initial
adoption story index and live as draft backlog context in
`product/future-concepts.md`.

Export remains an initial adoption product promise. Detailed
export/offline-analysis requirements should be derived in a later spec after
the product, architecture, and ADR boundaries are accepted.
