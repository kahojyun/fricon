# fricon Repository Agent Rules

## Scope

- `docs/` is the single active documentation directory for the v0.2+
  clean reset.
- Use the `archive/v0.1` branch only when historical code reference is
  explicitly needed.
- `README.md` and this file are lightweight repository scaffolding for the
  planning workspace.

## Repo-Wide Rules

- Current phase: product analysis and early domain analysis for the v0.2+
  reset. Keep changes focused on `docs/` unless the user explicitly asks
  for repository-structure cleanup.
- Do not infer v0.2 architecture from prior implementation scaffolding, local
  skills, scripts, package configs, or module boundaries. Recreate
  implementation guidance later from accepted product, domain, architecture,
  and ADR inputs.
- Do not install dependencies, regenerate artifacts, or run Rust/Python/Node
  implementation checks for docs-only work.
- Use `docs/README.md` as the documentation entry point. For agent routing
  and documentation update policy, use `docs/ai/`.
- For future data-library format, local runtime/API, IPC/protocol, export, or
  compatibility decisions, confirm accepted product/domain inputs first and
  update `docs/architecture/compatibility-policy.md` or add an ADR before
  durable implementation.
- Use `prek` when running the local hook config.
