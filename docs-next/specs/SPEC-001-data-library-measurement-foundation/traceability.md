# SPEC-001: Traceability

## Status

Draft.

## Related Capabilities

CAP-001, CAP-002, CAP-003, CAP-004, CAP-005, CAP-006, CAP-007, CAP-008,
CAP-009, CAP-010, CAP-012, CAP-013, CAP-014, CAP-015, CAP-017, CAP-026,
CAP-027, CAP-028, CAP-029.

## Related User Stories

US-001, US-002, US-003, US-004, US-005, US-006, US-007, US-008, US-010,
US-011, US-012, US-013, US-014, US-015, US-016.

## Related Domain Concepts

DataLibrary, Measurement, DatasetArtifact, Sample, SampleSession,
ParameterSnapshot, CodeProvenanceSummary, SetupProvenanceSummary,
ProcedureSummary, Event/AuditRecord, OperatorProfile.

## Related Architecture Docs

- `architecture/system-overview.md`
- `architecture/module-boundaries.md`
- `architecture/data-flow.md`
- `architecture/api-boundaries.md`
- `architecture/storage-model.md`
- `architecture/compatibility-policy.md`

## Related ADRs

- ADR-001: v0.2 Clean Reset Boundary
- ADR-002: Documentation Governance For v0.2+
- Pending: data-library storage layout
- Pending: service API contract
- Pending: dataset artifact storage and scan schema
- Pending: lifecycle and event model
- Pending: scan shape and readable-partial semantics
- Pending: passive setup summary shape
- Pending: passive procedure summary shape
- Pending: scan schema helper API shape
- Pending: internal stream/subpayload representation, if needed

## Affected Modules

Expected high-impact areas:

- `crates/fricon/src/**`
- `crates/fricon-py/**`
- `crates/fricon-ui/src/**`
- `crates/fricon-ui/frontend/src/**`
- `docs-next/**`
- later public `docs/**` after behavior lands

## Compatibility Impact

This spec intentionally breaks pre-v0.2 workspace/dataset-first assumptions.

Before implementation, confirm whether any pre-v0.2 data migration/import path
is required. Default policy is no automatic compatibility.

## Validation Links

See `validation.md`.
