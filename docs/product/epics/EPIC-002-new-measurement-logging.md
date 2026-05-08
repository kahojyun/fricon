# EPIC-002: New Measurement Logging Replacement

## Status

Draft pending product-analysis revalidation.

## Product Goal

Replace the simple LabRAD Data Vault/Grapher loop for new interactive
measurements: declare measured data, append values from Python, watch live
monitor plots, and reopen it later without manual file/folder discipline.

This is the center of the initial adoption slice. It should stay close to
ordinary Python measurement scripts instead of becoming a managed automation
framework.

## Initial Adoption Scope

- Explicit but low-ceremony Python measurement creation.
- Measurement-scoped dataset artifact writers.
- Dataset artifacts remain directly searchable and openable.
- Python-native scan plans or helpers for common 1D/2D/N-D scans, irregular
  step records, and traces, with raw schema for advanced cases.
- Explicit scan semantics for reliable live and historical plots.

## Not Initial Adoption

- LabRAD compatibility server, LabRAD-dependent helper module, or built-in
  Data Vault parser.
- Visual sweep builder as the primary acquisition model.
- Full managed runner, device control, or parameter registry.

## Key Stories

- US-003: Run an interactive unmanaged measurement from Python.
- US-004: Produce dataset artifacts from a measurement.
- US-015: Declare common scan shapes quickly.

## Related Migration Pressure

US-017 is owned by EPIC-005. EPIC-002 should stay focused on the new writer
model, but that model must remain ergonomic enough for Data Vault-style
measurement scripts to migrate without a full old-system importer.
