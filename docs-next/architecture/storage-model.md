# Deferred Storage Questions

## Status

Deferred. Requires domain acceptance and ADRs before implementation.

## Upstream Domain Inputs

The domain model owns logical concepts such as `DataLibrary`, `Sample`,
`SampleSession`, `Measurement`, `DatasetArtifact`, `AttachmentArtifact`,
provenance summaries, lifecycle events, and operator or system actors.

Those concepts are not yet storage tables, files, payload groups, manifests, or
indexes.

## Deferred Storage Questions

- What is the data-library root layout?
- Which durable IDs, timestamps, lifecycle states, links, and display fields
  belong in the catalog?
- Which logical concepts become separate tables versus embedded records or
  event payloads?
- How are dataset table facts, scan semantics, append positions, and large
  payloads stored?
- How are fixed-shape arrays and variable-length traces represented?
- Are internal stream-like groups needed behind dataset artifact APIs?
- Which events are append-only audit records, lifecycle records, or derived
  indexes?
- What checkpoint, backup, restore, migration, and repair metadata is required?
- Which local paths, copied files, hashes, and privacy-sensitive provenance
  details are stored or exported?

## Compatibility Policy Hooks

Later storage design must answer how it exposes:

- data-library format version
- feature/capability flags where needed
- migration state and last successful checkpoint
- active writer/import/export/migration blockers
- enough metadata for clients to fail before writes

## Candidate Inputs For Later ADRs

These are historical or plausible inputs only:

- Arrow IPC chunk concepts
- append-only record IDs
- semantic manifest validation ideas
- logical-index sidecars if they fit the v0.2 scan schema model
- dataset archive checksum and manifest concepts

Known redesign pressure:

- `.fricon_workspace.json` as user-facing root identity
- dataset-only SQLite catalog
- dataset-local ownership of favorite/status/tag meaning that belongs to
  measurement or sample/session context
- archive/import/export formats that cannot carry data-library and
  measurement-centered provenance

Do not promote any candidate to accepted storage architecture without an ADR.

## ADRs Needed Later

- data-library layout and migration policy
- measurement table versus generic activity-run table, if that question remains
  relevant
- dataset artifact storage for fixed arrays and variable-length traces
- scan shape and partial-read semantics
- internal stream/subpayload representation, if needed
- external asset/reference hooks for future detector files and images
- event/audit record schema
- export bundle format
- local actor/token storage, if needed
