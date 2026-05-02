# Fricon v0.2 Development Direction

## Status

Canonical planning entry point for the proposed v0.2 reset.

This directory describes the intended v0.2 product and architecture direction.
It is not current behavior. Do not document v0.2 behavior in public user docs
or implement durable storage/API contracts from these notes until the relevant
ADR, schema, IPC, Python SDK, desktop UI, migration, and release decisions have
landed.

## v0.2 Thesis

Fricon v0.2 should stop being framed primarily as a workspace dataset browser.
It should become a local lab data library and automation foundation for
scientific measurement work.

The main product goal is to prevent the common failure mode where experimental
history, sample context, copied code folders, JSON parameters, calibration
scripts, and measured data quietly diverge until they are hard to trust or
maintain.

The v0.2 direction is:

```text
one Fricon data library
  -> samples
  -> sample sessions / cooldowns
  -> experiment runs
  -> dataset artifacts
  -> analysis results
  -> parameter snapshots and proposals
  -> calibration workflow history
```

## Reading Order

Start here for v0.2 planning:

1. `product-direction.md`
2. `technical-direction.md`
3. `../measurement-system-foundation-redesign.md`
4. `../experiment-run-and-runner-design.md`
5. `../parameter-management-design.md`
6. `../future-concepts.md`

Use current implementation notes only to understand what must be replaced or
adapted:

- `../current-storage-notes.md`
- `../dataset-semantic-architecture-proposal.md`
- `../database-schema-changes.md`
- `../maintenance-checklist.md`

## Relationship To Older Docs

The older project route was dataset-first, Python-led, and local-first. v0.2
keeps the Python-led and local-first parts, but broadens the foundation from
datasets to the full measurement record:

- data library instead of user-facing workspace
- sample and session identity before experiment history gets fragmented
- experiment runs as the default measurement work record
- dataset artifacts as outputs, not the whole experiment record
- parameter and code history as first-class provenance
- calibration and analysis as future workflows over explicit records

Older dataset semantics work remains useful, especially append-only facts,
record IDs, manifests, and resolved interpretation. But v0.2 implementation
should reconcile that work with the broader data-library, sample/session, and
provenance model before committing storage or API contracts.

## Development Posture

Fricon is still pre-adoption. v0.2 may make breaking changes when they produce a
cleaner long-term foundation.

Prefer rewriting or replacing current domain boundaries when the existing shape
would preserve the wrong user model. Preserve reusable infrastructure where it
does not lock the product into workspace/dataset-first assumptions.

Good candidates to keep or adapt:

- Rust, Python binding, and frontend build infrastructure
- Arrow payload IO concepts
- append-only dataset facts and record IDs
- chart rendering work after DTO/model changes
- Tauri packaging and web frontend stack
- testing, release, and documentation infrastructure

Good candidates to redesign:

- public workspace mental model
- dataset catalog schema and metadata ownership
- Python dataset creation API
- desktop dataset-first navigation
- IPC/protobuf contracts
- archive/import/export format
- run, sample, parameter, code, and provenance boundaries

## v0.2 Non-Goals

Do not turn v0.2 into a complex lab administration system.

Non-goals:

- hosted SaaS
- full multi-user roles and permissions
- account/team administration
- distributed database semantics
- broad device-driver framework
- generic workflow DAG engine in the first implementation
- automatic notebook state capture
- mandatory parameter schema design before exploratory measurement

Remote access, authentication boundaries, and actor/audit records should be
planned early, but the first product should remain local-first and single-owner.
