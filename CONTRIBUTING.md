# Contributing to Fricon

Thank you for your interest in contributing to Fricon! This document provides guidelines and information for contributors.

## Development Environment Setup

### Prerequisites

#### System Dependencies

**Ubuntu/Debian:**

```bash
sudo apt-get update
sudo apt-get install -y \
    build-essential \
    pkg-config \
    libsqlite3-dev \
    protobuf-compiler \
    curl \
    git
```

**macOS:**

```bash
# Install Homebrew if not already installed
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# Install dependencies
brew install protobuf sqlite3 pkg-config
```

**Windows:**

```powershell
# Install via Chocolatey (install chocolatey first if needed)
choco install protoc
```

**Linux (Additional for Tauri UI):**

```bash
# Required for Tauri 2.0 on Linux (refer to official guide for latest info)
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
```

For the latest information, refer to the [official Tauri prerequisites guide](https://tauri.app/start/prerequisites/).

#### Programming Language Requirements

- **Rust**: Stable toolchain (automatically managed via `rust-toolchain.toml`)
- **Python**: 3.9+ (project uses Python 3.9 as specified in `.python-version`)
- **Node.js**: 22+ for frontend development

#### Package Managers and Tools

- **uv**: Modern Python package manager
- **pnpm**: Fast, disk space efficient Node.js package manager
- **diesel_cli**: Database migration tool for Rust

### Installation Steps

1. **Clone the repository:**

    ```bash
    git clone https://github.com/kahojyun/fricon.git
    cd fricon
    ```

2. **Install Rust toolchain:**

    ```bash
    # Rust will be automatically installed/configured via rust-toolchain.toml
    # when you run cargo commands
    ```

3. **Install uv (Python package manager):**

    Follow the official installation guide at [uv installation](https://docs.astral.sh/uv/getting-started/installation/)

    ```bash
    # Quick install via curl (recommended)
    curl -LsSf https://astral.sh/uv/install.sh | sh
    ```

4. **Install pnpm (Node.js package manager):**

    Follow the official installation guide at [pnpm installation](https://pnpm.io/installation)

    ```bash
    # Quick install via curl
    curl -fsSL https://get.pnpm.io/install.sh | sh
    ```

5. **Install diesel_cli (Rust database tool):**

    ```bash
    cargo install diesel_cli --no-default-features --features sqlite
    ```

6. **Run the development setup script:**

    ```bash
    python3 scripts/setup-dev.py
    ```

    This script will:
    - Create a `.dev` folder for development files
    - Set up the `.env` file with database configuration
    - Initialize and run database migrations

### Project Structure

```
fricon/
├── crates/
│   ├── fricon/          # Core Rust library
│   ├── fricon-py/       # Python bindings (PyO3)
│   ├── fricon-ui/       # Tauri desktop application
│   │   └── frontend/    # Vue3 frontend
│   └── fricon-cli/      # Command-line interface
├── docs/                # Documentation
├── examples/            # Python usage examples
├── scripts/             # Development scripts
├── tests/               # Python test suite
├── pyproject.toml       # Python project configuration
├── Cargo.toml           # Rust workspace configuration
└── package.json         # Node.js dependencies
```

## Development Workflow

### Building the Project

#### Rust Components

```bash
# Check all Rust crates
cargo check

# Build all crates
cargo build

# Build specific crate
cargo build -p fricon
cargo build -p fricon-ui
```

#### Python Development

```bash
# Set up Python environment and install dependencies
uv sync --dev

# Build Python package (requires Rust toolchain)
uv run maturin develop

# Note: After modifying Rust code, you need to re-run maturin develop for changes to take effect

# Run Python tests
uv run pytest
```

#### Frontend Development (UI)

```bash
# Navigate to frontend directory
cd crates/fricon-ui/frontend

# Install dependencies
pnpm install

# Start development server
pnpm run dev

# Build for production
pnpm run build
```

### Testing

#### Rust Tests

```bash
# Run all Rust tests
cargo test

# Run tests for specific crate
cargo test -p fricon
```

#### Python Tests

```bash
# Run Python tests with uv
uv run pytest

# Run with coverage
uv run pytest --cov=fricon
```

#### Frontend Tests

```bash
cd crates/fricon-ui/frontend
pnpm run test
```

### Code Style and Linting

#### Rust

```bash
# Format code
cargo fmt

# Run linter
cargo clippy

# Run clippy with all features
cargo clippy --all-features
```

#### Python

```bash
# Format with ruff
uv run ruff format

# Lint with ruff
uv run ruff check

# Type checking with mypy
uv run mypy
```

#### Frontend (TypeScript/Vue)

```bash
cd crates/fricon-ui/frontend

# Lint with ESLint
pnpm run lint

# Format with Prettier
pnpm run format

# Type checking
pnpm run type-check
```

### Database Management

The project uses SQLite with Diesel ORM for database operations.

```bash
# Set up database (done automatically by setup-dev.py)
cd crates/fricon
diesel setup

# Create new migration
diesel migration generate migration_name

# Run migrations
diesel migration run

# Rollback last migration
diesel migration revert
```

## Contributing Guidelines

### Code Style

#### Rust

- Follow the official Rust style guide
- Use `cargo fmt` for formatting
- Address all `cargo clippy` warnings
- Write documentation for public APIs
- Use `Result<T, Error>` for error handling, avoid `panic!` and `unwrap()`

#### Python

- Follow PEP 8 style guide
- Use type hints for all function signatures
- Write docstrings for public functions and classes
- Use `ruff` for formatting and linting

#### Vue3/TypeScript

- Use Composition API with `<script setup lang="ts">` syntax
- Follow Vue.js style guide
- Use Pinia for state management
- Apply Tailwind CSS for styling

### Commit Messages

Use conventional commit format:

```
type(scope): description

[optional body]

[optional footer]
```

Types: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`

Examples:

- `feat(core): add new dataset export functionality`
- `fix(ui): resolve workspace loading issue`
- `docs: update installation instructions`

### Pull Request Process

1. **Fork the repository** and create a feature branch
2. **Make your changes** following the code style guidelines
3. **Add tests** for new functionality
4. **Update documentation** if needed
5. **Run all tests and linting** to ensure everything passes
6. **Submit a pull request** with a clear description

### Issue Reporting

When reporting issues:

- Use the issue templates if available
- Provide clear reproduction steps
- Include relevant system information
- Attach logs or error messages

## Project-Specific Notes

### Architecture Overview

Fricon is a data collection automation framework with the following components:

- **Core Library (`fricon`)**: Main Rust library with workspace management, dataset handling, and server functionality
- **Python Bindings (`fricon-py`)**: PyO3-based Python interface to the core library
- **Desktop UI (`fricon-ui`)**: Tauri-based desktop application with Vue3 frontend
- **CLI (`fricon-cli`)**: Command-line interface for workspace and server management

### Key Concepts

- **Workspace**: A directory containing data files, metadata, and configuration
- **Dataset**: Arrow-format data tables with metadata, identified by UUID and incremental ID
- **Server**: IPC-based server process for managing workspace operations

### Development Dependencies

The project uses several development tools:

- **release-plz**: Automated release management
- **pre-commit**: Git hooks for code quality
- **GitHub Actions**: CI/CD workflows
- **mkdocs**: Documentation generation

## Getting Help

- **Documentation**: [GitHub Pages](https://kahojyun.github.io/fricon/)
- **Issues**: [GitHub Issues](https://github.com/kahojyun/fricon/issues)
- **Discussions**: Use GitHub Discussions for questions and ideas

## Quick Reference

### Development Utility Script

For convenience, use the development utility script:

```bash
# Set up development environment
python3 scripts/dev.py setup

# Build components
python3 scripts/dev.py build rust
python3 scripts/dev.py build python
python3 scripts/dev.py build frontend

# Run tests
python3 scripts/dev.py test rust
python3 scripts/dev.py test python
python3 scripts/dev.py test frontend

# Linting and formatting
python3 scripts/dev.py lint        # Check all code style
python3 scripts/dev.py fix         # Fix auto-fixable issues

# Clean build artifacts
python3 scripts/dev.py clean
```

### Common Development Tasks

**Full project build and test:**

```bash
# Set up environment (run once)
python3 scripts/setup-dev.py

# Build all components
cargo build
uv run maturin develop
cd crates/fricon-ui/frontend && pnpm install && pnpm run build

# Run all tests
cargo test
uv run pytest
cd crates/fricon-ui/frontend && pnpm run test
```

**Working with specific components:**

```bash
# Core Rust development
cargo check -p fricon
cargo test -p fricon

# Python development
uv run python examples/basic_usage.py
uv run pytest tests/

# UI development
cd crates/fricon-ui/frontend
pnpm run dev  # Start dev server
pnpm run build  # Build for production
```

**Database operations:**

```bash
cd crates/fricon
diesel migration generate new_feature
diesel migration run  # Automatically updates schema.rs in src/database/
```

**Code quality checks:**

```bash
# Rust
cargo fmt --all
cargo clippy --all-targets --all-features

# Python
uv run ruff format .
uv run ruff check .
uv run mypy .

# Frontend
cd crates/fricon-ui/frontend
pnpm run lint
pnpm run format
```

### Troubleshooting

**Build Issues:**

1. **Protobuf compiler not found:**

    ```bash
    # Ubuntu/Debian
    sudo apt-get install protobuf-compiler

    # macOS
    brew install protobuf
    ```

2. **SQLite development libraries missing:**

    ```bash
    # Ubuntu/Debian
    sudo apt-get install libsqlite3-dev

    # macOS
    brew install sqlite3
    ```

3. **Diesel CLI not found:**

    ```bash
    cargo install diesel_cli --no-default-features --features sqlite
    ```

4. **Node.js/pnpm issues:**
    ```bash
    # Update to Node.js 22+
    # Install pnpm
    npm install -g pnpm
    ```

**Runtime Issues:**

1. **Database connection errors:**
    - Ensure `.env` file exists and has correct `DATABASE_URL`
    - Run `python3 scripts/setup-dev.py` to recreate

2. **Python import errors:**
    - Make sure you've run `uv run maturin develop`
    - Check that Python environment is activated

3. **UI build failures:**
    - Ensure all system dependencies are installed
    - Try deleting `node_modules` and running `pnpm install` again

**Performance Tips:**

- Use `cargo check` instead of `cargo build` for faster compilation during development
- Use `--release` flag for performance testing: `cargo build --release`
- Enable incremental compilation: set `CARGO_INCREMENTAL=1`

## License

This project is licensed under MIT OR Apache-2.0. See [LICENSE-MIT](LICENSE-MIT) and [LICENSE-APACHE](LICENSE-APACHE) for details.
