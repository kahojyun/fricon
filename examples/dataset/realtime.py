"""Demonstrate how to create a dataset using fricon."""

from __future__ import annotations

import math
from time import sleep

from fricon import DatasetManager, Workspace


def realtime(manager: DatasetManager) -> None:
    with manager.create("Realtime update") as writer:
        for i in range(100):
            for j in range(100):
                sleep(0.01)
                writer.write(x=i, y=j, z=math.sqrt(i * j))


def main():
    ws = Workspace.connect(".dev/ws")
    manager = ws.dataset_manager
    realtime(manager)


if __name__ == "__main__":
    main()
