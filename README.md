# fricon

Local lab data library and automation foundation for scientific measurement
work.

## Documentation

The active documentation baseline is [docs/](docs/). It owns the
v0.2+ clean-reset product, domain, architecture, ADR, research, and agent
context.

Legacy public and developer documentation content has been merged or retired.
Treat old workspace/dataset-first behavior as prototype history unless a
current `docs/` document says otherwise.

The pre-v0.2 implementation has been removed from this branch. Use the
`archive/v0.1` branch for historical code reference.

## Overview

Fricon v0.2 is being redesigned around:

- one local data library per normal lab computer
- Python-led measurement recording
- measurement-centered dataset artifacts
- live inspection, partial recovery, reopen, and export workflows
- honest provenance for code, setup, parameters, and local configuration

## Quick Start

The repository is in a v0.2 clean-reset planning phase. `docs/` work does
not require dependency installation, a dev workspace, or language builds.

```bash
git clone https://github.com/kahojyun/fricon.git
cd fricon
```

## License

`fricon` is distributed under the terms of the
[MIT](https://spdx.org/licenses/MIT.html) OR
[Apache-2.0](https://spdx.org/licenses/Apache-2.0.html) license.
