# Contributing to Fricon

Thanks for helping improve Fricon. This document gives a concise, practical guide to get started, build and contribute.

## Quick start

Prerequisites (install the package manager or tooling appropriate for your OS):

- Rust: use the workspace toolchain (rust-toolchain.toml).
- Python: 3.10+
- Node.js: 24+ (frontend development)
- Tools: uv (Python package manager), pnpm (Node package manager), diesel_cli (optional for DB migrations)

Essential steps:

1. Clone the repo:

    git clone https://github.com/kahojyun/fricon.git
    cd fricon

2. Run the development setup (creates .dev, .env, and ensures the dev workspace exists via `fricon init`):

    python3 scripts/setup-dev.py

3. Install language/tool-specific dependencies as needed (see sections below).

### Working with git worktrees

For a new worktree, bootstrap only the checkout-local state first:

    git worktree add ../fricon-my-task -b my-task
    cd ../fricon-my-task
    python3 scripts/bootstrap-checkout.py

Then install and run only the task-specific tooling you need. `.venv/` and
`node_modules/` remain worktree-local; Rust build output is shared through
Cargo configuration. The checkout bootstrap does not create the workspace; use
`python3 scripts/setup-dev.py` when you want the dev workspace to be created
through `fricon init`.

## Install / Tooling notes

- macOS: brew install protobuf pkg-config
- Debian/Ubuntu: apt install build-essential pkg-config protobuf-compiler
- Install diesel_cli for migrations (optional):

    cargo install diesel_cli --no-default-features --features sqlite

- Install Python tools with uv when available:

    uv sync --all-groups

For a fresh worktree, only run this when the current task needs Python
dependencies.

- Frontend (pnpm workspace)

The repository uses a pnpm workspace. Run pnpm commands from the project root so workspace packages (including the frontend) are handled automatically:

    pnpm install          # run at project root to install all workspace deps

For a fresh worktree, only run this when the current task needs frontend
dependencies.

### Tauri v2 dependencies for Linux

To build and run the Tauri v2 frontend on Linux, install the following system dependencies:

    sudo apt update
    sudo apt install libwebkit2gtk-4.1-dev \
        build-essential \
        curl \
        wget \
        file \
        libxdo-dev \
        libssl-dev \
        libayatana-appindicator3-dev \
        librsvg2-dev

See [Tauri docs](https://tauri.app/start/prerequisites/) for details.

## Build & test

Rust (workspace):

    cargo check
    cargo build
    cargo test

Bootstrap a fresh worktree first:

    python3 scripts/bootstrap-checkout.py

Python:

    uv run maturin develop   # build extension for development
    uv run pytest
    uv run basedpyright
    uv run stubtest fricon._core

Frontend (UI):

Use `pnpm run check` as the default frontend validation command. It runs the
main static checks, including the dependency-cruiser boundary check.

Common frontend commands:

    pnpm run check
    pnpm run format:check
    pnpm run format
    pnpm run lint:fix
    pnpm run depcruise:frontend
    pnpm tauri dev

## Workspace setup

The dev workspace lives at `.dev/ws`. `python3 scripts/setup-dev.py` ensures it
exists by calling `fricon init` through the repo CLI entrypoint. The lighter
`python3 scripts/bootstrap-checkout.py` command only aligns checkout-local
configuration; it does not create the workspace.

## Database migrations

The project uses SQLite + Diesel. Typical commands (from crates/fricon):

    diesel migration generate <name>
    diesel migration run
    diesel migration redo

(Ensure `DATABASE_URL` is set in `.env` or created by setup script. It should
point at `.dev/ws/fricon.sqlite3` for the current checkout.)
(New worktrees should use `python3 scripts/bootstrap-checkout.py`.)

## Code style and linting

Rust:

    cargo +nightly fmt
    cargo clippy --all-targets --all-features

Python:

    uv run ruff format
    uv run ruff check
    uv run basedpyright

Frontend:

For frontend changes, start with:

    pnpm run check

Then use narrower commands when needed:

    pnpm run format:check
    pnpm run depcruise:frontend
    git diff --exit-code crates/fricon-ui/frontend/src/routeTree.gen.ts

Docs:

    uv run --group docs mkdocs build -s -v

## Contribution workflow

- Branch from `main`: feature/bugfix branches named clearly.
- Use conventional commits: `type(scope): description` (feat, fix, docs, style, refactor, test, chore).
- For release notes and version bumps, prefer `knope document-change` and commit the generated file under `.changeset/`.
- Add tests for new features when possible.
- Run relevant linters and tests before opening a PR.
- For frontend work, start with `pnpm run check`, then run narrower commands only if you need to investigate a specific failure.

PR checklist (recommended):

- Follow code style and pass linters
- Include tests or a clear rationale if none
- Update docs if behavior changes

## Release workflow

- Releases are managed by `knope`, not `release-plz`.
- The repository ships one unified release version and one canonical Git tag: `v<version>`.
- A push to `main` refreshes a rolling release preview PR from the `release` branch.
- Merging that PR creates the GitHub release and tag; publishing remains PyPI-only.
- Release notes and version bumps can come from either Knope change files or conventional commits. Use change files when you want explicit release notes or a release rule that should not rely on commit parsing.

## Reporting issues & getting help

- Open issues with clear reproduction steps and environment details.
- Use repository Discussions for general questions.
- See the project docs: https://kahojyun.github.io/fricon/

## Minimal troubleshooting

- Missing protoc: install `protobuf` (brew/apt)
- Python import/build errors: re-run `uv run maturin develop` after Rust changes
