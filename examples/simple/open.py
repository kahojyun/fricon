from __future__ import annotations

import sys
from pathlib import Path
from typing import cast

from fricon import Workspace


def main() -> None:
    if len(sys.argv) != 2:
        raise SystemExit("Usage: python examples/simple/open.py <workspace-path>")

    workspace_path = Path(sys.argv[1])
    # --8<-- [start:open-example]
    ws = Workspace.connect(workspace_path)
    manager = ws.dataset_manager
    df_index = manager.list_all()  # Returns a pandas DataFrame indexed by dataset id
    id_ = cast(int, df_index.index[0])
    assert isinstance(id_, int)
    dataset = manager.open(id_)
    print(dataset.id)
    _ = dataset.to_polars()
    # --8<-- [end:open-example]


if __name__ == "__main__":
    main()
