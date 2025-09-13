"""Demonstrate how to create a dataset using fricon."""

from __future__ import annotations

from fricon import DatasetManager, Workspace


def simple(manager: DatasetManager) -> None:
    """When no schema is provided, the schema is inferred from the first write."""
    with manager.create("example", description="test", tags=["tagA", "tagB"]) as writer:
        for i in range(10):
            for j in range(10):
                # supports primitive types and 1d arrays
                writer.write(i=i, j=j, a=i, b=i * j, c=[1, 2, 3])

    d = writer.dataset
    assert d.name == "example"
    assert d.description == "test"
    assert set(d.tags) == {"tagA", "tagB"}
    assert d.id is not None


def with_schema(manager: DatasetManager) -> None:
    """Schema is now automatically inferred from the first write.
    
    The schema inference supports fricon's core types: float64, complex128, and traces.
    """
    with manager.create("example") as writer:
        for i in range(10):
            writer.write(a=i, b=i * 2, c=1j, d=[1, 2])


if __name__ == "__main__":
    ws = Workspace.connect(".dev/ws")
    manager = ws.dataset_manager
    simple(manager)
    with_schema(manager)
