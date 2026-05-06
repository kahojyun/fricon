# Project Context For Agents

## Status

Accepted for v0.2+ planning.

## Start Here

For v0.2+ work, read:

1. `docs-next/README.md`
2. `docs-next/decisions/ADR-001-v02-clean-reset-boundary.md`
3. `docs-next/product/vision.md`
4. `docs-next/product/python-sdk-ux.md` for the Python SDK usage guideline
5. `docs-next/domain/conceptual-model.md`
6. `docs-next/architecture/module-boundaries.md`
7. the relevant spec under `docs-next/specs/`

Use `dev-docs/` for current implementation details and historical rationale,
not as the v0.2+ design source of truth.

## Product Direction

Fricon is a local lab data library for scientific measurement work.
Its long-term motivation is unified parameter management, measurement-code
management, SDK runner capture, dataset recording, and provenance in one
local-first product experience.

Product planning uses MVP, post-MVP priority, and ADR-gated labels. Do not
interpret post-MVP priorities as semantic-version labels; compatible additions
may remain on the same compatible release line.

Post-MVP priority order:

1. Parameter system.
2. Managed run.
3. Reviewable automation workflow.

Primary user model:

```text
I ran a measurement.
It produced datasets.
Fricon helps me inspect, annotate, recover, reopen, and export them.
```

## Hard Boundaries

- Do not preserve v0.1 compatibility by default.
- Do not make datasets own measurement, sample, parameter, code, or lifecycle
  meaning.
- Do not make Desktop the durable data backend.
- Do not bypass service compatibility checks for mutating Python APIs.
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
- Keep sample/session context optional and correctable.
- Make live views noncritical consumers.
- Keep code provenance honest: unmanaged means unmanaged.
- Use events for lifecycle, notes, corrections, and audit history.
- Use ADRs for storage, service API, export format, and compatibility
  decisions before durable implementation.
