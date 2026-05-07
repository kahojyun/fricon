# fricon Repository Agent Rules

## Scope

- `docs-next/` is the single active documentation directory for the v0.2+
  clean reset.
- The pre-v0.2 implementation has been removed from this branch. Use the
  `archive/v0.1` branch only as historical reference when explicitly needed.
- `README.md`, `CONTRIBUTING.md`, `mkdocs.yml`, and this file are lightweight
  repository scaffolding for the planning workspace.

## Repo-Wide Rules

- Current phase: product analysis and early domain analysis for the v0.2+
  reset. Keep changes focused on `docs-next/` unless the user explicitly asks
  for repository-structure cleanup.
- Do not infer v0.2 architecture from the removed v0.1 crates, local skills,
  scripts, package configs, or module boundaries. Recreate implementation
  guidance later from accepted product, domain, architecture, and ADR inputs.
- Do not install dependencies, regenerate artifacts, or run Rust/Python/Node
  implementation checks for docs-only work.
- Use `docs-next/README.md` as the documentation entry point. For agent routing
  and documentation update policy, use `docs-next/ai/`.
- For future data-library format, local runtime/API, IPC/protocol, export, or
  compatibility decisions, confirm accepted product/domain inputs first and
  update `docs-next/architecture/compatibility-policy.md` or add an ADR before
  durable implementation.
- Use `prek` when running the local hook config.
