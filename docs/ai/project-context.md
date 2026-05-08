# Project Context For Agents

## Status

Draft pending product-analysis revalidation.

## Start Here

For v0.2+ work, read:

1. `docs/README.md`
2. `docs/decisions/ADR-001-v02-clean-reset-boundary.md`
3. `docs/product/vision.md`
4. `docs/product/personas.md`
5. `docs/product/product-analysis-progress.md`
6. `docs/product/python-sdk-ux.md` for the Python SDK usage guideline
7. `docs/domain/README.md`
8. `docs/architecture/README.md`
9. `docs/architecture/compatibility-policy.md`

`docs/specs/` and `docs/implementation-plans/` are sentinels only
right now. Recreate or read downstream artifacts only when implementation
planning is the task and the relevant upstream baseline is accepted or
explicitly marked with open interview questions.

## Product Direction

Fricon is currently being revalidated as a local lab data library for
scientific measurement work. Treat `product/vision.md` and
`product/personas.md` as the strongest current product inputs. Treat
capabilities, stories, epics, and future backlog docs as draft derived material
until `product/product-analysis-progress.md` says they have been revalidated.

Product planning uses MVP, post-MVP priority, and ADR-gated labels. Do not
interpret post-MVP priorities as semantic-version labels; compatible additions
may remain on the same compatible release line.

Post-MVP priority order and rationale are draft backlog material until the MVP
product baseline is revalidated.

Primary user model:

```text
I ran a measurement.
It produced datasets.
Fricon helps me inspect, annotate, recover, reopen, and export them.
```

## Hard Boundaries

- Do not preserve prior workspace/dataset compatibility by default.
- Do not make datasets own measurement, sample, parameter, code, or lifecycle
  meaning.
- Do not make Desktop the durable data backend.
- Do not bypass local runtime compatibility checks for mutating Python APIs.
- Do not introduce SaaS, accounts, teams, roles, or distributed database
  behavior for the MVP.
- Do not implement post-MVP runner, device, calibration, or AI mutation systems
  before their ADRs/specs exist.

## Good Defaults

- Prefer measurement-scoped dataset writers in public examples.
- Use MVP/post-MVP/ADR-gated for product priority. Do not use `v0.3` or `v0.4`
  as shorthand for feature horizons.
- Treat high-impact Python SDK ergonomics as product requirements, not only
  implementation details.
- Keep product-level SDK docs focused on usage guidelines and non-binding
  sketches; exact syntax, capture mechanics, and object models belong in later
  ADRs/specs.
- For common workflows, provide appropriate simplification, but do not freeze
  the exact simplification shape before real script/notebook feedback.
- Keep sample/session context optional and correctable.
- Do not turn sample target binding into a heavy physical-component ontology
  unless a later product decision requires it.
- For post-MVP parameter, calibration, run-manifest, sample-visualizer, and
  setup/device reconciliation details, read `product/future-concepts.md` and
  `product/glossary.md` as draft backlog and terminology context.
- Make live views noncritical consumers.
- Keep code provenance honest: unmanaged means unmanaged.
- Use events for lifecycle, notes, corrections, and audit history.
- Use ADRs for storage, local runtime/API, export format, and compatibility
  decisions before durable implementation.
