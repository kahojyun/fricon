# Fricon Development Commands

## Setup Commands
```bash
# Initial development setup (creates .dev, .env, runs migrations)
python3 scripts/setup-dev.py

# Install dependencies
uv sync --all-groups          # Python
pnpm install                  # Frontend (from project root)
```

## Rust Workspace Commands
```bash
# Build all crates
cargo build

# Build in release mode
cargo build --release

# Run tests
cargo test

# Check code (quick validation)
cargo check

# Format code
cargo fmt

# Lint with clippy
cargo clippy --all-targets --all-features

# Clean build artifacts
cargo clean
```

## Python Package Commands
```bash
# Build extension for development
uv run maturin develop

# Run tests
uv run pytest

# Format code
uv run ruff format

# Lint code
uv run ruff check

# Type checking
uv run basedpyright
uv run mypy
```

## Frontend (Tauri UI) Commands
```bash
# Development mode
pnpm tauri dev

# Build frontend
pnpm tauri build

# Check and lint
pnpm run check              # pnpm run lint && pnpm run format:check
pnpm run format             # prettier --write .
pnpm run lint               # eslint .
pnpm run lint:fix           # eslint . --fix
```

## Database Migrations
```bash
# From crates/fricon directory
diesel migration generate <name>    # Create new migration
diesel migration run                # Run pending migrations
diesel migration revert             # Rollback last migration
diesel migration redo               # Revert and re-run last migration
diesel setup                        # Initial database setup
```

## System Utilities (Darwin/macOS)
```bash
# File operations
ls -la                             # List files with details
find . -name "*.rs"                # Find Rust files
grep -r "pattern" .                # Search recursively

# Git operations
git status                         # Check git status
git add .                          # Stage all changes
git commit -m "message"            # Commit changes
git push                           # Push to remote
git pull                           # Pull from remote

# Process management
ps aux | grep fricon              # Find fricon processes
kill -9 <pid>                     # Kill process by ID

# Development tools
code .                            # Open in VS Code
open .                            # Open in Finder
```

## Entry Points
```bash
# Python CLI
fricon workspace init path/to/workspace
fricon-gui                        # Launch GUI

# Rust CLI (from build)
cargo run --bin fricon-cli -- workspace init path/to/workspace
cargo run --bin fricon-ui         # Launch UI
```

## Quality Assurance (Run Before Committing)
```bash
# Rust
cargo fmt --check                 # Check formatting
cargo clippy --all-targets --all-features  # Lint
cargo test                        # Run tests

# Python
uv run ruff format --check         # Check formatting
uv run ruff check                  # Lint
uv run basedpyright               # Type check
uv run pytest                      # Run tests

# Frontend
pnpm run check                     # Lint and format check
```

## Troubleshooting
```bash
# Missing protoc (macOS)
brew install protobuf

# Missing diesel CLI
cargo install diesel_cli --no-default-features --features sqlite

# Rebuild Python extension after Rust changes
uv run maturin develop
```