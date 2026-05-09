# Domain Notes

## Status

Deferred product-analysis placeholder.

## Purpose

`docs/domain/` is intentionally minimal while Fricon is still returning to the
product layer. The migrated draft domain files mixed product scope, future
backlog, domain semantics, and architecture-adjacent constraints. Keeping them
active made the project look more settled than it is.

Use this directory as a reminder that a domain baseline is still needed, not as
an accepted conceptual model.

## Current Source Of Truth

During the current phase:

- `docs/product/vision.md` owns the current high-confidence product thesis,
  initial adoption goal, user promise, and high-level horizons.
- `docs/product/personas.md` owns the current high-confidence product-role
  model.
- `docs/product/product-analysis-progress.md` owns product-analysis progress,
  document confidence, open questions, and the next analysis sequence.
- `docs/product/story-map.md` and `docs/product/capability-map.md` own the
  current rederived draft story and capability baseline.
- `docs/product/glossary.md` is draft terminology context pending product
  analysis.
- `docs/product/future-concepts.md` owns strategic follow-on backlog context
  pending a stable initial adoption baseline.
- `docs/architecture/README.md` owns only accepted architecture constraints and
  deferred architecture questions.

If a topic is about what users need, product priority, initial adoption versus
strategic follow-on scope, or acceptable user-facing promises, keep it in
`docs/product/`.

If a topic is about transport, storage, process boundaries, API shape, module
names, export format, or migration mechanics, keep it out of `docs/domain/`
until architecture or ADR work starts.

## Domain Rebuild Trigger

Recreate detailed domain docs only after the relevant product inputs are
accepted or explicitly marked with open questions. During the current analysis
phase, use `docs/product/product-analysis-progress.md` to decide whether a
product input is ready to support domain work.

The first rebuilt domain baseline should be lean and should answer only:

- Which concepts exist?
- Which concept owns which meaning?
- Which concepts must stay separate?
- What lifecycle states matter independent of implementation?
- What invariants prevent product or architecture drift?

Expected future files, when the phase starts:

- `conceptual-model.md`
- `context-map.md`
- `lifecycle-model.md`
- `invariants.md`

Do not restore the old migrated draft files as active sources of truth. Mine
git history only when explicitly asked for historical reference, then rederive
domain content from accepted greenfield product documents.
