# Contributing to Fricon

## Current Phase

This branch is a v0.2 clean-reset planning workspace. The pre-v0.2
implementation has been removed here and remains available on the
`archive/v0.1` branch for historical reference.

The active work surface is `docs-next/`. Product analysis and early domain
analysis should not depend on old crates, setup scripts, package locks, local
skills, release automation, or generated artifacts.

## Quick Start

```bash
git clone https://github.com/kahojyun/fricon.git
cd fricon
```

For `docs-next/` editing, no Rust, Python package, Node, Tauri, Diesel, or
workspace setup is required.

## Documentation

Read [docs-next/README.md](docs-next/README.md) first. It defines the current
reading order and ownership model for product, domain, architecture, ADR,
research, user-documentation planning, and agent guidance.

Optional local docs build:

```bash
uv run --group docs mkdocs build --strict
```

If docs dependencies are not installed, use `uv sync --group docs` only when a
local rendered-docs check is actually needed.

## Local Hygiene

The repository keeps a minimal `prek`-compatible hook config for text and
config hygiene only:

```bash
prek run --all-files
```

Do not add implementation gates, dependency-update automation, generated-file
checks, or release automation back to this branch until implementation policy is
re-established from accepted v0.2 docs.

## Reintroducing Code

When implementation restarts, derive new setup, test, package, release, and
agent-steering files from accepted `docs-next/` product/domain/architecture
inputs. Do not restore v0.1 scaffolding wholesale from `archive/v0.1` unless a
current ADR explicitly accepts that part of the old implementation.
