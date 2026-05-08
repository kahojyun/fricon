# Architecture Notes

## Status

Pre-architecture placeholder.

## Purpose

Architecture is not the active design surface yet. Product analysis is still
upstream. Until those inputs are accepted, this directory
should only preserve accepted constraints and collect questions for later ADRs
or specs.

Do not treat transport choices, storage shape, module names, Python API syntax,
runtime flows, or export format details in this directory as accepted
architecture unless an ADR accepts them.

## Accepted Constraints

- v0.2 is a clean reset from the pre-v0.2 workspace/dataset-first model.
- Fricon remains local-first for the initial adoption slice; hosted SaaS,
  accounts, teams, permissions, and distributed database semantics are out of
  scope.
- Mutating clients must fail compatibility checks before writing to a data
  library.
- Desktop UI state must not become the durable data backend.
- Live inspection must not block acquisition writes.
- Measurement-first UX must not hide dataset artifacts; datasets remain
  searchable and directly openable.
- Dataset artifacts must not own measurement, sample, lifecycle, parameter,
  code, or provenance meaning.
- Strategic follow-on runner, device, calibration, and AI mutation systems
  require later ADRs/specs before implementation.

## Deferred Architecture Questions

Answer these only after the relevant product baseline is accepted or explicitly
marked with open interview questions:

- local runtime/process model and client discovery
- local runtime/API transport and protocol shape
- binary dataset payload format and chunking
- storage layout, schema, migrations, and checkpoints
- event/audit record schema
- Python SDK exact names, signatures, context mechanics, and writer object model
- export bundle format and offline-reader responsibilities
- concrete Rust crate/module boundaries
- Desktop shell versus browser-capable frontend boundaries
- migration/import routes for any real pre-v0.2 data

Do not create or expand architecture detail files until product inputs are
accepted and the user explicitly starts architecture design.

## Current Files

- `compatibility-policy.md` owns the accepted clean-reset compatibility policy
  and draft future compatibility gates.
- Detailed API, storage, runtime flow, system overview, module boundary, and
  architecture risk files are intentionally absent during product analysis.
