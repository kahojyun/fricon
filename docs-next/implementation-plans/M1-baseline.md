# M1: v0.2 Foundation Baseline

## Status

Draft implementation plan.

## Goal

Build the first coherent v0.2 foundation for a local data library with
measurement-scoped dataset artifacts, enough lifecycle/provenance to recover
and inspect runs, and enough service/API shape to avoid another dataset-first
dead end.

## Scope

M1 should prove:

- local data library identity and open/create flow
- explicit measurement record
- optional sample/session link
- measurement-scoped dataset artifact writer
- dataset artifact direct search/open path
- explicit scan schema and shape modes for plotted datasets
- scan schema helpers for common 1D/2D/N-D scans and traces
- internal stream/subpayload policy for dataset artifacts
- nonblocking live status/event flow
- readable partial/interrupted measurement data
- optional passive setup summary
- optional passive procedure summary
- Python reopen through stable IDs
- early compatibility negotiation before writes

## Out Of Scope

- full legacy workspace migration
- direct LabRAD/Data Vault import
- remote mode
- full parameter registry
- managed code execution
- resumable scan-point checkpoint execution
- user-facing stream concepts inside dataset artifacts
- broad device framework
- large external detector-file/image management
- calibration workflows
- AI mutating automation

## Suggested Work Sequence

1. Accept or revise ADRs for reset, data-library storage, service API, and
   dataset artifact storage.
2. Finalize `SPEC-001-data-library-measurement-foundation`.
3. Create vertical slices for data library, measurement, dataset artifact, and
   service API.
4. Implement Python measurement writer ergonomics against the service.
5. Replace the Desktop first screen with a measurement console skeleton.
6. Add dataset direct search/open entry points.
7. Add live read path and nonblocking chart/table preview.
8. Add lifecycle, readable partial data, and trash/recover basics.
9. Add Python reopen snippets and semantic read APIs.
10. Split measurement-centered export into SPEC-002 and keep it aligned with
    SPEC-001 semantic reads.
11. Add focused validation and docs sync.

## Readiness Gate

M1 is ready to implement when:

- `SPEC-001` status is Accepted
- ADRs exist for storage layout and service API direction
- lifecycle state vocabulary is accepted
- scan shape and readable-partial semantics are accepted
- scan helper and internal stream policies are accepted
- tests and validation plan cover Rust, Python, Desktop, and compatibility
- migration/import stance for pre-v0.2 data is explicit
