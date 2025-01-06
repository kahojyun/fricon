# Concepts

## Workspace

Fricon stores data in a *workspace*. A workspace is a directory that contains all
the data files and metadata. You can create a workspace using the CLI:

```shell
fricon init path/to/workspace
```

Currently a workspace contains the following files:

```tree
workspace/
  .fricon_version
  fricon.sqlite3
  data/
    <date>/
      <uuid>/
        dataset.arrow
  backup/
  log/
```

## Fricon Server

Fricon needs a *server process* to manage the workspace. You can start the server
using the CLI:

```shell
fricon serve path/to/workspace
```

The server process will listen to an IPC socket based on the workspace path. The
client connects to the server with the workspace path.

```python
from fricon import Workspace

ws = Workspace.connect("path/to/workspace")
```

## Dataset

Fricon allows users to store data in datasets. A dataset stores one and only
one data table based on the Arrow format with additional metadata.

### Identifiers

Each dataset will be given two unique identifiers:

* `uid`: A UUID that is practically unique across all workspaces. This is
useful when users want to export a dataset to other places.
* `id`: A self-incremental ID that is unique in the current workspace. This is
more human-readable and can be used to reference a dataset in a given
workspace.

Users can open a dataset by either `uid` or `id`.
