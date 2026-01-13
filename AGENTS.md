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

```bash
# Run from repo root
uv run maturin develop # Build Python extension for development
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
pnpm run test
pnpm run bench
```

## Coding Style & Naming Conventions

- **Rust**: Uses `rustfmt` and `clippy` with pedantic linting enabled. Follow Rust conventions, avoid `unwrap()`, `dbg_macro`, and `todo` macros.
- **Python**: Uses `ruff` for formatting and linting. Follow PEP 8 conventions.
- **Frontend**: Uses `prettier` and `eslint` with TypeScript.
- All commits should follow conventional format: `type(scope): description`

## Testing Guidelines

- **Testing Frameworks**: Rust (`cargo test`), Python (`pytest`), Frontend (`vitest`)
- **Coverage**: Write tests for new functionality when possible
- **Test Location**:
    - Rust unit tests in `mod tests`, integration tests in `<crate>/tests`
    - Python tests in `crates/fricon-py/tests/`
    - Frontend tests in `crates/fricon-ui/frontend/src/**/__tests__`
- **Running**: Use `cargo test` for Rust, `uv run pytest` for Python, `pnpm run test` for Frontend

## Commit & Pull Request Guidelines

- **Conventional Commits**: Use `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore` prefixes
- **Branch Strategy**: Feature/bugfix branches from `main`
- **PR Requirements**:
    - Pass all linters and tests
    - Include tests for new features
    - Update documentation if behavior changes
    - Clear description of changes
- **Pre-commit**: Automatic formatting and linting checks enforced via CI
