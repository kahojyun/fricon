# fricon

Data collection automation framework.

## Documentation

[GitHub Pages](https://kahojyun.github.io/fricon/)

## Overview

Fricon is a data collection automation framework designed for managing datasets in scientific and research workflows. It provides:

- **Workspace Management**: Organized data storage with metadata tracking
- **Dataset Operations**: Arrow-format data tables with UUID and incremental ID tracking
- **Desktop UI**: Cross-platform Tauri application with Vue3 frontend
- **Server Architecture**: IPC-based server for workspace operations

## Quick Start

### Installation

**For Python users (recommended):**

```bash
pip install fricon
```

**For development or building from source:**

Building from source requires several dependencies including Rust, protoc, uv, pnpm, and platform-specific libraries. See [CONTRIBUTING.md](CONTRIBUTING.md) for complete setup instructions.

```bash
git clone https://github.com/kahojyun/fricon.git
cd fricon
python3 scripts/setup-dev.py
```

### Basic Usage

```python
from fricon import Workspace

# Initialize a new workspace from CLI
# fricon workspace init path/to/workspace

# Connect to existing workspace
ws = Workspace.connect("path/to/workspace")

# Create and manipulate datasets
dataset = ws.create_dataset(data)
```

## Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for detailed guidelines on development environment setup, building components, testing, and contribution workflow.

## License

`fricon` is distributed under the terms of the
[MIT](https://spdx.org/licenses/MIT.html) OR
[Apache-2.0](https://spdx.org/licenses/Apache-2.0.html) license.
