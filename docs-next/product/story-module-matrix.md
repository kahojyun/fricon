# Story Module Matrix

## Status

Draft traceability baseline.

## Module Legend

| Module | Meaning |
| --- | --- |
| Core Domain | Rust domain/application model under `crates/fricon/src/**` after reset. |
| Storage | SQLite/catalog, payload storage, manifests, migrations. |
| Service API | Local service HTTP/WebSocket/binary endpoints and compatibility negotiation. |
| Python SDK | `crates/fricon-py` user-facing Python APIs and bindings. |
| Desktop UI | Tauri shell plus React frontend. |
| CLI | Setup, diagnostics, service control, developer workflows. |
| Export | Portable bundle writer/reader and offline viewer path. |
| Docs | Public docs, docs-next, and migration guidance. |

## Matrix

| Story | Core Domain | Storage | Service API | Python SDK | Desktop UI | CLI | Export | Docs |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| US-001 Install and launch | X |  | X | X | X | X |  | X |
| US-002 Create/open data library | X | X | X | X | X | X |  | X |
| US-003 Run measurement | X | X | X | X | X |  |  | X |
| US-004 Produce dataset artifacts | X | X | X | X | X |  | X | X |
| US-005 Watch live data | X | X | X |  | X |  |  | X |
| US-006 Recover partial measurement | X | X | X | X | X | X | X | X |
| US-007 Annotate at right level | X | X | X | X | X |  | X | X |
| US-008 Reopen from Python | X | X | X | X |  |  | X | X |
| US-009 Export measurement | X | X | X | X | X | X | X | X |
| US-010 Update safely | X | X | X | X | X | X |  | X |
| US-011 Code provenance | X | X | X | X | X | X | X | X |
| US-012 Sample/session | X | X | X | X | X |  | X | X |
| US-013 Dataset direct open | X | X | X | X | X |  | X | X |
| US-014 Passive setup context | X | X | X | X | X | X | X | X |
| US-015 Scan schema helpers | X | X | X | X | X |  | X | X |
| US-016 Passive procedure context | X | X | X | X | X | X | X | X |
