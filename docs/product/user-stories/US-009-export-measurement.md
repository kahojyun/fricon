# US-009: Export A Measurement For Offline Analysis

## Status

Accepted.

## Primary Epic

EPIC-004: Recovery, annotation, reopen, and export.

## Story

As an experimentalist, I want to export a complete measurement bundle so that I
can analyze it on another computer without setting up a Fricon data library or
importing the data first.

## Success Criteria

- Export starts from a measurement by default.
- Produced datasets and semantic metadata are included.
- Common analysis files are included where practical, but the Fricon manifest
  remains the source of meaning.
- A simple human-readable manifest or index preview helps users inspect an
  exported bundle before loading it in code.
- Selected code, setup, procedure, and run-bound configuration summaries can be
  included so the bundle explains the measurement without old local folders.
- Sensitive provenance is previewed or opt-in, including local paths, dirty
  code details, environment summaries, source computer labels, setup details,
  and extended sample metadata.
- Python can open the bundle directly.
- A read-only Desktop bundle viewer remains useful product pressure, but full
  viewer polish can follow after the write/reopen/export loop works.

## Not In Scope

- Importing the bundle into another editable data library as the default path.
- Legacy Data Vault, Labber, or HDF5 compatibility layers.

## Related Capabilities

CAP-010, CAP-011.
