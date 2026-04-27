# Architecture Guidelines

## Status

Canonical architecture guidance for current Fricon implementation work.

## Purpose

This note captures the current architecture rules agents and maintainers should
apply while editing Fricon. Historical discussion and rationale live in
`dev-docs/architecture-ai-notes.md`.

## Product Shape

Fricon is a local-first, single-user scientific experiment measurement
management system. Users interact through the Python API, CLI, and desktop UI.
Internal Rust APIs between crates are not stable user contracts and may be
refactored when it improves architecture.

Do not introduce multi-user, SaaS, account, role, or distributed-system
architecture unless the product direction explicitly changes.

## Feature Slices

Prefer feature-local vertical slices.

A feature may contain:

- core/domain types
- service or workflow orchestration
- thin local adapters for persistence, transport, runtime, or filesystem edges

Do not require every feature to have identical layers. Add boundaries where
workflow complexity crosses real infrastructure edges; keep simple CRUD-heavy
features lightweight.

Preferred dependency direction inside a feature:

```text
composition/app -> services + adapters
services -> core types + feature-defined ports
adapters -> core types + infrastructure
```

## Infrastructure Boundaries

Core and service code should not directly depend on:

- Diesel schema modules
- Diesel query APIs
- transport request/response types
- runtime-specific UI or Tauri types
- filesystem layout details, except through owning storage adapters

Adapter modules own those dependencies. The point of the boundary is local
reasoning and cross-resource orchestration, not future database swapping.

## Persistence

Use repository or persistence-port abstractions when a workflow coordinates
database state with files, sessions, runtime state, or events.

Avoid generic repository wrappers that only mirror ORM CRUD operations. For
simple database-heavy features, a thin feature-local adapter is usually enough.

Keep Diesel-specific code concentrated in persistence/database modules.

## App And Transport

The `app` layer acts as the composition root:

- create infrastructure objects
- instantiate adapters and services
- inject dependencies
- expose stable handles to inbound adapters

gRPC and IPC handlers are inbound adapters. They should parse transport input,
validate transport-level shape, call services, and map errors to transport
statuses. They should not directly query Diesel or own business orchestration.

## Runtime Dependencies

Tokio is acceptable in app-level orchestration, transport glue, background
tasks, and runtime coordination.

Avoid exposing Tokio synchronization types as the primary API of feature core
modules unless stream or concurrency semantics are part of the business
contract.

## Events

Keep these concepts separate:

- feature events, such as dataset created or updated
- app or shell commands, such as show main window
- transport or UI event shapes, such as Tauri/frontend DTOs

Feature code may publish fire-and-forget notifications through a small
publisher trait when needed. App and adapter code should own logging, metrics,
delivery failures, and transport-specific conversion.

## Desktop UI Slices

`fricon-ui` is organized as vertical feature slices across Rust and React.

Rust dependency flow:

```text
desktop_runtime -> tauri_api -> features/<feature>/tauri -> features/<feature>/workflow -> fricon
```

Frontend dependency flow:

```text
app/routes -> features/<feature> -> feature-local api -> shared/lib/tauri.ts -> generated bindings
```

Rules:

- Rust `src/tauri_api.rs` owns global Tauri/Specta binding export and command
  or event aggregation.
- Rust `src/features/<feature>/tauri.rs` files are Tauri adapters only. They
  own commands, events, exported DTOs, and native dialogs.
- Rust `src/features/<feature>/workflow.rs` files own feature orchestration and
  should not depend on Tauri types.
- Pure Rust data shaping helpers stay inside the owning feature slice.
- Frontend features own their own `api/`, `ui/`, `model`, and `hooks` modules.
- Files under `frontend/src/features/**` use relative imports only.
- `frontend/src/app/**` and `frontend/src/routes/**` import features only
  through public barrels such as `@/features/<feature>`.
- `frontend/src/shared/lib/tauri.ts` stays generic; feature-specific
  normalization and query/event wiring belong in each feature's `api/` folder.

## Compatibility Boundaries

Workspace compatibility and IPC compatibility are separate concerns.

- Workspace compatibility is controlled by `.fricon_workspace.json`,
  `WORKSPACE_VERSION`, and migration steps in `crates/fricon/src/workspace.rs`.
- IPC compatibility is controlled by the protocol handshake exposed by the
  `Version` RPC and `IPC_PROTOCOL_VERSION`.
- UI probes should only verify that a workspace is recognizable and IPC is
  reachable; they should not replace workspace migration.

For actionable maintenance steps, use `dev-docs/maintenance-checklist.md`.

## Anti-Decay Checks

Healthy changes usually have these properties:

- Most code changes stay within one feature slice.
- Services read like business steps instead of persistence scripts.
- Diesel-specific code stays in adapter/persistence modules.
- Feature APIs use domain/application terms instead of infrastructure terms.

Watch for these drift signals:

- services importing Diesel or schema modules directly
- feature boundaries becoming inconsistent across modules
- shared helpers appearing before behavior is stable across multiple features
- repository traits becoming generic wrappers with no domain meaning
- events mixing feature semantics with UI shell control
