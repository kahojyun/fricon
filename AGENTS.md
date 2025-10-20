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

Directly run in project root without cd

### Rust (workspace)

```bash
cargo check          # Verify compilation
cargo build          # Build all workspace members
cargo test           # Run tests
cargo +nightly fmt   # Format code
cargo clippy --all-targets --all-features  # Lint
```

### Python

```bash
uv run maturin develop   # Build Python extension for development
uv run pytest            # Run tests
uv run ruff format       # Format Python code
uv run ruff check        # Lint Python code
```

### Frontend/UI

```bash
pnpm install         # Install workspace dependencies
pnpm run check       # Type checking and linting
pnpm run format      # Format code
pnpm tauri dev       # Run development server
```

## Coding Style & Naming Conventions

- **Rust**: Uses `rustfmt` and `clippy` with pedantic linting enabled. Follow Rust conventions, avoid `unwrap()`, `dbg_macro`, and `todo` macros.
- **Python**: Uses `ruff` for formatting and linting. Follow PEP 8 conventions.
- **Frontend**: Uses `prettier` and `eslint` with TypeScript.
- All commits should follow conventional format: `type(scope): description`

## Testing Guidelines

- **Testing Frameworks**: Rust (`cargo test`), Python (`pytest`)
- **Coverage**: Write tests for new functionality when possible
- **Test Location**:
    - Rust unit tests in `mod tests`, integration tests in `<crate>/tests`
    - Python tests in `crates/fricon-py/tests/`
- **Running**: Use `cargo test` for Rust, `uv run pytest` for Python

## Commit & Pull Request Guidelines

- **Conventional Commits**: Use `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore` prefixes
- **Branch Strategy**: Feature/bugfix branches from `main`
- **PR Requirements**:
    - Pass all linters and tests
    - Include tests for new features
    - Update documentation if behavior changes
    - Clear description of changes
- **Pre-commit**: Automatic formatting and linting checks enforced via CI
