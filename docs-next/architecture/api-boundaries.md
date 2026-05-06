# API Boundaries

## Status

Draft.

## Public Surfaces

| Surface | Purpose | v0.2 Stability Posture |
| --- | --- | --- |
| Fricon Desktop | Primary GUI for local measurement console, browsing, diagnostics, export. | Product surface, but UI can change during v0.x. |
| Python SDK | Primary acquisition and analysis API. | Ergonomic but not strict pre-1.0 API stability. Fail before writes on incompatibility. |
| CLI | Setup, diagnostics, service control, developer workflows. | Narrow public surface. |
| Export Bundle Reader | Read-only offline analysis. | Format versioned from first implementation. |
| Local Service API | Shared client/service contract. | Internal Fricon contract first; no third-party long-term stability promise in v0.2. |

## Service Contract Direction

Prefer one browser-capable service contract for Desktop, CLI, and Python SDK:

- JSON HTTP for metadata, control, compatibility negotiation, diagnostics, and
  ordinary mutations
- WebSocket or SSE for live measurement, dataset, and service status events
- binary Arrow IPC or Arrow-compatible chunk endpoints for dataset writes and
  reads
- explicit write sessions with create, append, finish, and abort operations
- server-side summaries, paging, and downsampling for UI reads
- semantic read shapes for tables, dependent-with-axes views, regular grids,
  partial grids, irregular/adaptive points, repeated points, and trace data

The current gRPC path is implementation background, not the preferred durable
v0.2 public Python contract unless an ADR proves otherwise.

## Python API Direction

Draft shape:

```python
lib = fricon.library()
lib.use_context(sample="qpu-017", session="cooldown-2026-05")

with lib.measurement("rabi q3") as meas:
    rabi = meas.dataset(
        "rabi",
        scan={"independent": "amp", "dependent": "signal"},
    )
    rabi.write(amp=0.1, signal=0.25)
```

Rules:

- `fricon.library()` returns a reusable handle.
- Measurement creation is explicit but short.
- Sample/session context is a resolved default, not hidden provenance.
- Low-level datasets may exist, but normal examples are measurement-scoped.
- Dataset artifacts remain searchable/openable first-class records.
- Scan schema authoring should have Python-native scan plans or helpers for
  common 1D/2D/N-D scan and trace shapes, plus raw schema APIs for advanced
  cases.
- Reads use stable IDs and semantic APIs, not storage paths.

## Compatibility Negotiation

Before writes, clients and service negotiate:

- service API/protocol version
- data-library format version
- client capabilities
- write capability
- measurement creation capability
- export/migration capability where relevant

Read-only operations may be allowed during limited mismatches if explicitly
safe.
