"""Create a sample dataset for testing."""

from __future__ import annotations

from fricon import DatasetManager, Trace, Workspace


def simple(manager: DatasetManager) -> None:
    with manager.create("line", index=["x", "y"]) as writer:
        for i in range(10):
            for j in range(10):
                writer.write(
                    x=i,
                    y=j,
                    z=i + j,
                    t=[1, 2, 3, 4],
                    t2=Trace.fixed_step(0.1, 1, [1, 2, 3, 4]),
                    t3=Trace.variable_step([2, 3, 4, 6], [1, 2, 3, 4]),
                )


if __name__ == "__main__":
    ws = Workspace.connect(".dev/ws")
    manager = ws.dataset_manager
    simple(manager)
