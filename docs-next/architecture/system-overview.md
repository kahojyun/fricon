# System Overview

## Status

Draft v0.2+ architecture baseline.

## C4: System Context

```text
Experimentalist / Analyst / Lab Maintainer
  -> Fricon Desktop
  -> fricon Python SDK
  -> fricon CLI

Fricon Desktop / Python SDK / CLI
  -> local Fricon service
  -> one local Fricon data library
```

## Target Containers

| Container | Responsibility | Notes |
| --- | --- | --- |
| Fricon Desktop | Local GUI, measurement console, live/history views, setup, diagnostics, export/offline viewer shell. | Tauri shell plus browser-capable React app. |
| Python SDK | Measurement creation, dataset writes, reopen, export reads, service discovery diagnostics. | First-class acquisition surface. |
| CLI | Setup, diagnostics, service control, developer workflows. | Not the broad ordinary-user workflow surface. |
| Local Service | Compatibility gate, data-library coordination, catalog, writes, live events, export, migration. | Authoritative data backend. |
| Data Library Storage | SQLite/catalog, artifact payloads, manifests, events, checkpoints. | Local only in v0.2. |

## Key Architecture Decisions To Preserve

- The local service owns coordinated data-library mutations.
- Desktop may launch or supervise the service but does not own durable data.
- Python, Desktop, and CLI converge on one service API compatibility boundary.
- Dataset payload transfer uses binary Arrow-compatible chunks where practical,
  not row-by-row JSON.
- Live views are noncritical consumers and must not block acquisition writes.
- The reset is a domain-model reset, not a mandatory rewrite of every reusable
  infrastructure component.

## Current Implementation Relationship

Current v0.1 modules are implementation source material, not v0.2 boundaries.

Reusable or adaptable:

- Rust/Python/frontend build infrastructure
- Arrow payload IO concepts
- append-only dataset writer/read path concepts
- record IDs and semantic manifest ideas
- chart rendering after DTO/model changes
- Tauri packaging and frontend stack
- testing and release infrastructure

High redesign risk:

- workspace mental model
- dataset catalog schema and metadata ownership
- Python dataset creation API
- desktop dataset-first navigation
- IPC/protobuf public contract assumptions
- archive/import/export formats
- setup/update/service compatibility assumptions
