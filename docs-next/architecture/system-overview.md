# Deferred System Overview Questions

## Status

Deferred. `docs-next/architecture/README.md` owns the current pre-architecture
stance.

## Current Product Context

Product docs currently establish these surfaces and constraints:

- Fricon Desktop
- Python SDK
- CLI
- one local data library per normal lab computer
- local-first operation
- compatibility checks before mutating writes
- measurement-first UX with first-class dataset artifact discovery
- nonblocking live inspection

This is not yet a container model, process model, deployment view, or service
contract.

## Deferred System Questions

- Is the local Fricon service a required long-lived process, an on-demand local
  authority, or a packaging/runtime detail?
- How do Desktop, Python SDK, and CLI discover and authenticate with the local
  authority?
- Which responsibilities belong to Desktop shell, browser-capable frontend,
  Python bindings, CLI, local service, and storage adapters?
- Which parts of v0.1 infrastructure should be adapted after the product/domain
  reset?
- Which diagrams are useful once architecture work starts: C4 system context,
  containers, components, or runtime views?

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

Do not promote reuse candidates into accepted architecture without an ADR or
implementation-slice design derived from accepted upstream docs.
