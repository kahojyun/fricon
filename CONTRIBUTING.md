# Contributing to Fricon

Thanks for helping improve Fricon. This document gives a concise, practical guide to get started, build and contribute.

## Quick start

Prerequisites (install the package manager or tooling appropriate for your OS):

- Rust: use the workspace toolchain (rust-toolchain.toml).
- Python: 3.10+
- Node.js: 22+ (frontend development)
- Tools: uv (Python package manager), pnpm (Node package manager), diesel_cli (optional for DB migrations)

Essential steps:

1. Clone the repo:

    git clone https://github.com/kahojyun/fricon.git
    cd fricon

2. Run the development setup (creates .dev, .env, runs migrations):

    python3 scripts/setup-dev.py

3. Install language/tool-specific dependencies as needed (see sections below).

## Install / Tooling notes

- macOS: brew install protobuf pkg-config
- Debian/Ubuntu: apt install build-essential pkg-config protobuf-compiler
- Install diesel_cli for migrations (optional):

    cargo install diesel_cli --no-default-features --features sqlite

- Install Python tools with uv when available:

    uv sync --all-groups

- Frontend (pnpm workspace)

The repository uses a pnpm workspace. Run pnpm commands from the project root so workspace packages (including the frontend) are handled automatically:

    pnpm install          # run at project root to install all workspace deps

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

Python:

    uv run maturin develop   # build extension for development
    uv run pytest

Frontend (UI):

    pnpm run check
    pnpm run format
    pnpm run lint:fix
    pnpm tauri dev

## Database migrations

The project uses SQLite + Diesel. Typical commands (from crates/fricon):

    diesel migration generate <name>
    diesel migration run
    diesel migration redo

(Ensure `DATABASE_URL` is set in `.env` or created by setup script.)

## Code style and linting

Rust:

    cargo +nightly fmt
    cargo clippy --all-targets --all-features

Python:

    uv run ruff format
    uv run ruff check
    uv run basedpyright

Frontend:

    cd crates/fricon-ui/frontend
    pnpm run lint

## Contribution workflow

- Branch from `main`: feature/bugfix branches named clearly.
- Use conventional commits: `type(scope): description` (feat, fix, docs, style, refactor, test, chore).
- Add tests for new features when possible.
- Run relevant linters and tests before opening a PR.

PR checklist (recommended):

- Follow code style and pass linters
- Include tests or a clear rationale if none
- Update docs if behavior changes

## Reporting issues & getting help

- Open issues with clear reproduction steps and environment details.
- Use repository Discussions for general questions.
- See the project docs: https://kahojyun.github.io/fricon/

## Minimal troubleshooting

- Missing protoc: install `protobuf` (brew/apt)
- Python import/build errors: re-run `uv run maturin develop` after Rust changes
