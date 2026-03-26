from __future__ import annotations

import sys
from pathlib import Path

from fricon import Trace, Workspace


def main() -> None:
    if len(sys.argv) != 2:
        raise SystemExit("Usage: python examples/simple/create.py <workspace-path>")

    workspace_path = Path(sys.argv[1])
    # --8<-- [start:create-example]
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
    # --8<-- [end:create-example]


if __name__ == "__main__":
    main()
