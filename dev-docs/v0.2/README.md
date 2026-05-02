# Fricon v0.2 Reset

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

The v0.2 product model is:

```text
one Fricon data library
  -> samples and sample sessions
  -> measurement, analysis, simulation, import, and calibration activity records
  -> artifacts: datasets first, then results, reports, logs, attachments,
     device snapshots, and parameter proposals
  -> parameter snapshots, code summaries, notes, tags, and quality state
  -> measurement-centered portable exports for offline analysis
```

For normal users, the first mental model should stay simple:

```text
I selected an active sample/session when it mattered.
I ran a measurement.
It produced datasets.
Fricon helps me inspect, annotate, recover, analyze, and calibrate from them.
```

## Reading Order

Start here for v0.2 planning:

1. `design.md` - canonical v0.2 product and architecture synthesis
2. `product-direction.md` - supporting product stories and user pain
3. `technical-direction.md` - supporting engineering posture and API/runtime
   direction
4. `measurement-system-foundation-redesign.md` - background redesign proposal
   for dataset artifacts and run-like provenance
5. `experiment-run-and-runner-design.md` - older detailed runner proposal;
   reconcile its `ExperimentRun` terminology with the v0.2 `Measurement`
   naming policy before implementation
6. `parameter-management-design.md` - detailed future parameter proposal
7. `future-concepts.md` - lower-confidence or less narrowed ideas

Use current implementation notes only to understand what must be replaced or
adapted:

- `../current-storage-notes.md`
- `../dataset-semantic-architecture-proposal.md`
- `../database-schema-changes.md`
- `../maintenance-checklist.md`

## Relationship To Older Docs

The older project route was dataset-first, Python-led, and local-first. v0.2
keeps the Python-led and local-first parts, but broadens the foundation from a
dataset catalog to a measurement record system:

- data library instead of user-facing workspace
- optional active sample and session identity before measurement history gets
  fragmented
- measurements as the default data-taking work record
- artifacts as outputs, with datasets as the primary table-shaped case
- parameter and code history as first-class provenance
- analysis, simulation, import, and calibration as activity records that
  consume and produce artifacts

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

Current PR decision: prefer a clean v0.2 model over compatibility with old
workspace/dataset-first APIs and storage assumptions. Keep useful
infrastructure, but do not preserve compatibility just to avoid breaking
pre-adoption local test workspaces.

The intended breaking scope is broad at the product boundary, but not a mandate
to rewrite every line of code. v0.2 may intentionally break:

- public workspace and dataset-first Python APIs
- workspace/storage layout and compatibility assumptions
- desktop navigation and IPC DTOs that assume datasets are the only organizing
  object
- archive, import, and export formats that cannot carry data-library,
  measurement, sample/session, and provenance context
- metadata ownership rules that put measurement, sample, parameter, or code
  meaning inside dataset-local metadata

Compatibility for existing local test workspaces should be an explicit
storage/migration ADR decision. Until that ADR exists, do not optimize the v0.2
model around preserving pre-v0.2 workspace behavior.

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

The reset should not make the user model bigger than necessary. Keep most
internal provenance objects invisible unless the user is debugging automation,
recovering an interrupted run, or configuring managed execution.

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
