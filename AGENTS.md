# AGENTS.md

## Development Commands

### Rust Workspace (All crates)

```bash
# Build all crates
cargo build

# Run tests
cargo test

# Check code
cargo check

# Format code
cargo +nightly fmt

# Lint with clippy
cargo clippy --all-targets --all-features
```

### Python Package

```bash
# Install dependencies
uv sync --all-groups

# Build extension for development
uv run maturin develop

# Run tests
uv run pytest

# Format and lint
uv run ruff format
uv run ruff check
uv run basedpyright
```

### Frontend (Tauri UI)

```bash
# Install dependencies (run from project root)
pnpm install

# Development mode
pnpm tauri dev

# Check and lint
pnpm run check
pnpm run format
pnpm run lint:fix
```

### Database Migrations

```bash
# From crates/fricon directory
diesel migration generate <name>
diesel migration run
diesel migration revert
```

## Architecture Overview

Fricon is a data collection automation framework with multiple components:

### Core Architecture

- **Workspace-based design**: Each workspace contains datasets and metadata
- **Client-Server architecture**: gRPC-based communication between client and server
- **Dataset storage**: Apache Arrow format with SQLite metadata tracking
- **Multi-language support**: Rust core with Python bindings and Tauri frontend

### Key Components

1. **crates/fricon**: Core Rust library
    - `Workspace`: Workspace management and initialization
    - `DatasetManager`: Centralized dataset CRUD operations
    - `Client/Server`: gRPC communication layer
    - Database models and migrations (Diesel + SQLite)

2. **crates/fricon-py**: Python bindings
    - PyO3-based Python extension module
    - Provides Workspace, Dataset, DatasetWriter, and Trace classes
    - Handles conversion between Python and Arrow data types
    - IPC client

3. **crates/fricon-ui**: Tauri desktop application
    - Vue3 frontend with PrimeVue components
    - Cross-platform desktop GUI
    - IPC server

4. **crates/fricon-cli**: Command-line interface
    - Workspace initialization
    - Start GUI

### Data Flow

1. User creates/connects to workspace
2. DatasetManager handles dataset creation via gRPC
3. Data written as Arrow files with metadata in SQLite
4. Python bindings provide high-level interface
5. Optional UI for desktop interaction

### Important Implementation Details

- **Dataset IDs**: Each dataset has both an integer ID and UUID
- **Schema inference**: Automatic schema detection from first row written
- **Write sessions**: Managed through write sessions with file locking
- **Batch writing**: Data written in batches for performance
- **Status tracking**: Datasets have writing/completed/aborted states

## Development Notes

### Setup Development Environment

```bash
python3 scripts/setup-dev.py
```

### Key Dependencies

- **Data**: Apache Arrow, Polars, pandas
- **Database**: SQLite with Diesel ORM
- **Serialization**: serde, prost (protobuf)
- **Async**: tokio, futures
- **Python**: PyO3, maturin
- **Frontend**: Vue3, Tauri, PrimeVue

### Code Style

- Rust: Follows standard Rust patterns with clippy pedantic mode
    - Enforce self named module files
    - Ignoring lints with `#[expect(..., reason="...")]` if not applicable
    - Use reason as message in `expect`
- Python: Uses ruff for formatting and linting
- Frontend: Uses ESLint and Prettier

### Testing

- Rust tests: `cargo test`
- Python tests: `uv run pytest` (`uv run maturin develop` required if Rust code changed)

### Compatibility

- Pre-1.0; breaking API/storage changes expected. Prioritize clean architecture over backward support.
