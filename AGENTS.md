# Repository Guidelines

## Project Structure & Module Organization

This is a multi-language workspace with Rust core components, Python bindings, and a Tauri-based UI. The main structure:

- `crates/` - Rust workspace with core modules:
    - `fricon/` - Core library
    - `fricon-py/` - Python bindings
    - `fricon-ui/` - Tauri frontend
    - `fricon-cli/` - Command-line interface
- `examples/` - Usage examples
- `scripts/` - Development and setup utilities
- `docs/` - Documentation source

## Build, Test, and Development Commands

### Rust (workspace)

```bash
cargo check
cargo build
cargo test
cargo +nightly fmt
cargo clippy --all-targets --all-features
```

### Python

ALWAYS run `maturin develop` before running `pytest` unless you are sure that the Python bindings are up to date.

```bash
# Run from repo root
uv run maturin develop
uv run pytest
uv run ruff format
uv run ruff check
```

### Frontend/UI

```bash
# Run from repo root
pnpm install
pnpm run check # Type checking and linting
pnpm run fix # Fix auto-fixable issues
pnpm run test --run
```

## Testing Guidelines

- **Testing Frameworks**: Rust (`cargo test`), Python (`pytest`), Frontend (`vitest`)
- **Coverage**: Write tests for critical paths
- **Test Location**:
    - Rust unit tests in `mod tests`, integration tests in `<crate>/tests`
    - Python tests in `crates/fricon-py/tests/`
    - Frontend tests in `crates/fricon-ui/frontend/src/**/*.test.*`
- **Running**: Use `cargo test` for Rust, `uv run pytest` for Python, `pnpm run test` for Frontend
