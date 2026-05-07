# Story Module Traceability

## Status

Accepted traceability guide.

## Purpose

Avoid a large story-by-module table. Use this file to route future work to the
right code areas at epic level. Precise task/file traceability belongs in a
derived implementation artifact after upstream scope is accepted.

## Module Legend

- Core Domain: Rust domain/application model under `crates/fricon/src/**`.
- Storage: catalog, payload storage, manifests, migrations.
- Service API: local service API, events, compatibility negotiation.
- Python SDK: `crates/fricon-py`.
- Desktop UI: Tauri shell and React frontend.
- CLI: setup, diagnostics, service control.
- Export: portable bundle writer/reader and offline viewer path.
- Docs: `docs-next/`, future public docs, and migration guidance.

## Epic Routing

EPIC-001 Local setup and data-library adoption:
Core Domain, Storage, Service API, Python SDK, Desktop UI, CLI, Docs.

EPIC-002 New measurement logging replacement:
Core Domain, Storage, Service API, Python SDK, Desktop UI, Docs.

EPIC-003 Measurement console and inspection:
Core Domain, Service API, Desktop UI, Python SDK for reopen snippets, Docs.

EPIC-004 Recovery, annotation, reopen, and export:
Core Domain, Storage, Service API, Python SDK, Desktop UI, Export, Docs.

EPIC-005 Migration ergonomics and future lab scaling:
Python SDK, Desktop UI, Docs, then later Service API, CLI, and Export as
migration helpers mature.

## Rule

When a story becomes implementation-ready, add precise traceability downstream
instead of expanding this file into a matrix.
