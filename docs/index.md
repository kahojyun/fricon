# Fricon

Fricon is a framework for data collection automation.

## Current features

- Data storage.
- Desktop UI for browsing and managing datasets.

## Usage

Install via PyPI on supported wheel platforms:

```shell
pip install fricon
```

If PyPI does not provide a compatible wheel for your system, build from source
using the repository setup instructions.

Initialize workspace via CLI:

```shell
fricon init path/to/workspace
```

Launch the desktop UI for a workspace:

```shell
fricon-gui path/to/workspace
```

Connect from Python to a workspace with a running server:

```python
from fricon import Workspace

ws = Workspace.connect("path/to/workspace")
```

Create and populate a dataset from Python:

```python
from pathlib import Path

from fricon import Trace, Workspace

workspace_path = Path("path/to/workspace")
ws = Workspace.connect(workspace_path)
manager = ws.dataset_manager

with manager.create("example_dataset") as writer:
    writer.write(
        i=1,
        a=42.0,
        b=[1.0, 2.0],
        c=[1 + 2j, 3 + 4j],
        d=Trace.fixed_step(0.1, 1.1, [1, 2, 3]),
    )

print(f"Id of the dataset: {writer.dataset.id}")
```

Query and open a dataset from Python:

```python
from pathlib import Path

from fricon import Workspace

workspace_path = Path("path/to/workspace")
ws = Workspace.connect(workspace_path)
manager = ws.dataset_manager
datasets = manager.list_all()
dataset_id = int(datasets.index[0])
dataset = manager.open(dataset_id)
table = dataset.to_arrow()
```

Runnable script versions are available in `examples/simple/`.
