# Fricon Tech Stack

## Core Technologies
- **Rust**: Primary language (edition 2024, workspace toolchain)
- **Python**: 3.9+ with PyO3 bindings
- **Node.js**: 22+ for frontend development
- **SQLite**: Database with Diesel ORM
- **Apache Arrow**: Data format and processing

## Key Dependencies

### Rust Workspace
- **Data**: arrow, arrow-schema, polars, numpy
- **Database**: diesel, diesel_migrations, libsqlite3-sys
- **Serialization**: serde, serde_json, prost, prost-types
- **Async**: tokio, futures, async-stream
- **gRPC**: tonic, tonic-prost
- **Python**: pyo3, pyo3-async-runtimes
- **Tauri**: tauri, tauri-build, tauri-plugin-*
- **Utilities**: uuid, chrono, tempfile, anyhow, thiserror

### Python Package
- **Data processing**: numpy, pandas, polars, pyarrow
- **Development**: ruff, pytest, basedpyright, mypy
- **Build**: maturin

### Frontend (Tauri UI)
- **Framework**: Vue3 with TypeScript
- **UI Components**: PrimeVue
- **Desktop**: Tauri v2
- **Package Manager**: pnpm (workspace)
- **Linting**: ESLint, Prettier

## Build System
- **Rust**: Cargo workspace with 4 crates
- **Python**: maturin for building PyO3 extensions
- **Frontend**: pnpm workspace for Tauri app

## Development Tools
- **Rust**: cargo, rustfmt, clippy
- **Python**: uv (package manager), ruff, pytest
- **Database**: diesel_cli for migrations
- **Protobuf**: protoc for gRPC definitions
- **Frontend**: pnpm, ESLint, Prettier
