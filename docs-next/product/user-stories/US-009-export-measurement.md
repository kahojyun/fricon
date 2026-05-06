# US-009: Export A Measurement For Offline Analysis

## Status

Accepted. Detailed requirements live in `../../specs/SPEC-002-measurement-export-offline-analysis/`.

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
- Sensitive provenance is previewed or opt-in.
- Python can open the bundle directly.

## Not In Scope

- Importing the bundle into another editable data library as the default path.
- Legacy Data Vault, Labber, or HDF5 compatibility layers.

## Related Capabilities / Specs

CAP-010, CAP-011, SPEC-002.
