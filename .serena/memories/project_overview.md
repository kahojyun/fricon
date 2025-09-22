# Fricon Project Overview

## Project Purpose
Fricon is a data collection automation framework designed for managing datasets in scientific and research workflows. It provides workspace management, dataset operations with Arrow-format data tables, a cross-platform desktop UI (Tauri + Vue3), and IPC-based server architecture.

## Architecture
- **Workspace-based design**: Each workspace contains datasets and metadata
- **Client-Server architecture**: gRPC-based communication between client and server
- **Dataset storage**: Apache Arrow format with SQLite metadata tracking
- **Multi-language support**: Rust core with Python bindings and Tauri frontend

## Key Components
1. **crates/fricon**: Core Rust library with Workspace, DatasetManager, Client/Server gRPC communication
2. **crates/fricon-py**: Python bindings using PyO3, provides Workspace, Dataset, DatasetWriter, Trace classes
3. **crates/fricon-ui**: Tauri desktop application with Vue3 frontend and PrimeVue components
4. **crates/fricon-cli**: Command-line interface for workspace initialization and GUI

## Data Flow
1. User creates/connects to workspace
2. DatasetManager handles dataset creation via gRPC
3. Data written as Arrow files with metadata in SQLite
4. Python bindings provide high-level interface
5. Optional UI for desktop interaction

## Important Implementation Details
- Dataset IDs: Each dataset has both an integer ID and UUID
- Schema inference: Automatic schema detection from first row written
- Write sessions: Managed through write sessions with file locking
- Batch writing: Data written in batches for performance
- Status tracking: Datasets have writing/completed/aborted states
- Pre-1.0; breaking API/storage changes expected
