# Story Implementation Routing Placeholder

## Status

Deferred.

## Purpose

Avoid creating a large story-by-module table while the project is still in
product and initial domain analysis. This file records only likely downstream
implementation concerns so future planning can derive traceability after
product, domain, architecture, and required ADR inputs are accepted.

Do not treat this file as an accepted Rust module map, frontend feature map,
storage/API design, or task/file traceability matrix.

## Concern Legend

- Product/domain model: accepted concepts, lifecycles, invariants, and context
  ownership.
- Local runtime/API: future architecture for mutation gates, compatibility,
  events, and client access.
- Storage/export: future architecture for catalog, payloads, migrations,
  checkpoints, and portable bundles.
- Python SDK: user-facing measurement recording, reopen, export, diagnostics,
  and notebook-friendly context.
- Desktop: local measurement console, live/history views, setup, diagnostics,
  and dataset artifact discovery.
- CLI: setup, diagnostics, and local maintenance flows.
- Docs: `docs-next/`, future public docs, and migration guidance.

## Epic Routing

EPIC-001 Local setup and data-library adoption:
Product/domain model, Local runtime/API, Storage/export, Python SDK, Desktop,
CLI, Docs.

EPIC-002 New measurement logging replacement:
Product/domain model, Local runtime/API, Storage/export, Python SDK, Desktop,
Docs.

EPIC-003 Measurement console and inspection:
Product/domain model, Local runtime/API, Python SDK for reopen snippets,
Desktop, Docs.

EPIC-004 Recovery, annotation, reopen, and export:
Product/domain model, Local runtime/API, Storage/export, Python SDK, Desktop,
Docs.

EPIC-005 Migration ergonomics and future lab scaling:
Python SDK, Desktop, Docs, then later Local runtime/API, CLI, and
Storage/export as migration helpers mature.

## Rule

When a story becomes implementation-ready, add precise traceability downstream
instead of expanding this file into a matrix.
