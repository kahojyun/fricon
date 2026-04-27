# Concepts

## Workspace

Fricon stores data in a _workspace_. A workspace is a directory that contains all
the data files and metadata. You can create a workspace using the CLI:

```shell
fricon init path/to/workspace
```

A workspace should be treated as Fricon-managed storage. Use the CLI, Python
API, or desktop UI to read and modify it.

## Fricon Server

Fricon needs a _server process_ to manage the workspace. The desktop app can
launch it for you:

```shell
fricon-gui path/to/workspace
```

Python scripts can connect to an already running workspace server:

```python
from fricon import Workspace

ws = Workspace.connect("path/to/workspace")
```

Python scripts connect to the server with the workspace path.

## Dataset

Fricon allows users to store data in datasets. A dataset stores one and only
one data table based on the Arrow format with additional metadata.

### Identifiers

Each dataset will be given two unique identifiers:

- `uid`: A UUID that is practically unique across all workspaces. This is
  useful when users want to export a dataset to other places.
- `id`: A self-incremental ID that is unique in the current workspace. This is
  more human-readable and can be used to reference a dataset in a given
  workspace.

Users can open a dataset by either `uid` or `id`.
