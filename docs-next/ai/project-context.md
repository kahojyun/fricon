# Project Context For Agents

## Status

Accepted for v0.2+ planning.

## Start Here

For v0.2+ work, read:

1. `docs-next/README.md`
2. `docs-next/decisions/ADR-001-v02-clean-reset-boundary.md`
3. `docs-next/product/vision.md`
4. `docs-next/domain/conceptual-model.md`
5. `docs-next/architecture/module-boundaries.md`
6. the relevant spec under `docs-next/specs/`

Use `dev-docs/` for current implementation details and historical rationale,
not as the v0.2+ design source of truth.

## Product Direction

Fricon v0.2+ is a local lab data library for scientific measurement work.

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
  behavior for v0.2.
- Do not implement future runner, device, calibration, or AI mutation systems
  before their ADRs/specs exist.

## Good Defaults

- Prefer measurement-scoped dataset writers in public examples.
- Keep sample/session context optional and correctable.
- Make live views noncritical consumers.
- Keep code provenance honest: unmanaged means unmanaged.
- Use events for lifecycle, notes, corrections, and audit history.
- Use ADRs for storage, service API, export format, and compatibility
  decisions before durable implementation.
