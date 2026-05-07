# Storage Model

## Status

Draft. Requires ADRs before implementation.

## First-Slice Records

```text
DataLibrary
  Sample
  SampleSession
  Measurement
  DatasetArtifact
  AttachmentArtifact
  ParameterSummary
  RunConfigSnapshot
  CodeProvenanceSummary
  SetupProvenanceSummary
  ProcedureSummary
  Event/AuditRecord
  OperatorProfile
```

## Storage Responsibilities

| Area | Responsibility |
| --- | --- |
| Catalog | Stable record IDs, display names, timestamps, lifecycle state, links. |
| Dataset payloads | Arrow-compatible chunked facts and append positions. |
| Dataset semantics | Variable roles, labels, units, scan axes, dependencies, duplicate policy, display hints. |
| Scan shape semantics | Regular grid, partial grid, irregular/adaptive points, repeated points, fixed arrays, and variable-length traces. |
| Internal payload grouping | Optional internal stream-like groups for storage/export/read APIs without making streams a normal user concept. |
| Events | Measurement lifecycle, notes, corrections, export/recovery/system actions. |
| Provenance summaries | Parameter summaries, run-bound configuration snapshots, code provenance, passive setup/device/environment summaries, and passive procedure summaries. |
| Checkpoints | Backup/restore and migration safety. |
| Export manifests | Portable read-only package identity, checksums, selected metadata. |

## Compatibility Policy Hooks

Storage must expose:

- data-library format version
- feature/capability flags where needed
- migration state and last successful checkpoint
- active writer/import/export/migration blockers
- enough metadata for clients to fail before writes

## v0.1 Reuse Candidates

- Arrow IPC chunk concepts
- append-only record IDs
- semantic manifest validation ideas
- logical-index sidecars if they fit the v0.2 scan schema model
- dataset archive checksum and manifest concepts

## v0.1 Redesign Candidates

- `.fricon_workspace.json` as user-facing root identity
- dataset-only SQLite catalog
- dataset-local ownership of favorite/status/tag meaning that belongs to
  measurement or sample/session context
- archive/import/export formats that cannot carry data-library and
  measurement-centered provenance

## ADRs Needed

- data-library layout and migration policy
- measurement table versus generic activity-run table
- dataset artifact storage for fixed arrays and variable-length traces
- scan shape and partial-read semantics
- internal stream/subpayload representation, if needed
- external asset/reference hooks for future detector files and images
- event/audit record schema
- export bundle format
- local actor/token storage
