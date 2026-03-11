# Concepts

## Workspace

Fricon stores data in a _workspace_. A workspace is a directory that contains all
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
    <uid[0:2]>/
      <uid>/
        data_chunk_0.arrow
        data_chunk_1.arrow (optional, written when first exceeds chunk size)
  backup/
  log/
```

## Fricon Server

Fricon needs a _server process_ to manage the workspace. You can start the server
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

- `uid`: A UUID that is practically unique across all workspaces. This is
  useful when users want to export a dataset to other places.
- `id`: A self-incremental ID that is unique in the current workspace. This is
  more human-readable and can be used to reference a dataset in a given
  workspace.

Users can open a dataset by either `uid` or `id`.

## Desktop UI Architecture

`fricon-ui` is organized as vertical feature slices across the Rust and React boundary.

Rust dependency flow:

```text
desktop_runtime -> api::<feature> -> application::<feature> -> fricon
```

Frontend dependency flow:

```text
app/routes -> features/<feature> -> feature-local api -> shared/lib/tauri.ts -> generated bindings
```

The important boundary rules are:

- Rust `src/api/*.rs` files are Tauri adapters only. They own commands, events, and exported DTOs.
- Rust `src/application/*.rs` files own feature orchestration and should not depend on Tauri types.
- Frontend features own their own `api/`, `ui/`, `model/`, and `hooks/` modules.
- Files under `frontend/src/features/**` use relative imports only.
- `frontend/src/app/**` and `frontend/src/routes/**` import features only through public barrels such as `@/features/<feature>`.
- `frontend/src/shared/lib/tauri.ts` stays generic; feature-specific normalization and query/event wiring belong in each feature's `api/` folder.
