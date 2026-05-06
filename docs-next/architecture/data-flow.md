# Data Flow

## Status

Draft.

## Measurement Write Flow

```text
Python script
  -> Python SDK creates Measurement
  -> local service checks client/service/data-library compatibility
  -> service records Measurement and optional context/provenance
  -> SDK declares DatasetArtifact writer with scan schema
  -> SDK appends Arrow-compatible payload chunks
  -> service persists facts and emits live events
  -> Desktop subscribes and renders nonblocking previews
  -> SDK finishes Measurement
  -> service finalizes datasets and lifecycle events
```

Dataset artifacts remain directly searchable and openable after this flow. The
measurement is the primary navigation context, not the only durable handle.

## Live Read Flow

```text
Desktop live console
  -> subscribe to service events
  -> receive measurement and dataset summaries
  -> request paged/downsampled semantic reads
  -> render table/chart views
```

Rules:

- Live readers use explicit append positions or event sequence IDs.
- Preview updates may be coalesced or dropped under load.
- Committed data is never dropped.
- Rendering errors do not fail acquisition writes.

## Python Reopen Flow

```text
Python SDK
  -> connect/discover local service
  -> get Measurement by stable ID/search result
  -> list produced DatasetArtifacts
  -> request semantic table, dependent-with-axes, or grid-like view
```

Semantic reads must distinguish complete regular grids from partial grids,
irregular/adaptive points, repeated points, and trace data where the scan schema
declares those modes.

## Export Flow

```text
User selects Measurement
  -> service validates idle/read-safe state
  -> export writer gathers selected metadata and artifacts
  -> privacy preview handles sensitive provenance
  -> bundle manifest/checksums/common files are written
  -> Python or Desktop opens bundle read-only without importing
```

## Migration/Update Flow

```text
update or data-library format change
  -> client asks service for safe_to_update / safe_to_migrate
  -> service reports blockers
  -> if idle: create checkpoint where practical
  -> run migration
  -> report success or recovery guidance
```
