"""Demonstrate how to create a dataset using fricon."""

from __future__ import annotations

import numpy as np
from fricon import DatasetManager, Trace, Workspace


def simple(manager: DatasetManager) -> None:
    """When no schema is provided, the schema is inferred from the first write."""
    with manager.create(
        "simple example", description="test", tags=["tagA", "tagB"]
    ) as writer:
        for i in range(10):
            # supports primitive types and 1d arrays
            writer.write(a=i, b=i * 2, c=[1, 2, 3])

    d = writer.dataset
    assert d.name == "simple example"
    assert d.description == "test"
    assert set(d.tags) == {"tagA", "tagB"}
    assert d.id is not None


def complex_data_types(manager: DatasetManager) -> None:
    """Example using complex data types and schema inference.

    Demonstrates how to work with complex numbers and lists.
    The schema is automatically inferred from the data types used.
    """
    with manager.create("complex data types") as writer:
        for i in range(10):
            # Write data with various types including complex numbers
            writer.write(
                a=i,  # int
                b=i * 2,  # int
                c=complex(i, i * 2),  # complex number
                d=[1, 2, 3],  # list
                e=Trace.fixed_step(
                    0.1, 0.5, np.exp(1j * np.linspace(0, i * np.pi, 20))
                ),
            )


def multi_index(manager: DatasetManager) -> None:
    with manager.create("Multi indexing") as writer:
        for i in range(10):
            for j in range(10):
                writer.write(i=i, j=j, k=i + 1j * j)


def special_name(manager: DatasetManager) -> None:
    with manager.create("Special name") as writer:
        for i in range(10):
            for j in range(10):
                writer.write_dict(
                    {"Name with space": i, "∆J": j, "$x^2$": i**2, "©ˆ˙¬ƒ˚∆˜": j**0.5}
                )


def main():
    ws = Workspace.connect(".dev/ws")
    manager = ws.dataset_manager
    simple(manager)
    complex_data_types(manager)
    multi_index(manager)
    special_name(manager)


if __name__ == "__main__":
    main()
