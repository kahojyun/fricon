# US-009: Export A Measurement For Offline Analysis

## Status

Draft pending product-analysis revalidation.

## Primary Epic

EPIC-004: Recovery, annotation, reopen, and export.

## Story

As an Experimental Data Analyst, I want to export a complete measurement bundle
so that I can analyze it on another computer without setting up a Fricon data
library or importing the data first.

## Success Criteria

- Export starts from a measurement by default.
- Produced datasets and semantic metadata are included.
- A portable Fricon package with a lightweight Python reader is the baseline;
  generic analysis files can be added when demand is validated.
- A simple human-readable manifest or index preview helps users inspect an
  exported bundle before loading it in code.
- Selected code, setup, procedure, and run-bound configuration summaries can be
  included so the bundle explains the measurement without old local folders.
- Sensitive provenance is previewed or opt-in, including local paths, code
  details, environment summaries, source computer labels, setup details, and
  extended sample metadata.
- Python can open the bundle directly through reader views consistent with
  local reopen APIs where practical.
- A read-only Desktop bundle viewer remains useful product pressure, but full
  viewer polish can follow after the write/reopen/export loop works.

## Not In Scope

- Importing the bundle into another editable data library as the default path.
- Legacy Data Vault, Labber, or HDF5 compatibility layers.
- First-slice report or presentation generation.

## Related Capabilities

CAP-010, CAP-011.
