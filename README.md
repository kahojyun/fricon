# fricon

Data collection automation framework.

## Documentation

[GitHub Pages](https://kahojyun.github.io/fricon/)

## Overview

Fricon is a multi-language data collection automation framework designed for managing datasets in scientific and research workflows. It provides:

- **Workspace Management**: Organized data storage with metadata tracking
- **Dataset Operations**: Arrow-format data tables with UUID and incremental ID tracking  
- **Multi-language Support**: Core Rust library with Python bindings
- **Desktop UI**: Cross-platform Tauri application with Vue3 frontend
- **Server Architecture**: IPC-based server for workspace operations

## Quick Start

### Installation

**Prerequisites:** Ensure you have [Rust](https://rustup.rs/), [Python 3.9+](https://python.org), and [Node.js 22+](https://nodejs.org/) installed.

1. **Install package managers:**
   ```bash
   # Install uv (Python package manager)
   pip install uv
   
   # Install pnpm (Node.js package manager)
   npm install -g pnpm
   ```

2. **Install from PyPI (Python users):**
   ```bash
   pip install fricon
   ```

3. **Or build from source:**
   ```bash
   git clone https://github.com/kahojyun/fricon.git
   cd fricon
   python3 scripts/setup-dev.py
   ```

### Basic Usage

```python
from fricon import Workspace

# Create a new workspace
ws = Workspace.init("path/to/workspace")

# Or connect to existing workspace
ws = Workspace.connect("path/to/workspace")

# Create and manipulate datasets
dataset = ws.create_dataset(data)
```

## Development

### System Requirements

- **Rust**: Stable toolchain (managed via `rust-toolchain.toml`)
- **Python**: 3.9+ (see `.python-version`)
- **Node.js**: 22+ for frontend development
- **System Dependencies:**
  - Protobuf compiler (`protoc`)
  - SQLite3 development libraries
  - Build tools (`build-essential` on Ubuntu, Xcode on macOS)

### Development Dependencies

- **uv**: Modern Python package manager
- **pnpm**: Fast Node.js package manager
- **diesel_cli**: Database migrations for Rust

### Quick Development Setup

1. **Install system dependencies:**
   ```bash
   # Ubuntu/Debian
   sudo apt-get install build-essential pkg-config libsqlite3-dev protobuf-compiler
   
   # macOS
   brew install protobuf sqlite3 pkg-config
   ```

2. **Install tools:**
   ```bash
   # Python package manager
   pip install uv
   
   # Node.js package manager
   npm install -g pnpm
   
   # Rust database tool
   cargo install diesel_cli --no-default-features --features sqlite
   ```

3. **Set up development environment:**
   ```bash
   python3 scripts/setup-dev.py
   ```

   Or use the development utility script:
   ```bash
   python3 scripts/dev.py setup
   ```

This script automatically:
- Creates development folder structure
- Sets up environment variables
- Initializes and runs database migrations

### Building Components

```bash
# Rust (core library)
cargo build -p fricon
# or
python3 scripts/dev.py build rust

# Python bindings
uv run maturin develop
# or
python3 scripts/dev.py build python

# Desktop UI
cargo build -p fricon-ui

# Frontend development
cd crates/fricon-ui/frontend && pnpm run dev
# or
python3 scripts/dev.py build frontend --dev
```

### Testing

```bash
# Rust tests
cargo test
# or 
python3 scripts/dev.py test rust

# Python tests  
uv run pytest
# or
python3 scripts/dev.py test python

# Frontend tests
cd crates/fricon-ui/frontend && pnpm run test
# or
python3 scripts/dev.py test frontend

# All linting and formatting
python3 scripts/dev.py lint
python3 scripts/dev.py fix
```

For detailed development instructions, see [CONTRIBUTING.md](CONTRIBUTING.md).

## Project Structure

- `crates/fricon/`: Core Rust library
- `crates/fricon-py/`: Python bindings (PyO3)
- `crates/fricon-ui/`: Desktop application (Tauri + Vue3)
- `crates/fricon-cli/`: Command-line interface
- `docs/`: Documentation source
- `examples/`: Usage examples
- `tests/`: Python test suite

## Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for detailed guidelines on:
- Setting up the development environment
- Code style and testing requirements
- Pull request process
- Issue reporting

## License

`fricon` is distributed under the terms of the
[MIT](https://spdx.org/licenses/MIT.html) OR
[Apache-2.0](https://spdx.org/licenses/Apache-2.0.html) license.
