# Module Boundaries

## Status

Draft.

## Target Feature Slices

| Slice | Owns | Does Not Own |
| --- | --- | --- |
| `data_library` | Library identity, opening, compatibility, checkpoints, backup/restore. | Measurement lifecycle or dataset payload semantics. |
| `measurement` | Measurement records, lifecycle, produced artifact links, notes/events, sample/session links. | Dataset facts, chart projection, global parameter profile mutation. |
| `dataset_artifact` | Artifact identity, append sessions, Arrow-compatible facts, variable roles, scan schema, semantic reads. | Measurement intent, sample identity, code provenance. |
| `sample_context` | Samples, sessions, active context, correction history. | Measurement execution or dataset facts. |
| `provenance` | Parameter summaries, run-bound configuration snapshots, code provenance summaries, passive setup/procedure summaries, actor/event records. | Effective parameter snapshots, profile management, or managed runner internals until ADR-gated. |
| `export` | Measurement-centered bundle writing/reading, manifest, checksums, privacy preview. | Source data-library mutation after export. |
| `service_api` | HTTP/control API, live events, binary payload endpoints, capability negotiation. | Domain decisions hidden in transport DTOs. |
| `desktop_console` | Measurement console and live/history interaction model. | Durable storage or business orchestration. |
| `python_sdk` | User-facing Python ergonomics and diagnostics. | Bypassing service compatibility for writes. |

## Dependency Direction

```text
composition/app
  -> services + adapters
services
  -> domain types + feature-defined ports
adapters
  -> domain types + database/filesystem/transport/runtime
```

## Boundary Rules

- Core/service code must not import Diesel schema modules directly.
- Transport handlers parse transport shape, call services, and map errors.
- Feature events are separate from UI shell commands and transport event DTOs.
- Frontend feature code should remain browser-capable where practical.
- Tauri-specific file dialogs, launch behavior, updater, diagnostics, and
  shell integration stay behind shell adapters.
- Add traits only for real boundaries or multiple plausible implementations.

## Current-to-Target Mapping

| Current Area | v0.2 Treatment |
| --- | --- |
| `crates/fricon/src/workspace.rs` | Replace user-facing workspace with data library. Reuse only low-level patterns that fit. |
| `crates/fricon/src/dataset/**` | Split into dataset artifact facts, semantics, live append, and artifact boundary. |
| `crates/fricon/src/transport/**` | Treat gRPC/IPC as migration background; design one service API contract by ADR. |
| `crates/fricon-py/**` | Redesign public API around `fricon.library()` and measurement-scoped writes. |
| `crates/fricon-ui/frontend/src/features/datasets/**` | Move first-screen UX toward measurement console; keep chart code where model-compatible. |
| `crates/fricon-ui/src/features/**` | Keep vertical slice discipline; replace dataset-first Tauri command assumptions. |
