# fricon

[![PyPI - Version](https://img.shields.io/pypi/v/fricon.svg)](https://pypi.org/project/fricon)
[![PyPI - Python Version](https://img.shields.io/pypi/pyversions/fricon.svg)](https://pypi.org/project/fricon)

-----

## Table of Contents

- [Installation](#installation)
- [License](#license)

## Installation

Not published yet.

## Development

This project use `uv` for python development.

```console
uv sync
uv run examples/dataset.py # Assuming the server is running
```

Grpc binding files are generated automatically during the build process. When
protos are changed, `uv run` or `uv sync` will automatically regenerate the
bindings.

## License

`fricon` is distributed under the terms of the
[MIT](https://spdx.org/licenses/MIT.html) OR
[Apache-2.0](https://spdx.org/licenses/Apache-2.0.html) license.
