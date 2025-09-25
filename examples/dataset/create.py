"""Demonstrate how to create a dataset using fricon."""

from __future__ import annotations

from fricon import DatasetManager, Workspace


def simple(manager: DatasetManager) -> None:
    """When no schema is provided, the schema is inferred from the first write."""
    with manager.create("example", description="test", tags=["tagA", "tagB"]) as writer:
        for i in range(10):
            # supports primitive types and 1d arrays
            writer.write(a=i, b=i * 2, c=[1, 2, 3])

    d = writer.dataset
    assert d.name == "example"
    assert d.description == "test"
    assert set(d.tags) == {"tagA", "tagB"}
    assert d.id is not None


def complex_data_types(manager: DatasetManager) -> None:
    """Example using complex data types and schema inference.

    Demonstrates how to work with complex numbers and lists.
    The schema is automatically inferred from the data types used.
    """
    with manager.create("complex_example") as writer:
        for i in range(10):
            # Write data with various types including complex numbers
            writer.write(
                a=i,  # int
                b=i * 2,  # int
                c=complex(i, i * 2),  # complex number
                d=[1, 2, 3],  # list
            )


if __name__ == "__main__":
    ws = Workspace.connect(".dev/ws")
    manager = ws.dataset_manager
    simple(manager)
    complex_data_types(manager)
