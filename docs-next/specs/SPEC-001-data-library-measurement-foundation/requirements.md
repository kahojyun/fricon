# SPEC-001: Data Library Measurement Foundation Requirements

## Status

Draft.

## Purpose

Define the first system slice for the v0.2 reset: one local data library,
explicit measurement records, measurement-scoped dataset artifacts, live
inspection, partial recovery, and Python reopen.

## Related Capabilities

CAP-001, CAP-002, CAP-003, CAP-004, CAP-005, CAP-006, CAP-007, CAP-008,
CAP-009, CAP-010, CAP-012, CAP-013, CAP-014, CAP-015, CAP-017.

## Requirements

| ID | Requirement | Acceptance Criteria |
| --- | --- | --- |
| REQ-001 | Create/open a local data library with durable identity. | Library has UUID, display name, format version, and remembered location. |
| REQ-002 | Route mutating operations through the local service. | Python, Desktop, and CLI cannot write without compatibility negotiation. |
| REQ-003 | Create explicit measurements from Python. | API is short, reusable in notebooks, and records title/start time/lifecycle. |
| REQ-004 | Support optional sample/session context. | Context is resolved on measurement creation, may be absent, and is correctable later. |
| REQ-005 | Create dataset artifacts under a measurement. | A measurement can produce multiple datasets; writers share lifecycle by default. |
| REQ-006 | Require explicit scan schema for plotted datasets. | Variables include roles, labels/units where supplied, dependencies, and axis structure. |
| REQ-007 | Preserve append-only facts and partial data. | Crashes leave readable partial facts and visible interrupted/failed state. |
| REQ-008 | Provide nonblocking live inspection events. | Live table/chart consumers never block, slow, or fail acquisition writes. |
| REQ-009 | Record measurement events and notes. | Lifecycle events, notes, markers, corrections, and actor labels form a timeline. |
| REQ-010 | Reopen measurement outputs from Python. | Reads use stable IDs and public APIs, not storage paths. |
| REQ-011 | Fail before writes on incompatibility. | Client/service/library mismatch produces clear diagnostics before mutation. |
| REQ-012 | Support trash/recover and backup checkpoint hooks. | Ordinary cleanup is recoverable; migration/repair has checkpoint strategy. |
| REQ-013 | Record honest code provenance level. | Unmanaged code is labeled unmanaged unless user supplies summary. |
| REQ-014 | Allow optional flexible parameter snapshot. | Measurement can link a snapshot without global profile registry. |

## Non-Goals

- Opening or migrating old v0.1 workspaces by default.
- Data Vault/Grapher compatibility server or direct legacy importer.
- Full export bundle format.
- Remote access.
- Managed runner or task queue.
- Full parameter registry or calibration workflow.
- Device communication.
- AI mutating automation.
