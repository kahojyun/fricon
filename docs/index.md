# Fricon

Fricon is a framework for data collection automation.

## Current features

- Data storage.
- Desktop UI for browsing and managing datasets.

## Usage

Install via PyPI:

```shell
pip install fricon
```

Initialize workspace via CLI:

```shell
fricon init path/to/workspace
```

Launch the desktop UI for a workspace:

```shell
fricon gui path/to/workspace
```

Connect from Python to a workspace with a running server:

```python
from fricon import Workspace

ws = Workspace.connect("path/to/workspace")
```

Create and populate a dataset from Python:

```python title="examples/simple/create.py"
--8<-- "examples/simple/create.py:create-example"
```

Query and open a dataset from Python:

```python title="examples/simple/open.py"
--8<-- "examples/simple/open.py:open-example"
```
